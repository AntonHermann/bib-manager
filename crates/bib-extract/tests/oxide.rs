mod common;

use bib_core::normalize::normalize;
use bib_extract::backends::oxide::PdfOxide;
use bib_extract::{Backend, PageContent};

#[test]
fn extracts_every_page_with_geometry() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    assert_eq!(extraction.backend, "pdf_oxide");
    assert!(!extraction.pages.is_empty());
    for page in &extraction.pages {
        assert!(page.has_geometry(), "Seite {} ohne Geometrie", page.index);
    }
}

#[test]
fn first_page_size_is_plausible() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    let size = extraction.pages[0].size.expect("Seitengröße");
    assert!((500.0..700.0).contains(&size.width), "Breite {}", size.width);
    assert!((700.0..900.0).contains(&size.height), "Höhe {}", size.height);
}

#[test]
fn title_is_near_the_top_in_top_left_coordinates() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    let page = &extraction.pages[0];
    let PageContent::Spans(spans) = &page.content else { panic!("keine Spans") };
    let title_top = spans
        .iter()
        .filter(|s| s.text.contains("Differential"))
        .map(|s| s.bbox.top)
        .fold(f32::INFINITY, f32::min);
    let height = page.size.unwrap().height;
    assert!(title_top < 0.25 * height, "Titel bei top={title_top}, Seitenhöhe {height}");
}

#[test]
fn text_contains_the_title_after_normalization() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    assert!(normalize(&extraction.text()).text.contains("differential privacy"));
}
