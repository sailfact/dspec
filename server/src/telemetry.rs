use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// A single telemetry record appended to the JSONL event log.
///
/// One `Draft` event is recorded per pipeline run (draft + gate), and one
/// `Outcome` event is recorded when the caller later reports what happened
/// to that draft. The two are joined by `id` when computing [`Stats`].
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    /// Recorded once per `draft_task` pipeline run, regardless of outcome.
    Draft {
        /// Unique identifier for this pipeline run, used to join with a later `Outcome` event.
        id: String,
        /// Unix timestamp in milliseconds when the event was recorded.
        ts_ms: u64,
        /// Gate model's confidence score (0-100), or `None` if the gate call failed or its output was unparseable.
        confidence: Option<u8>,
        /// The pipeline's resulting decision: `"verify"` or `"discard"`.
        decision: String,
        /// Wall-clock time spent in the draft model call, in milliseconds.
        draft_ms: u64,
        /// Wall-clock time spent in the gate model call, in milliseconds.
        gate_ms: u64,
        /// Error message if any stage of the pipeline failed, otherwise `None`.
        error: Option<String>,
    },
    /// Recorded when the caller reports the final result of a previously drafted task.
    Outcome {
        /// Identifier matching the `id` of the corresponding `Draft` event.
        id: String,
        /// Unix timestamp in milliseconds when the event was recorded.
        ts_ms: u64,
        /// One of the fixed strings "accepted" | "patched" | "rejected" | "discarded" (validated by the caller).
        outcome: String,
        /// Fraction of the draft that was changed during verification, if applicable (e.g. for "patched" outcomes).
        patch_ratio: Option<f32>,
    },
}

/// Appends `event` as a single JSON-serialized line to the file at `path`.
///
/// Creates the file's parent directories if they don't already exist. This
/// function only ever appends; it does not read or validate existing file
/// contents, so pre-existing corrupt lines in the file do not cause it to fail.
pub fn append(path: &Path, event: &Event) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{}", serde_json::to_string(event).expect("event serializes"))
}

/// Aggregate telemetry report computed from the joined `Draft`/`Outcome` event log.
///
/// Returned by `spec_stats`. Used to judge whether the gate's confidence
/// scores are well-calibrated: per project convention, `mean_confidence_good`
/// should exceed `mean_confidence_bad` by at least 10 points.
#[derive(Debug, Serialize, Default, PartialEq)]
pub struct Stats {
    /// Total number of `Draft` events seen.
    pub drafts: u64,
    /// Number of drafts discarded (gate confidence below threshold, or a pipeline error).
    pub discarded: u64,
    /// Number of drafts accepted verbatim by the verifying model.
    pub accepted: u64,
    /// Number of drafts accepted after being patched by the verifying model.
    pub patched: u64,
    /// Number of drafts rejected by the verifying model.
    pub rejected: u64,
    /// (accepted + patched) / (accepted + patched + rejected); `None` if that denominator is zero.
    pub verify_acceptance_rate: Option<f32>,
    /// Mean `patch_ratio` across `Outcome` events that reported one; `None` if none did.
    pub mean_patch_ratio: Option<f32>,
    /// Mean `draft_ms` across all `Draft` events; `None` if there are none.
    pub mean_draft_ms: Option<f32>,
    /// Mean `gate_ms` across all `Draft` events; `None` if there are none.
    pub mean_gate_ms: Option<f32>,
    /// Mean gate confidence of drafts whose outcome was accepted or patched; `None` if there are none.
    pub mean_confidence_good: Option<f32>,
    /// Mean gate confidence of drafts whose outcome was rejected; `None` if there are none.
    pub mean_confidence_bad: Option<f32>,
}

fn mean(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f32>() / values.len() as f32)
    }
}

