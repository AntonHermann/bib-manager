#![cfg(feature = "pdfium")]

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::pdfium::Pdfium;
use bib_extract::{Backend, PageContent};

fn backend() -> Pdfium {
    Pdfium::from_env().expect("BIB_PDFIUM_LIB_DIR setzen, z. B. auf bench/cache/pdfium/lib")
}

#[test]
fn extracts_pages_with_geometry() {
    let extraction = backend().extract(&common::fixture("abadi2016")).unwrap();
    assert!(!extraction.pages.is_empty());
    assert!(extraction.pages.iter().all(|p| p.has_geometry() && p.size.is_some()));
}

#[test]
fn pdfium_and_mutool_agree_on_title_position() {
    let mutool = Mutool::default();
    if !mutool.is_available() {
        eprintln!("ÜBERSPRUNGEN: mutool nicht installiert");
        return;
    }
    let path = common::fixture("dwork2006");
    let top_of = |e: &bib_extract::Extraction| {
        let PageContent::Spans(spans) = &e.pages[0].content else { panic!("keine Spans") };
        spans.iter().filter(|s| s.text.contains("Differential")).map(|s| s.bbox.top).fold(f32::INFINITY, f32::min)
    };
    let reference = top_of(&mutool.extract(&path).unwrap());
    let pdfium = top_of(&backend().extract(&path).unwrap());
    assert!((reference - pdfium).abs() < 6.0, "top: mutool {reference} vs pdfium {pdfium}");
}
