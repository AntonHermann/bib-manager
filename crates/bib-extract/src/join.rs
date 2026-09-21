//! Join spans into text in reading order.

use crate::Span;

pub fn join_spans(spans: &[Span]) -> String {
    let mut out = String::new();
    for (i, span) in spans.iter().enumerate() {
        if i > 0 {
            out.push_str(separator(&spans[i - 1], span));
        }
        out.push_str(&span.text);
    }
    out
}

fn separator(prev: &Span, next: &Span) -> &'static str {
    let prev_height = prev.bbox.bottom - prev.bbox.top;
    let next_height = next.bbox.bottom - next.bbox.top;
    let overlap = prev.bbox.bottom.min(next.bbox.bottom) - prev.bbox.top.max(next.bbox.top);
    let same_line = overlap > 0.5 * prev_height.min(next_height);
    let gap = next.bbox.left - prev.bbox.right;
    if !same_line || gap < -prev_height.max(next_height) {
        "\n"
    } else if gap < 0.15 * prev_height.max(next_height) {
        ""
    } else {
        " "
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    fn span(text: &str, left: f32, top: f32, right: f32, bottom: f32) -> Span {
        Span {
            text: text.into(),
            bbox: Rect {
                left,
                top,
                right,
                bottom,
            },
            font: "F".into(),
        }
    }

    #[test]
    fn adjacent_spans_on_one_line_are_glued() {
        let spans = [
            span("differ", 0.0, 100.0, 30.0, 110.0),
            span("ential", 30.5, 100.0, 60.0, 110.0),
        ];
        assert_eq!(join_spans(&spans), "differential");
    }

    #[test]
    fn spans_with_a_gap_on_one_line_get_a_space() {
        let spans = [
            span("privacy", 0.0, 100.0, 30.0, 110.0),
            span("loss", 34.0, 100.0, 60.0, 110.0),
        ];
        assert_eq!(join_spans(&spans), "privacy loss");
    }

    #[test]
    fn next_line_gets_a_newline() {
        let spans = [
            span("end of line", 0.0, 100.0, 60.0, 110.0),
            span("next", 0.0, 112.0, 20.0, 122.0),
        ];
        assert_eq!(join_spans(&spans), "end of line\nnext");
    }

    #[test]
    fn jump_back_on_same_height_is_a_newline() {
        let spans = [
            span("right column", 300.0, 100.0, 400.0, 110.0),
            span("left", 0.0, 100.0, 20.0, 110.0),
        ];
        assert_eq!(join_spans(&spans), "right column\nleft");
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(join_spans(&[]), "");
    }
}
