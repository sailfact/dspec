use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub draft_model: String,
    pub gate_model: String,
    pub threshold: u8,
    pub timeout_secs: u64,
    pub data_dir: PathBuf,
    pub claude_bin: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    
    #[test]
    fn defaults_when_nothing_set() {
        let cfg = Config::from_lookup(|_| None);
        assert_eq!(cfg.draft_model, "haiku");
        assert_eq!(cfg.gate_model, "haiku");
        assert_eq!(cfg.threshold, 60);
        assert_eq!(cfg.timeout_secs, 120);
        assert_eq!(cfg.claude_bin, "claude");
        assert!(cfg.data_dir.ends_with(".dspec"));
    }

    #[test]
    fn overrides_apply_and_bad_numbers_fall_back() {
        let cfg = Config::from_lookup(|k| match k {
            "DSPEC_DRAFT_MODEL" => Some("sonnet".into()),
            "DSPEC_THRESHOLD" => Some("not-a-number".into()),
            "DSPEC_TIMEOUT_SECS" => Some("30".into()),
            "DSPEC_DATA_DIR" => Some("/tmp/dspec-test".into()),
            _ => None,
        });
        assert_eq!(cfg.draft_model, "sonnet");
        assert_eq!(cfg.threshold, 60); // unparseable -> default
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.data_dir, PathBuf::from("/tmp/dspec-test"));
    }
}

impl Config {
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            draft_model: get("DSPEC_DRAFT_MODEL").unwrap_or_else(|| "haiku".into()),
            gate_model: get("DSPEC_GATE_MODEL").unwrap_or_else(|| "haiku".into()),
            threshold: get("DSPEC_THRESHOLD")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            timeout_secs: get("DSPEC_TIMEOUT_SECS")
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            data_dir: get("DSPEC_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".dspec")),
            claude_bin: get("DSPEC_CLAUDE_BIN").unwrap_or_else(|| "claude".into()),
        }
    }

    pub fn from_env() -> Self {
        Self::from_lookup(|k| std::env::var(k).ok())
    }
}