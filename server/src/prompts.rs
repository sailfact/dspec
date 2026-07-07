pub fn drafter_prompt(task: &str, context: Option<&str>) -> String {
    let context_block = match context {
        Some(c) if !c.trim().is_empty() => format!("\n## Context\n{c}\n"),
        _ => String::new(),
    };
    format!(
        "You are the DRAFT MODEL in a speculative decoding pipeline. A stronger model will \
verify your output and accept it verbatim if it is correct, so produce your best complete \
attempt at the task.\n\
Rules:\n\
- Output ONLY the deliverable. No preamble, no explanation, no questions, no markdown fences \
around the whole answer unless the deliverable is code.\n\
- Do not describe what you would do. Do it.\n\
- If the task cannot be attempted from the information given, output exactly: DRAFT_ABSTAIN\n\
{context_block}\n## Task\n{task}\n"
    )
}

pub fn gate_prompt(task: &str, draft: &str) -> String {
    format!(
        "You are a CONFIDENCE GATE in a speculative decoding pipeline. Estimate the probability \
(0-100) that a strong senior model, given the same task, would accept this draft with at most \
minor edits.\n\
Scoring rubric:\n\
- 80-100: complete, mechanical/boilerplate work with nothing to get wrong (config files, \
docstrings, commit messages, format conversions) that fully satisfies the task.\n\
- 50-79: plausible and complete, but contains claims or design choices you cannot verify.\n\
- 20-49: incomplete, partially wrong, or made significant unstated assumptions.\n\
- 0-19: wrong approach, missing requirements, hallucinated APIs, or the draft is DRAFT_ABSTAIN.\n\
You are grading someone else's work. Be skeptical; an overconfident gate wastes expensive \
verification.\n\
Respond with ONLY this JSON object and nothing else:\n\
{{\"confidence\": <0-100>, \"reasons\": [\"<short reason>\", ...]}}\n\n\
## Task\n{task}\n\n## Draft\n{draft}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drafter_includes_task_and_optional_context() {
        let p = drafter_prompt("write a .gitignore for Rust", Some("repo uses cargo workspaces"));
        assert!(p.contains("write a .gitignore for Rust"));
        assert!(p.contains("cargo workspaces"));
        let p2 = drafter_prompt("task only", None);
        assert!(p2.contains("task only"));
        assert!(!p2.contains("## Context"));
    }

    #[test]
    fn gate_includes_task_draft_and_json_contract() {
        let p = gate_prompt("the task", "the draft body");
        assert!(p.contains("the task"));
        assert!(p.contains("the draft body"));
        assert!(p.contains("\"confidence\""));
    }
}