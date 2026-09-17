//! The same passage of text must be at the same position across all backends with geometry (Spec §7).

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::oxide::PdfOxide;
use bib_extract::{Backend, PageContent, Rect};

const TOLERANCE: f32 = 6.0;

pub fn first_box_containing(extraction: &bib_extract::Extraction, needle: &str) -> Rect {
    let PageContent::Spans(spans) = &extraction.pages[0].content else {
        panic!("{}: no spans", extraction.backend)
    };
    spans
        .iter()
        .filter(|s| s.text.contains(needle))
        .min_by(|a, b| a.bbox.top.total_cmp(&b.bbox.top))
        .unwrap_or_else(|| panic!("{}: \"{needle}\" not found", extraction.backend))
        .bbox
}

#[test]
fn pdf_oxide_and_mutool_agree_on_title_position() {
    let mutool = Mutool::default();
    if !mutool.is_available() {
        eprintln!("SKIPPED: mutool not installed");
        return;
    }
    let path = common::fixture("dwork2006");
    let reference = first_box_containing(&mutool.extract(&path).unwrap(), "Differential");
    let oxide = first_box_containing(&PdfOxide.extract(&path).unwrap(), "Differential");
    assert!(
        (reference.top - oxide.top).abs() < TOLERANCE,
        "top: mutool {} vs pdf_oxide {}",
        reference.top,
        oxide.top
    );
    assert!(
        (reference.left - oxide.left).abs() < TOLERANCE,
        "left: mutool {} vs pdf_oxide {}",
        reference.left,
        oxide.left
    );
}
