use crate::config::Config;
use crate::error::DraftError;
use crate::live::LiveMirror;
use crate::stream::{parse_line, StreamLine};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn run_claude(cfg: &Config, model: &str, prompt: &str, mut mirror: LiveMirror) -> Result<String, DraftError> {
    let started = Instant::now();
    let result = run_streaming(cfg, model, prompt, &mut mirror).await;
    match &result {
        Ok(_) => mirror.finish(&format!("done in {}ms", started.elapsed().as_millis())),
        Err(e) => mirror.finish(&format!("error: {e}")),
    }
    result
}

async fn run_streaming( cfg: &Config, model: &str, prompt: &str, mirror: &mut LiveMirror) -> Result<String, DraftError> {
    let mut child = Command::new(&cfg.claude_bin)
        .args([
            "-p",
            "--model",
            model,
            "--output-format",
            "stream-json",
            "--include-partial-messages",
            "--verbose",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true) // dropping on timeout kills the process
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

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    // Drain stderr concurrently so a chatty child can't deadlock on a
    // full pipe while we read stdout.
    let stderr_task = tokio::spawn(async move {
        let mut buf = String::new();
        let mut stderr = stderr;
        let _ = stderr.read_to_string(&mut buf).await;
        buf
    });

    let work = async {
        let mut lines = BufReader::new(stdout).lines();
        let mut deltas = String::new();
        let mut raws = String::new();
        let mut final_text: Option<String> = None;

        while let Some(line) = lines.next_line().await? {
            match parse_line(&line) {
                StreamLine::Delta(t) => {
                    mirror.write(&t);
                    deltas.push_str(&t);
                }
                StreamLine::Final(t) => final_text = Some(t),
                StreamLine::Raw(t) => {
                    mirror.write(&t);
                    mirror.write("\n");
                    raws.push_str(&t);
                    raws.push('\n');
                }
                StreamLine::Other => {}
            }
        }

        let status = child.wait().await?;
        Ok::<_, DraftError>((status, final_text, deltas, raws))
    };

    let (status, final_text, deltas, raws) =
        timeout(Duration::from_secs(cfg.timeout_secs), work)
            .await
            .map_err(|_| DraftError::Timeout(cfg.timeout_secs))??;

    let stderr_buf = stderr_task.await.unwrap_or_default();

    if !status.success() {
        return Err(DraftError::CliFailed {
            status: status.code().unwrap_or(-1),
            stderr: stderr_buf.trim().to_string(),
        });
    }

    let text = final_text.unwrap_or_else(|| if deltas.is_empty() { raws } else { deltas });
    Ok(text.trim().to_string())
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
    async fn plain_text_output_falls_back_to_raw() {
        // MOCK_MODE default `ok` emits plain text, not stream-json
        let out = run_claude(&cfg_with(&mock_bin(), 10), "haiku", "hello", LiveMirror::noop())
            .await
            .unwrap();
        assert_eq!(out, "mock output");
    }

    #[tokio::test]
    async fn stream_mode_mirrors_deltas_and_returns_final() {
        unsafe {std::env::set_var("MOCK_MODE", "stream")};
        let data_dir = std::env::temp_dir().join(format!("dspec-cli-stream-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data_dir);
        let mirror = LiveMirror::begin(&data_dir, "draft", "t-1", "haiku");
        let out = run_claude(&cfg_with(&mock_bin(), 10), "haiku", "x", mirror).await.unwrap();
        unsafe {std::env::remove_var("MOCK_MODE")};
        assert_eq!(out, "hello");
        let log = std::fs::read_to_string(data_dir.join("live/draft.log")).unwrap();
        assert!(log.contains("── draft t-1 model=haiku ──"));
        assert!(log.contains("hello"));
        assert!(log.contains("── done in"));
    }

    #[tokio::test]
    async fn missing_binary_is_cli_missing() {
        let err = run_claude(&cfg_with("/nonexistent/claude-nope", 10), "haiku", "x", LiveMirror::noop())
            .await
            .unwrap_err();
        assert!(matches!(err, DraftError::CliMissing(_)));
    }

    #[tokio::test]
    async fn nonzero_exit_is_cli_failed() {
        unsafe {std::env::set_var("MOCK_MODE", "fail")};
        let err = run_claude(&cfg_with(&mock_bin(), 10), "haiku", "x", LiveMirror::noop())
            .await
            .unwrap_err();
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
        let err = run_claude(&cfg_with(&mock_bin(), 1), "haiku", "x", LiveMirror::noop())
            .await
            .unwrap_err();
        unsafe {std::env::remove_var("MOCK_MODE")};
        assert!(matches!(err, DraftError::Timeout(1)));
    }
}