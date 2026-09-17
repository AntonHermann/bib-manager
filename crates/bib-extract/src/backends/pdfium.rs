//! Backend auf Basis von pdfium (Chromes PDF-Engine) über `pdfium-render`.
//! Braucht `libpdfium.so` zur Laufzeit; Pfad über `BIB_PDFIUM_LIB_DIR`.

use std::path::{Path, PathBuf};

use pdfium_render::prelude::{PdfPageRenderRotation, Pdfium as PdfiumLib, PdfiumError};

use crate::{Backend, ExtractError, Extraction, Page, PageContent, PageSize, Rect, Span, guard};

pub struct Pdfium {
    pub lib_dir: PathBuf,
}

impl Pdfium {
    pub fn from_env() -> Option<Self> {
        std::env::var_os("BIB_PDFIUM_LIB_DIR").map(|dir| Self { lib_dir: PathBuf::from(dir) })
    }

    /// Bindet die native `libpdfium`. `pdfium-render` hält die Bindings prozessweit in einer
    /// globalen `OnceCell`; ein zweiter Bindungsversuch (z. B. paralleler Testlauf in
    /// diesem Prozess) schlägt fehl, obwohl die Bibliothek bereits geladen ist. In diesem
    /// Fall greifen wir wie `Pdfium::default()` auf die bereits gebundene Instanz zurück.
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
            Some(build) => format!("pdfium-render 0.9.4, libpdfium {build}"),
            None => format!("pdfium-render 0.9.4, libpdfium aus {}", self.lib_dir.display()),
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

/// `pdfium-render` 0.9.4 hat keine API, um die tatsächlich geladene `libpdfium`-Version zur
/// Laufzeit abzufragen (`PdfiumApiVersion` spiegelt nur das zur Kompilierzeit gewählte
/// `pdfium_*`-Feature wider, nicht das reale Binary). Stattdessen lesen wir die `VERSION`-Datei,
/// die die `pdfium-binaries`-Releases neben `lib/` mitliefern (`MAJOR`/`MINOR`/`BUILD`/`PATCH`).
fn read_libpdfium_build(lib_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(lib_dir.parent()?.join("VERSION")).ok()?;
    let field = |key: &str| content.lines().find_map(|line| line.split_once('=').filter(|(k, _)| k.trim() == key).map(|(_, v)| v.trim()));
    let (major, minor, build, patch) = (field("MAJOR")?, field("MINOR")?, field("BUILD")?, field("PATCH")?);
    Some(format!("{major}.{minor}.{build}.{patch}"))
}
