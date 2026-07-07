use crate::error::DraftError;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct GateScore {
    pub confidence: u8,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Verify,
    Discard,
}

pub fn parse_gate_output(raw: &str) -> Result<GateScore, DraftError> {
    let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) else {
        return Err(DraftError::GateUnparseable(snippet(raw)));
    };
    if end < start {
        return Err(DraftError::GateUnparseable(snippet(raw)));
    }
    let score: GateScore = serde_json::from_str(&raw[start..=end])
        .map_err(|e| DraftError::GateUnparseable(format!("{e}: {}", snippet(raw))))?;
    if score.confidence > 100 {
        return Err(DraftError::GateUnparseable(format!(
            "confidence {} out of range 0-100",
            score.confidence
        )));
    }
    Ok(score)
}

pub fn decide(confidence: u8, threshold: u8) -> Decision {
    if confidence >= threshold {
        Decision::Verify
    } else {
        Decision::Discard
    }
}

fn snippet(raw: &str) -> String {
    raw.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_json() {
        let s = parse_gate_output(r#"{"confidence": 85, "reasons": ["complete", "mechanical"]}"#).unwrap();
        assert_eq!(s.confidence, 85);
        assert_eq!(s.reasons.len(), 2);
    }

    #[test]
    fn parses_json_wrapped_in_prose() {
        let raw = "Sure! Here is my assessment:\n{\"confidence\": 40, \"reasons\": [\"unverified claims\"]}\nHope that helps.";
        assert_eq!(parse_gate_output(raw).unwrap().confidence, 40);
    }

    #[test]
    fn missing_reasons_defaults_empty() {
        let s = parse_gate_output(r#"{"confidence": 10}"#).unwrap();
        assert!(s.reasons.is_empty());
    }

    #[test]
    fn garbage_is_unparseable() {
        assert!(matches!(parse_gate_output("mock output"), Err(DraftError::GateUnparseable(_))));
    }

    #[test]
    fn out_of_range_confidence_rejected() {
        assert!(matches!(parse_gate_output(r#"{"confidence": 150}"#), Err(DraftError::GateUnparseable(_))));
    }

    #[test]
    fn threshold_is_inclusive() {
        assert_eq!(decide(60, 60), Decision::Verify);
        assert_eq!(decide(59, 60), Decision::Discard);
    }
}