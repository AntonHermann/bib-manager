mod common;

use std::io::Write;

use bib_core::normalize::normalize;
use bib_extract::Backend;
use bib_extract::backends::pdf_extract::PdfExtract;

#[test]
fn pages_are_text_only() {
    let extraction = PdfExtract.extract(&common::fixture("dwork2006")).unwrap();
    assert_eq!(extraction.backend, "pdf-extract");
    assert!(!extraction.pages.is_empty());
    assert!(extraction.pages.iter().all(|p| !p.has_geometry() && p.size.is_none()));
}

#[test]
fn recovers_epsilon_from_type1_math_font() {
    // Measured during brainstorming: 28 ε in Dwork 2006 (pdf_oxide: 0).
    let extraction = PdfExtract.extract(&common::fixture("dwork2006")).unwrap();
    let epsilons = normalize(&extraction.text()).text.matches('ε').count();
    assert!(epsilons >= 25, "only {epsilons} ε found");
}

#[test]
fn broken_pdf_is_an_error_not_a_panic() {
    let mut file = tempfile::Builder::new().suffix(".pdf").tempfile().unwrap();
    file.write_all(b"%PDF-1.4\nnot a real PDF\n").unwrap();
    assert!(PdfExtract.extract(file.path()).is_err());
}
