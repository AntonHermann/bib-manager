//! Backend based on `pdf-extract` (MIT). Returns only text, no geometry,
//! but reads glyph names from embedded Type1 fonts (ε in old LaTeX).

use std::path::Path;

use crate::{Backend, ExtractError, Extraction, Page, PageContent, guard};

/// Must match the exact version in `Cargo.toml` (`=0.12.0`).
const VERSION: &str = "0.12.0";

pub struct PdfExtract;

impl Backend for PdfExtract {
    fn name(&self) -> &'static str {
        "pdf-extract"
    }

    fn version(&self) -> String {
        VERSION.to_string()
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        guard(|| {
            let texts = ::pdf_extract::extract_text_by_pages(path).map_err(|e| ExtractError::Open(e.to_string()))?;
            let pages = texts
                .into_iter()
                .enumerate()
                .map(|(index, text)| Page {
                    index,
                    size: None,
                    content: PageContent::Plain(text),
                })
                .collect();
            Ok(Extraction {
                backend: self.name(),
                backend_version: self.version(),
                pages,
            })
        })
    }
}
