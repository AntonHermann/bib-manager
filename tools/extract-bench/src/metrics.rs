//! Evaluation of an extracted text against the curated assertions.

use bib_core::normalize::normalize;

use crate::corpus::Expect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextScores {
    pub sentences_found: usize,
    pub sentences_total: usize,
    pub order_ok: usize,
    pub order_total: usize,
    pub chars_missing: Vec<String>,
    /// Control characters other than \n, \r, \t, and form feed.
    pub control_chars: usize,
    /// Occurrences of `(cid:`.
    pub cid_markers: usize,
    /// Occurrences of U+FFFD.
    pub replacement_chars: usize,
}

pub fn evaluate(expect: &Expect, raw: &str) -> TextScores {
    let text = normalize(raw).text;
    let position = |needle: &str| text.find(&normalize(needle).text);
    let sentences_found = expect.sentences.iter().filter(|s| position(s).is_some()).count();
    let order_ok = expect
        .before
        .iter()
        .filter(|(first, second)| matches!((position(first), position(second)), (Some(a), Some(b)) if a < b))
        .count();
    let chars_missing = expect
        .chars
        .iter()
        .filter(|c| !text.contains(&normalize(c).text))
        .cloned()
        .collect();
    TextScores {
        sentences_found,
        sentences_total: expect.sentences.len(),
        order_ok,
        order_total: expect.before.len(),
        chars_missing,
        control_chars: raw
            .chars()
            .filter(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t' | '\u{c}'))
            .count(),
        cid_markers: raw.matches("(cid:").count(),
        replacement_chars: raw.matches('\u{FFFD}').count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect() -> Expect {
        Expect {
            sentences: vec![
                "Deeper networks are harder to train.".into(),
                "Not in the text at all.".into(),
            ],
            before: vec![("deeper networks".into(), "residual learning".into())],
            chars: vec!["ε".into(), "δ".into()],
        }
    }

    #[test]
    fn counts_sentences_after_normalization() {
        let raw = "Deeper net-\nworks are harder to train. We use residual learning with ϵ.";
        let scores = evaluate(&expect(), raw);
        assert_eq!((scores.sentences_found, scores.sentences_total), (1, 2));
    }

    #[test]
    fn checks_reading_order() {
        let in_order = evaluate(&expect(), "Deeper networks first, then residual learning.");
        assert_eq!((in_order.order_ok, in_order.order_total), (1, 1));
        let reversed = evaluate(&expect(), "Residual learning first, then deeper networks.");
        assert_eq!(reversed.order_ok, 0);
        let missing = evaluate(&expect(), "Only deeper networks here.");
        assert_eq!(missing.order_ok, 0);
    }

    #[test]
    fn reports_missing_characters() {
        let scores = evaluate(&expect(), "privacy with ϵ only");
        assert_eq!(scores.chars_missing, vec!["δ".to_string()]);
    }

    #[test]
    fn counts_garbage() {
        let scores = evaluate(&expect(), "a\u{f}b (cid:15) c\u{FFFD}\n\t\u{c}");
        assert_eq!(scores.control_chars, 1);
        assert_eq!(scores.cid_markers, 1);
        assert_eq!(scores.replacement_chars, 1);
    }
}
