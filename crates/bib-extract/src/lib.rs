//! PDF text extraction behind a swappable interface (Spec §7).

pub mod backends;
mod join;

use std::path::Path;

pub use join::join_spans;

/// Rectangle in points, **origin top-left, y grows downward**, relative to the MediaBox.
/// Every backend converts into this system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Contiguous piece of text with a box and font, in reading order.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub bbox: Rect,
    pub font: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
    /// `None` if the backend does not know the rotation.
    pub rotation: Option<u16>,
}

/// Text is mandatory, geometry is optional (Spec §7).
#[derive(Debug, Clone, PartialEq)]
pub enum PageContent {
    Spans(Vec<Span>),
    Plain(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub index: usize,
    pub size: Option<PageSize>,
    pub content: PageContent,
}

impl Page {
    pub fn text(&self) -> String {
        match &self.content {
            PageContent::Spans(spans) => join_spans(spans),
            PageContent::Plain(text) => text.clone(),
        }
    }

    pub fn has_geometry(&self) -> bool {
        matches!(self.content, PageContent::Spans(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extraction {
    pub backend: &'static str,
    pub backend_version: String,
    pub pages: Vec<Page>,
}

impl Extraction {
    /// Text of all pages, separated by line breaks.
    pub fn text(&self) -> String {
        self.pages.iter().map(Page::text).collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("could not open PDF: {0}")]
    Open(String),
    #[error("page {index}: {message}")]
    Page { index: usize, message: String },
    #[error("backend panicked: {0}")]
    Panicked(String),
    #[error("tool not available: {0}")]
    Unavailable(String),
}

pub trait Backend {
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError>;
}

/// Catches panics from a backend (`pdf-extract` aborts hard on corrupt files).
pub(crate) fn guard<T>(f: impl FnOnce() -> Result<T, ExtractError>) -> Result<T, ExtractError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown cause".to_string());
            Err(ExtractError::Panicked(message))
        }
    }
}
