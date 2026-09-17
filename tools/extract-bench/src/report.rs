//! Markdown report: one row per document and backend, followed by a total per backend.

use crate::metrics::TextScores;

#[derive(Debug, Clone)]
pub struct Row {
    pub doc: String,
    pub backend: String,
    pub scores: Option<TextScores>,
    pub pages: usize,
    pub pages_with_geometry: usize,
    pub millis: u128,
    pub error: Option<String>,
}

pub fn render_markdown(rows: &[Row]) -> String {
    let mut md = String::new();
    md.push_str("## Details\n\n");
    md.push_str("| Document | Backend | Sentences | Order | Missing chars | Control chars | (cid:) | U+FFFD | Pages with geometry | ms |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for row in rows {
        match (&row.scores, &row.error) {
            (Some(s), _) => md.push_str(&format!(
                "| {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {}/{} | {} |\n",
                row.doc,
                row.backend,
                s.sentences_found,
                s.sentences_total,
                s.order_ok,
                s.order_total,
                if s.chars_missing.is_empty() {
                    "–".to_string()
                } else {
                    s.chars_missing.join(" ")
                },
                s.control_chars,
                s.cid_markers,
                s.replacement_chars,
                row.pages_with_geometry,
                row.pages,
                row.millis,
            )),
            (None, error) => md.push_str(&format!(
                "| {} | {} | error: {} |\n",
                row.doc,
                row.backend,
                error.as_deref().unwrap_or("unknown")
            )),
        }
    }

    md.push_str("\n## Totals per backend\n\n");
    md.push_str(
        "| Backend | Sentences | Order | Docs with missing chars | Control chars | (cid:) | U+FFFD | Errors |\n",
    );
    md.push_str("|---|---|---|---|---|---|---|---|\n");
    let mut backends: Vec<&str> = Vec::new();
    for row in rows {
        if !backends.contains(&row.backend.as_str()) {
            backends.push(&row.backend);
        }
    }
    for backend in backends {
        let of_backend: Vec<&Row> = rows.iter().filter(|r| r.backend == backend).collect();
        let scored: Vec<&TextScores> = of_backend.iter().filter_map(|r| r.scores.as_ref()).collect();
        let sum = |f: fn(&TextScores) -> usize| scored.iter().map(|s| f(s)).sum::<usize>();
        md.push_str(&format!(
            "| {} | {}/{} | {}/{} | {} | {} | {} | {} | {} |\n",
            backend,
            sum(|s| s.sentences_found),
            sum(|s| s.sentences_total),
            sum(|s| s.order_ok),
            sum(|s| s.order_total),
            scored.iter().filter(|s| !s.chars_missing.is_empty()).count(),
            sum(|s| s.control_chars),
            sum(|s| s.cid_markers),
            sum(|s| s.replacement_chars),
            of_backend.iter().filter(|r| r.error.is_some()).count(),
        ));
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores(found: usize, total: usize, order: usize) -> TextScores {
        TextScores {
            sentences_found: found,
            sentences_total: total,
            order_ok: order,
            order_total: 1,
            chars_missing: vec!["ε".into()],
            control_chars: 2,
            cid_markers: 0,
            replacement_chars: 0,
        }
    }

    fn row(doc: &str, backend: &str, scores: Option<TextScores>, error: Option<&str>) -> Row {
        Row {
            doc: doc.into(),
            backend: backend.into(),
            scores,
            pages: 6,
            pages_with_geometry: 6,
            millis: 120,
            error: error.map(Into::into),
        }
    }

    #[test]
    fn renders_detail_rows_and_backend_totals() {
        let md = render_markdown(&[
            row("a", "pdf_oxide", Some(scores(2, 3, 1)), None),
            row("b", "pdf_oxide", Some(scores(3, 3, 0)), None),
            row("a", "mutool", None, Some("not installed")),
        ]);
        assert!(
            md.contains("| a | pdf_oxide | 2/3 | 1/1 | ε | 2 | 0 | 0 | 6/6 | 120 |"),
            "{md}"
        );
        assert!(md.contains("| a | mutool | error: not installed |"), "{md}");
        assert!(md.contains("| pdf_oxide | 5/6 | 1/2 | 2 | 4 | 0 | 0 | 0 |"), "{md}");
    }
}