/// Reads the JSONL event log at `path` and computes the aggregate [`Stats`].
///
/// Joins `Draft` and `Outcome` events by `id`. If `path` does not exist,
/// returns `Ok` with zeroed `Stats` rather than an error. Lines that fail to
/// parse as an `Event` are skipped silently.
pub fn stats(path: &Path) -> std::io::Result<Stats> {
    let mut s = Stats::default();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(s),
        Err(e) => return Err(e),
    };

    let mut confidence_by_id: HashMap<String, Option<u8>> = HashMap::new();
    let mut draft_ms = Vec::new();
    let mut gate_ms = Vec::new();
    let mut outcomes: Vec<(String, String, Option<f32>)> = Vec::new();

    for line in content.lines() {
        match serde_json::from_str::<Event>(line) {
            Ok(Event::Draft { id, confidence, draft_ms: d, gate_ms: g, .. }) => {
                s.drafts += 1;
                confidence_by_id.insert(id, confidence);
                draft_ms.push(d as f32);
                gate_ms.push(g as f32);
            }
            Ok(Event::Outcome { id, outcome, patch_ratio, .. }) => {
                outcomes.push((id, outcome, patch_ratio));
            }
            Err(_) => continue, // corrupt line: skip, never fail stats
        }
    }

    let mut good_conf = Vec::new();
    let mut bad_conf = Vec::new();
    let mut patch_ratios = Vec::new();

    for (id, outcome, patch_ratio) in outcomes {
        let conf = confidence_by_id.get(&id).copied().flatten();
        match outcome.as_str() {
            "accepted" => {
                s.accepted += 1;
                if let Some(c) = conf { good_conf.push(c as f32); }
            }
            "patched" => {
                s.patched += 1;
                if let Some(c) = conf { good_conf.push(c as f32); }
                if let Some(r) = patch_ratio { patch_ratios.push(r); }
            }
            "rejected" => {
                s.rejected += 1;
                if let Some(c) = conf { bad_conf.push(c as f32); }
            }
            "discarded" => s.discarded += 1,
            _ => {}
        }
    }

    let verified = s.accepted + s.patched + s.rejected;
    if verified > 0 {
        s.verify_acceptance_rate = Some((s.accepted + s.patched) as f32 / verified as f32);
    }
    s.mean_patch_ratio = mean(&patch_ratios);
    s.mean_draft_ms = mean(&draft_ms);
    s.mean_gate_ms = mean(&gate_ms);
    s.mean_confidence_good = mean(&good_conf);
    s.mean_confidence_bad = mean(&bad_conf);
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmpfile(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("dspec-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn draft(id: &str, conf: Option<u8>, decision: &str) -> Event {
        Event::Draft {
            id: id.into(),
            ts_ms: 1,
            confidence: conf,
            decision: decision.into(),
            draft_ms: 100,
            gate_ms: 50,
            error: None,
        }
    }

    fn outcome(id: &str, outcome: &str, patch_ratio: Option<f32>) -> Event {
        Event::Outcome { id: id.into(), ts_ms: 2, outcome: outcome.into(), patch_ratio }
    }

    #[test]
    fn append_then_stats_roundtrip() {
        let p = tmpfile("roundtrip");
        append(&p, &draft("a", Some(90), "verify")).unwrap();
        append(&p, &outcome("a", "accepted", None)).unwrap();
        append(&p, &draft("b", Some(70), "verify")).unwrap();
        append(&p, &outcome("b", "patched", Some(0.2))).unwrap();
        append(&p, &draft("c", Some(65), "verify")).unwrap();
        append(&p, &outcome("c", "rejected", None)).unwrap();
        append(&p, &draft("d", Some(10), "discard")).unwrap();
        append(&p, &outcome("d", "discarded", None)).unwrap();

        let s = stats(&p).unwrap();
        assert_eq!(s.drafts, 4);
        assert_eq!(s.accepted, 1);
        assert_eq!(s.patched, 1);
        assert_eq!(s.rejected, 1);
        assert_eq!(s.discarded, 1);
        // (accepted + patched) / (accepted + patched + rejected) = 2/3
        assert!((s.verify_acceptance_rate.unwrap() - 2.0 / 3.0).abs() < 1e-6);
        assert!((s.mean_patch_ratio.unwrap() - 0.2).abs() < 1e-6);
        // good = accepted+patched confidences (90, 70) -> 80; bad = rejected (65)
        assert!((s.mean_confidence_good.unwrap() - 80.0).abs() < 1e-6);
        assert!((s.mean_confidence_bad.unwrap() - 65.0).abs() < 1e-6);
        assert!((s.mean_draft_ms.unwrap() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn empty_or_missing_file_gives_zeroed_stats() {
        let p = tmpfile("empty");
        let s = stats(&p).unwrap();
        assert_eq!(s.drafts, 0);
        assert!(s.verify_acceptance_rate.is_none());
    }

    #[test]
    fn corrupt_lines_are_skipped() {
        let p = tmpfile("corrupt");
        append(&p, &draft("a", Some(90), "verify")).unwrap();
        std::fs::write(&p, format!("{}\nnot json\n", std::fs::read_to_string(&p).unwrap())).unwrap();
        assert_eq!(stats(&p).unwrap().drafts, 1);
    }
}