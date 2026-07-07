use crate::config::Config;
use crate::error::DraftError;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn run_claude(cfg: &Config, model: &str, prompt: &str) -> Result<String, DraftError> {
    let mut child = Command::new(&cfg.claude_bin)
        .args(["-p", "--model", model, "--output-format", "text"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true) // dropping the future on timeout kills the process
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DraftError::CliMissing(cfg.claude_bin.clone())
            } else {
                DraftError::Io(e)
            }
        })?;

    {
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin.write_all(prompt.as_bytes()).await?;
        // stdin dropped here -> EOF for the child
    }

    let output = timeout(Duration::from_secs(cfg.timeout_secs), child.wait_with_output())
        .await
        .map_err(|_| DraftError::Timeout(cfg.timeout_secs))??;

    if !output.status.success() {
        return Err(DraftError::CliFailed {
            status: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn cfg_with(bin: &str, timeout_secs: u64) -> Config {
        Config {
            draft_model: "haiku".into(),
            gate_model: "haiku".into(),
            threshold: 60,
            timeout_secs,
            data_dir: std::env::temp_dir().join("dspec-test"),
            claude_bin: bin.into(),
        }
    }

    fn mock_bin() -> String {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mock_claude.sh");
        p.to_string_lossy().into_owned()
    }

    #[tokio::test]
    async fn success_returns_trimmed_stdout() {
        let out = run_claude(&cfg_with(&mock_bin(), 10), "haiku", "hello").await.unwrap();
        assert_eq!(out, "mock output");
    }

    #[tokio::test]
    async fn missing_binary_is_cli_missing() {
        let err = run_claude(&cfg_with("/nonexistent/claude-nope", 10), "haiku", "x")
            .await
            .unwrap_err();
        assert!(matches!(err, DraftError::CliMissing(_)));
    }

    #[tokio::test]
    async fn nonzero_exit_is_cli_failed() {
        unsafe {std::env::set_var("MOCK_MODE", "fail")};
        let err = run_claude(&cfg_with(&mock_bin(), 10), "haiku", "x").await.unwrap_err();
        unsafe {std::env::remove_var("MOCK_MODE")};
        match err {
            DraftError::CliFailed { status, stderr } => {
                assert_eq!(status, 2);
                assert_eq!(stderr, "boom");
            }
            other => panic!("expected CliFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn slow_process_times_out() {
        unsafe {std::env::set_var("MOCK_MODE", "slow")};
        let err = run_claude(&cfg_with(&mock_bin(), 1), "haiku", "x").await.unwrap_err();
        unsafe {std::env::remove_var("MOCK_MODE")};
        assert!(matches!(err, DraftError::Timeout(1)));
    }
}