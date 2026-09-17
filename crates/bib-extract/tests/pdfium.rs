#![cfg(feature = "pdfium")]

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::pdfium::Pdfium;
use bib_extract::{Backend, PageContent, Rect};

const TOLERANCE: f32 = 6.0;

fn backend() -> Pdfium {
    Pdfium::from_env().expect("set BIB_PDFIUM_LIB_DIR, e.g. to bench/cache/pdfium/lib")
}

fn first_box_containing(extraction: &bib_extract::Extraction, needle: &str) -> Rect {
    let PageContent::Spans(spans) = &extraction.pages[0].content else { panic!("{}: no spans", extraction.backend) };
    spans
        .iter()
        .filter(|s| s.text.contains(needle))
        .min_by(|a, b| a.bbox.top.total_cmp(&b.bbox.top))
        .unwrap_or_else(|| panic!("{}: \"{needle}\" not found", extraction.backend))
        .bbox
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
        eprintln!("SKIPPED: mutool not installed");
        return;
    }
    let path = common::fixture("dwork2006");
    let reference = first_box_containing(&mutool.extract(&path).unwrap(), "Differential");
    let pdfium = first_box_containing(&backend().extract(&path).unwrap(), "Differential");
    assert!((reference.top - pdfium.top).abs() < TOLERANCE, "top: mutool {} vs pdfium {}", reference.top, pdfium.top);
    assert!((reference.left - pdfium.left).abs() < TOLERANCE, "left: mutool {} vs pdfium {}", reference.left, pdfium.left);
}
