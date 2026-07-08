use crate::claude_cli::run_claude;
use crate::config::Config;
use crate::gate::{decide, parse_gate_output, Decision};
use crate::prompts::{drafter_prompt, gate_prompt};
use crate::telemetry::{append, stats, Event};

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn task_id(task: &str) -> String {
    let digest = Sha256::digest(task.as_bytes());
    let hash: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    format!("{hash}-{}", now_ms())
}

fn log_event(cfg: &Config, event: &Event) {
    let path = cfg.data_dir.join("events.jsonl");
    if let Err(e) = append(&path, event) {
        eprintln!("dspec: telemetry write failed: {e}");
    }
}

pub async fn run_draft_pipeline(cfg: &Config, task: &str, context: Option<&str>) -> serde_json::Value {
    let id = task_id(task);
    let mut error: Option<String> = None;

    let t0 = Instant::now();
    let draft = match run_claude(cfg, &cfg.draft_model, &drafter_prompt(task, context)).await {
        Ok(d) => d,
        Err(e) => {
            error = Some(e.to_string());
            String::new()
        }
    };
    let draft_ms = t0.elapsed().as_millis() as u64;

    let mut confidence: Option<u8> = None;
    let mut reasons: Vec<String> = Vec::new();
    let mut gate_ms = 0u64;
    if error.is_none() {
        let t1 = Instant::now();
        let gate_result = run_claude(cfg, &cfg.gate_model, &gate_prompt(task, &draft)).await;
        gate_ms = t1.elapsed().as_millis() as u64;
        match gate_result.and_then(|raw| parse_gate_output(&raw)) {
            Ok(score) => {
                confidence = Some(score.confidence);
                reasons = score.reasons;
            }
            Err(e) => error = Some(e.to_string()),
        }
    }

    // Fail-open: any error path is a discard. Never block the task.
    let decision = match (confidence, &error) {
        (Some(c), None) => decide(c, cfg.threshold),
        _ => Decision::Discard,
    };

    log_event(
        cfg,
        &Event::Draft {
            id: id.clone(),
            ts_ms: now_ms(),
            confidence,
            decision: serde_json::to_value(decision)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_else(|| "discard".into()),
            draft_ms,
            gate_ms,
            error: error.clone(),
        },
    );

    json!({
        "draft_id": id,
        "decision": decision,
        "confidence": confidence,
        "reasons": reasons,
        "draft": draft,
        "draft_ms": draft_ms,
        "gate_ms": gate_ms,
        "error": error,
    })
}

#[derive(Clone)]
pub struct DspecServer {
    pub cfg: Config,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DraftTaskParams {
    /// The task to draft, exactly as the user stated it plus any clarifications.
    pub task: String,
    /// Minimal context (file excerpts, constraints) the drafter needs. Be frugal.
    pub context: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecordOutcomeParams {
    /// The draft_id returned by draft_task.
    pub draft_id: String,
    /// One of: accepted | patched | rejected | discarded
    pub outcome: String,
    /// If patched: estimated fraction of the draft that was changed, 0.0-1.0.
    pub patch_ratio: Option<f32>,
}

#[tool_router]
impl DspecServer {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    #[tool(
        description = "Run the speculative pipeline: a cheap draft model attempts the task, an independent confidence gate scores it, and a server-side threshold decides verify vs discard. Returns JSON with draft_id, decision, confidence, reasons, draft, timings, error."
    )]
    async fn draft_task(&self, Parameters(p): Parameters<DraftTaskParams>) -> String {
        run_draft_pipeline(&self.cfg, &p.task, p.context.as_deref())
            .await
            .to_string()
    }

    #[tool(
        description = "Record the final outcome of a speculation. outcome must be one of: accepted (draft used verbatim), patched (targeted edits; include patch_ratio), rejected (regenerated from scratch), discarded (gate/error killed the draft)."
    )]
    async fn record_outcome(&self, Parameters(p): Parameters<RecordOutcomeParams>) -> String {
        const VALID: [&str; 4] = ["accepted", "patched", "rejected", "discarded"];
        if !VALID.contains(&p.outcome.as_str()) {
            return json!({"ok": false, "error": format!("invalid outcome '{}'", p.outcome)}).to_string();
        }
        log_event(
            &self.cfg,
            &Event::Outcome {
                id: p.draft_id,
                ts_ms: now_ms(),
                outcome: p.outcome,
                patch_ratio: p.patch_ratio,
            },
        );
        json!({"ok": true}).to_string()
    }

    #[tool(
        description = "Aggregate speculation telemetry: counts per outcome, verify-path acceptance rate, mean patch ratio, latencies, and gate calibration (mean confidence of accepted/patched vs rejected drafts)."
    )]
    async fn spec_stats(&self) -> String {
        match stats(&self.cfg.data_dir.join("events.jsonl")) {
            Ok(s) => serde_json::to_string(&s).unwrap_or_else(|e| json!({"error": e.to_string()}).to_string()),
            Err(e) => json!({"error": e.to_string()}).to_string(),
        }
    }
}

#[tool_handler]
impl ServerHandler for DspecServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        
        info.server_info.name = "dspec".into();
        info.server_info.version = "1.0".into();
        
        info.instructions = Some("Speculative draft-then-verify pipeline".into());
        
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_cfg(mode_dir: &str) -> Config {
        Config {
            draft_model: "haiku".into(),
            gate_model: "haiku".into(),
            threshold: 60,
            timeout_secs: 10,
            data_dir: std::env::temp_dir().join(format!("dspec-srv-{mode_dir}-{}", std::process::id())),
            claude_bin: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/mock_claude.sh")
                .to_string_lossy()
                .into_owned(),
        }
    }

    #[tokio::test]
    async fn json_gate_output_yields_verify() {
        // mock returns the JSON blob for BOTH draft and gate calls; the gate
        // parse succeeds with confidence 85 >= 60 -> verify.
        unsafe {std::env::set_var("MOCK_MODE", "json")};
        let v = run_draft_pipeline(&test_cfg("verify"), "some task", None).await;
        unsafe {std::env::remove_var("MOCK_MODE")};
        assert_eq!(v["decision"], "verify");
        assert_eq!(v["confidence"], 85);
        assert!(v["draft_id"].as_str().unwrap().contains('-'));
        assert!(v["error"].is_null());
    }

    #[tokio::test]
    async fn unparseable_gate_fails_open_to_discard() {
        // mock returns "mock output" -> gate parse fails -> discard with error.
        unsafe {std::env::set_var("MOCK_MODE", "ok")};
        let v = run_draft_pipeline(&test_cfg("discard"), "some task", None).await;
        unsafe {std::env::remove_var("MOCK_MODE")};
        assert_eq!(v["decision"], "discard");
        assert!(v["error"].as_str().unwrap().contains("unparseable"));
    }

    #[tokio::test]
    async fn missing_cli_fails_open_to_discard() {
        let mut cfg = test_cfg("nocli");
        cfg.claude_bin = "/nonexistent/claude-nope".into();
        let v = run_draft_pipeline(&cfg, "some task", None).await;
        assert_eq!(v["decision"], "discard");
        assert!(v["error"].as_str().unwrap().contains("not found"));
    }
}