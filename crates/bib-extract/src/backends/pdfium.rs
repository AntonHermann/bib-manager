//! Backend based on pdfium (Chrome's PDF engine) via `pdfium-render`.
//! Needs `libpdfium.so` at runtime; path via `BIB_PDFIUM_LIB_DIR`.
//!
//! Coordinate assumption: pdfium returns page size and coordinates relative to the CropBox
//! (not the MediaBox) and already rotation-corrected (width/height swapped at 90°/270°).
//! The benchmark corpus only exercises the case of a MediaBox origin at (0,0) without its own
//! CropBox and without `/Rotate`; a differing CropBox or an offset MediaBox would silently lead
//! to wrong coordinates here (see Spec §18).

use std::path::{Path, PathBuf};

use pdfium_render::prelude::{PdfPageRenderRotation, Pdfium as PdfiumLib, PdfiumError};

use crate::{Backend, ExtractError, Extraction, Page, PageContent, PageSize, Rect, Span, guard};

/// Must match the exact version in `Cargo.toml` (`=0.9.4`).
const PDFIUM_RENDER_VERSION: &str = "0.9.4";

pub struct Pdfium {
    pub lib_dir: PathBuf,
}

impl Pdfium {
    pub fn from_env() -> Option<Self> {
        std::env::var_os("BIB_PDFIUM_LIB_DIR").map(|dir| Self { lib_dir: PathBuf::from(dir) })
    }

    /// Binds the native `libpdfium`. `pdfium-render` keeps the bindings process-wide in a
    /// global `OnceCell`; a second binding attempt (e.g. a parallel test run in this
    /// process) fails even though the library is already loaded. In that case we fall
    /// back to the already-bound instance, same as `Pdfium::default()`.
    fn bind(&self) -> Result<PdfiumLib, ExtractError> {
        let lib = PdfiumLib::pdfium_platform_library_name_at_path(&self.lib_dir);
        match PdfiumLib::bind_to_library(lib) {
            Ok(bindings) => Ok(PdfiumLib::new(bindings)),
            Err(PdfiumError::PdfiumLibraryBindingsAlreadyInitialized) => Ok(PdfiumLib::default()),
            Err(e) => Err(ExtractError::Unavailable(e.to_string())),
        }
    }
}

impl Backend for Pdfium {
    fn name(&self) -> &'static str {
        "pdfium"
    }

    fn version(&self) -> String {
        match read_libpdfium_build(&self.lib_dir) {
            Some(build) => format!("pdfium-render {PDFIUM_RENDER_VERSION}, libpdfium {build}"),
            None => format!("pdfium-render {PDFIUM_RENDER_VERSION}, libpdfium from {}", self.lib_dir.display()),
        }
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        guard(|| {
            let pdfium = self.bind()?;
            let doc = pdfium.load_pdf_from_file(path, None).map_err(|e| ExtractError::Open(e.to_string()))?;
            let mut pages = Vec::new();
            for (index, page) in doc.pages().iter().enumerate() {
                let page_error = |e: PdfiumError| ExtractError::Page { index, message: e.to_string() };
                let height = page.height().value;
                let rotation = match page.rotation().map_err(page_error)? {
                    PdfPageRenderRotation::None => 0,
                    PdfPageRenderRotation::Degrees90 => 90,
                    PdfPageRenderRotation::Degrees180 => 180,
                    PdfPageRenderRotation::Degrees270 => 270,
                };
                let text = page.text().map_err(page_error)?;
                let spans = text
                    .segments()
                    .iter()
                    .filter(|segment| !segment.text().trim().is_empty())
                    .map(|segment| {
                        let b = segment.bounds();
                        Span {
                            text: segment.text(),
                            bbox: Rect {
                                left: b.left().value,
                                top: height - b.top().value,
                                right: b.right().value,
                                bottom: height - b.bottom().value,
                            },
                            font: String::new(),
                        }
                    })
                    .collect();
                pages.push(Page {
                    index,
                    size: Some(PageSize { width: page.width().value, height, rotation: Some(rotation) }),
                    content: PageContent::Spans(spans),
                });
            }
            Ok(Extraction { backend: self.name(), backend_version: self.version(), pages })
        })
    }
}

/// `pdfium-render` 0.9.4 has no API to query the actually loaded `libpdfium` version at
/// runtime (`PdfiumApiVersion` only reflects the `pdfium_*` feature chosen at compile time,
/// not the real binary). Instead we read the `VERSION` file that `pdfium-binaries` releases
/// ship alongside `lib/` (`MAJOR`/`MINOR`/`BUILD`/`PATCH`).
fn read_libpdfium_build(lib_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(lib_dir.parent()?.join("VERSION")).ok()?;
    let field = |key: &str| content.lines().find_map(|line| line.split_once('=').filter(|(k, _)| k.trim() == key).map(|(_, v)| v.trim()));
    let (major, minor, build, patch) = (field("MAJOR")?, field("MINOR")?, field("BUILD")?, field("PATCH")?);
    Some(format!("{major}.{minor}.{build}.{patch}"))
}
