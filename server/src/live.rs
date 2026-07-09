use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Best-effort append-only mirror of a model subprocess's output.
/// Never fails: any I/O error degrades to a no-op (dashboard may
/// degrade, the pipeline may not).
pub struct LiveMirror {
    file: Option<File>,
}

impl LiveMirror {
    pub fn noop() -> Self {
        Self { file: None }
    }

    pub fn begin(data_dir: &Path, stream: &str, id: &str, model: &str) -> Self {
        let dir = data_dir.join("live");
        if let Err(e) = fs::create_dir_all(&dir) {
            eprintln!("dspec: live dir create failed: {e}");
            return Self::noop();
        }
        let path = dir.join(format!("{stream}.log"));
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                let _ = writeln!(f, "\n── {stream} {id} model={model} ──");
                Self { file: Some(f) }
            }
            Err(e) => {
                eprintln!("dspec: live mirror open failed: {e}");
                Self::noop()
            }
        }
    }

    pub fn write(&mut self, text: &str) {
        if let Some(f) = &mut self.file {
            let _ = f.write_all(text.as_bytes());
        }
    }

    pub fn finish(&mut self, status: &str) {
        if let Some(f) = &mut self.file {
            let _ = writeln!(f, "\n── {status} ──");
        }
        self.file = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("dspec-live-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn begin_write_finish_roundtrip() {
        let dir = tmpdir("roundtrip");
        let mut m = LiveMirror::begin(&dir, "draft", "abc-123", "haiku");
        m.write("hello ");
        m.write("world");
        m.finish("done in 42ms");
        let content = std::fs::read_to_string(dir.join("live/draft.log")).unwrap();
        assert!(content.contains("── draft abc-123 model=haiku ──"));
        assert!(content.contains("hello world"));
        assert!(content.contains("── done in 42ms ──"));
    }

    #[test]
    fn two_begins_append_to_same_file() {
        let dir = tmpdir("append");
        let mut a = LiveMirror::begin(&dir, "gate", "id-1", "haiku");
        a.finish("done in 1ms");
        let mut b = LiveMirror::begin(&dir, "gate", "id-2", "haiku");
        b.finish("done in 2ms");
        let content = std::fs::read_to_string(dir.join("live/gate.log")).unwrap();
        assert!(content.contains("id-1"));
        assert!(content.contains("id-2"));
    }

    #[test]
    fn writes_after_finish_are_ignored() {
        let dir = tmpdir("after");
        let mut m = LiveMirror::begin(&dir, "gate", "id-1", "haiku");
        m.finish("done in 1ms");
        m.write("late");
        let content = std::fs::read_to_string(dir.join("live/gate.log")).unwrap();
        assert!(!content.contains("late"));
    }

    #[test]
    fn unwritable_data_dir_degrades_to_noop() {
        let dir = tmpdir("blocked");
        std::fs::create_dir_all(&dir).unwrap();
        let file_as_dir = dir.join("blocker");
        std::fs::write(&file_as_dir, "x").unwrap();
        // data_dir is a *file*, so create_dir_all("<file>/live") must fail
        let mut m = LiveMirror::begin(&file_as_dir, "draft", "id", "haiku");
        m.write("x");
        m.finish("done"); // must not panic
    }

    #[test]
    fn noop_never_panics() {
        let mut m = LiveMirror::noop();
        m.write("x");
        m.finish("done");
    }
}