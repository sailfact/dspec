use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum StreamLine {
    Delta(String),
    Final(String),
    Raw(String),
    Other,
}

pub fn parse_line(line: &str) -> StreamLine {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return StreamLine::Other;
    }
    let Ok(v) = serde_json::from_str::<Value>(trimmed) else {
        return StreamLine::Raw(trimmed.to_string());
    };
    match v.get("type").and_then(Value::as_str) {
        Some("stream_event") => {
            let event_type = v.pointer("/event/type").and_then(Value::as_str);
            let delta_type = v.pointer("/event/delta/type").and_then(Value::as_str);
            let text = v.pointer("/event/delta/text").and_then(Value::as_str);
            match (event_type, delta_type, text) {
                (Some("content_block_delta"), Some("text_delta"), Some(t)) => {
                    StreamLine::Delta(t.to_string())
                }
                _ => StreamLine::Other,
            }
        }
        Some("result") => match v.get("result").and_then(Value::as_str) {
            Some(t) => StreamLine::Final(t.to_string()),
            None => StreamLine::Other,
        },
        Some(_) => StreamLine::Other,
        None => StreamLine::Raw(trimmed.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_delta() {
        let line = r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"hel"}}}"#;
        assert_eq!(parse_line(line), StreamLine::Delta("hel".into()));
    }

    #[test]
    fn parses_result_final() {
        let line = r#"{"type":"result","subtype":"success","result":"hello"}"#;
        assert_eq!(parse_line(line), StreamLine::Final("hello".into()));
    }

    #[test]
    fn known_wrapper_events_are_other() {
        assert_eq!(parse_line(r#"{"type":"system","subtype":"init"}"#), StreamLine::Other);
        let non_text = r#"{"type":"stream_event","event":{"type":"message_start"}}"#;
        assert_eq!(parse_line(non_text), StreamLine::Other);
    }

    #[test]
    fn bare_json_without_type_is_raw() {
        // the mock fixture's `json` mode emits exactly this shape
        let line = r#"{"confidence": 85, "reasons": ["complete"]}"#;
        assert_eq!(parse_line(line), StreamLine::Raw(line.into()));
    }

    #[test]
    fn non_json_is_raw() {
        assert_eq!(parse_line("mock output"), StreamLine::Raw("mock output".into()));
    }

    #[test]
    fn empty_line_is_other() {
        assert_eq!(parse_line("   "), StreamLine::Other);
    }
}