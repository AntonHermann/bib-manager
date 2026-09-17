//! Backend auf Basis von `pdf_oxide` (MIT/Apache-2.0, reines Rust).
//!
//! `pdf_oxide` liefert Boxen in PDF-Koordinaten (Ursprung unten links, `y` ist die
//! Unterkante). Umrechnung: top = ury - (y + height), bottom = ury - y.

use std::path::Path;

use crate::{Backend, ExtractError, Extraction, Page, PageContent, PageSize, Rect, Span, guard, join_spans};

pub struct PdfOxide;

impl Backend for PdfOxide {
    fn name(&self) -> &'static str {
        "pdf_oxide"
    }

    fn version(&self) -> String {
        pdf_oxide::VERSION.to_string()
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        guard(|| {
            let doc = pdf_oxide::PdfDocument::open(path).map_err(|e| ExtractError::Open(e.to_string()))?;
            let count = doc.page_count().map_err(|e| ExtractError::Open(e.to_string()))?;
            let mut pages = Vec::with_capacity(count);
            for index in 0..count {
                pages.push(extract_page(&doc, index)?);
            }
            Ok(Extraction { backend: self.name(), backend_version: self.version(), pages })
        })
    }
}

fn extract_page(doc: &pdf_oxide::PdfDocument, index: usize) -> Result<Page, ExtractError> {
    let page_error = |e: pdf_oxide::Error| ExtractError::Page { index, message: e.to_string() };
    let raw_spans = doc.extract_spans(index).map_err(page_error)?;
    let raw_spans: Vec<_> = raw_spans.into_iter().filter(|s| !s.text.trim().is_empty()).collect();

    // pdf_oxide löst geerbte Attribute selbst auf (get_page_media_box sieht auch eine von einem
    // Vorfahrenknoten geerbte MediaBox); dieser Zweig greift nur, wenn die MediaBox ganz fehlt
    // oder fehlerhaft ist. Dann nur Text, ehrlich ohne Geometrie.
    let Ok((llx, lly, urx, ury)) = doc.get_page_media_box(index) else {
        let spans: Vec<Span> = raw_spans
            .into_iter()
            .map(|s| Span {
                text: s.text,
                bbox: Rect { left: s.bbox.x, top: -s.bbox.y - s.bbox.height, right: s.bbox.x + s.bbox.width, bottom: -s.bbox.y },
                font: s.font_name,
            })
            .collect();
        return Ok(Page { index, size: None, content: PageContent::Plain(join_spans(&spans)) });
    };

    let rotation = doc.get_page_rotation(index).map_err(page_error)?.rem_euclid(360) as u16;
    let spans = raw_spans
        .into_iter()
        .map(|s| Span {
            bbox: Rect {
                left: s.bbox.x - llx,
                top: ury - (s.bbox.y + s.bbox.height),
                right: s.bbox.x + s.bbox.width - llx,
                bottom: ury - s.bbox.y,
            },
            text: s.text,
            font: s.font_name,
        })
        .collect();
    Ok(Page {
        index,
        size: Some(PageSize { width: urx - llx, height: ury - lly, rotation: Some(rotation) }),
        content: PageContent::Spans(spans),
    })
}
