use crate::error::AppError;
use crate::models::{OcrCandidate, TextMode};

pub fn cleanup_text(input: &str, mode: TextMode) -> String {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = normalized.lines().map(|line| line.trim_end().to_owned()).collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    let output = lines.join("\n");
    if mode == TextMode::SingleLine {
        output.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        output
    }
}

pub fn select_best_candidate(mut candidates: Vec<OcrCandidate>) -> Result<OcrCandidate, AppError> {
    candidates.retain(|candidate| !candidate.text.trim().is_empty());
    candidates.into_iter().max_by_key(|candidate| candidate.score).ok_or(AppError::OcrEmptyResult)
}

pub fn score_text(text: &str) -> i64 {
    if text.trim().is_empty() {
        return i64::MIN / 2;
    }
    let replacement_penalty = text.chars().filter(|character| *character == '\u{fffd}' || character.is_control() && *character != '\n' && *character != '\t').count() as i64 * 100;
    let punctuation = text.chars().filter(|character| "{}[]()<>|/_-:;.'\"`=+*&$#@!?,\\".contains(*character)).count() as i64;
    let line_bonus = text.lines().count().min(50) as i64 * 2;
    text.chars().count() as i64 + punctuation + line_bonus - replacement_penalty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_cleanup_preserves_indentation_blank_lines_and_punctuation() {
        let input = "  cargo test --locked\r\n\r\n  let value = foo::<Bar>();  \r\n";
        assert_eq!(cleanup_text(input, TextMode::Terminal), "  cargo test --locked\n\n  let value = foo::<Bar>();");
    }

    #[test]
    fn cleanup_is_idempotent() {
        let once = cleanup_text("a  \n\n", TextMode::Terminal);
        assert_eq!(cleanup_text(&once, TextMode::Terminal), once);
    }
}
