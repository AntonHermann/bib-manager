//! A project's `.bib` export, read in memory only and never written (spec §6). Parsed with `biblatex`, the
//! library Typst uses.

use std::collections::BTreeMap;

use biblatex::{Bibliography, ChunksExt, DateValue, PermissiveType};

use crate::normalize::normalize;
use crate::resolve::SourceRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibEntry {
    pub key: String,
    pub title: Option<String>,
    pub year: Option<i32>,
    pub doi: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BibFinding {
    /// Key only in the `.bib` file: not in Zotero, add it there.
    OnlyInBib { key: String },
    /// Both know the key but a field differs.
    MetadataDiffers {
        key: String,
        field: &'static str,
        bib: String,
        zotero: String,
    },
}

pub fn parse_bib(src: &str) -> Result<Vec<BibEntry>, String> {
    let bibliography = Bibliography::parse(src).map_err(|e| e.to_string())?;
    Ok(bibliography
        .iter()
        .map(|entry| BibEntry {
            key: entry.key.clone(),
            title: entry
                .title()
                .ok()
                .map(|chunks| chunks.format_verbatim().replace(['{', '}'], "")),
            year: match entry.date() {
                Ok(PermissiveType::Typed(date)) => Some(match date.value {
                    DateValue::At(d) | DateValue::After(d) | DateValue::Before(d) | DateValue::Between(d, _) => d.year,
                }),
                _ => None,
            },
            doi: entry.doi().ok(),
        })
        .collect())
}

/// Findings sorted by key. Keys only in Zotero are not reported here: whether that matters depends on the
/// document's citations (Typst parser, spec step 5).
pub fn compare(bib: &[BibEntry], zotero: &[SourceRow]) -> Vec<BibFinding> {
    let zotero_by_key: BTreeMap<&str, &SourceRow> = zotero
        .iter()
        .filter_map(|s| s.citation_key.as_deref().map(|key| (key, s)))
        .collect();
    let bib_by_key: BTreeMap<&str, &BibEntry> = bib.iter().map(|e| (e.key.as_str(), e)).collect();
    let mut findings = Vec::new();
    for (key, entry) in bib_by_key {
        let Some(source) = zotero_by_key.get(key) else {
            findings.push(BibFinding::OnlyInBib { key: key.to_string() });
            continue;
        };
        let mut differs = |field: &'static str, bib: String, zotero: String| {
            findings.push(BibFinding::MetadataDiffers {
                key: key.to_string(),
                field,
                bib,
                zotero,
            })
        };
        if let Some(title) = &entry.title
            && normalize(title).text != normalize(&source.title).text
        {
            differs("title", title.clone(), source.title.clone());
        }
        if let (Some(bib_year), Some(zotero_year)) = (entry.year, source.year)
            && bib_year != zotero_year
        {
            differs("year", bib_year.to_string(), zotero_year.to_string());
        }
        if let (Some(bib_doi), Some(zotero_doi)) = (&entry.doi, &source.doi)
            && !bib_doi.eq_ignore_ascii_case(zotero_doi)
        {
            differs("doi", bib_doi.clone(), zotero_doi.clone());
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::LibraryRef;

    const BIB: &str = r#"
@inproceedings{dwork2006,
  title = {Differential {P}rivacy},
  author = {Dwork, Cynthia},
  year = {2006},
  doi = {10.1007/11787006_1},
}
@article{abadi2016, title = {Deep Learning with Differential Privacy}, date = {2016-10}}
@misc{nodate, title = {Untitled draft}}
"#;

    fn source(key: &str, title: &str, year: Option<i32>, doi: Option<&str>) -> SourceRow {
        SourceRow {
            id: 1,
            library: LibraryRef::User,
            item_key: "K".into(),
            citation_key: Some(key.into()),
            title: title.into(),
            year,
            doi: doi.map(Into::into),
        }
    }

    #[test]
    fn parses_keys_titles_years_and_dois() {
        let entries = parse_bib(BIB).unwrap();
        assert_eq!(
            entries,
            [
                BibEntry {
                    key: "dwork2006".into(),
                    title: Some("Differential Privacy".into()),
                    year: Some(2006),
                    doi: Some("10.1007/11787006_1".into())
                },
                BibEntry {
                    key: "abadi2016".into(),
                    title: Some("Deep Learning with Differential Privacy".into()),
                    year: Some(2016),
                    doi: None
                },
                BibEntry {
                    key: "nodate".into(),
                    title: Some("Untitled draft".into()),
                    year: None,
                    doi: None
                },
            ]
        );
    }

    #[test]
    fn broken_file_is_an_error() {
        assert!(parse_bib("@article{x, title = {unclosed").is_err());
    }

    #[test]
    fn reports_missing_keys_and_drift_but_tolerates_formatting() {
        let bib = parse_bib(BIB).unwrap();
        let zotero = [
            source(
                "dwork2006",
                "Differential privacy",
                Some(2006),
                Some("10.1007/11787006_1"),
            ),
            source("abadi2016", "Deep Learning with Differential Privacy", Some(2015), None),
            source("unused2020", "Cited nowhere", Some(2020), None),
        ];
        assert_eq!(
            compare(&bib, &zotero),
            [
                BibFinding::MetadataDiffers {
                    key: "abadi2016".into(),
                    field: "year",
                    bib: "2016".into(),
                    zotero: "2015".into()
                },
                BibFinding::OnlyInBib { key: "nodate".into() },
            ]
        );
    }

    #[test]
    fn differing_title_and_doi_are_reported() {
        let bib = parse_bib(BIB).unwrap();
        let zotero = [source(
            "dwork2006",
            "Calibrating Noise",
            Some(2006),
            Some("10.1007/OTHER"),
        )];
        let findings = compare(&bib[..1], &zotero);
        let fields: Vec<_> = findings
            .iter()
            .map(|f| match f {
                BibFinding::MetadataDiffers { field, .. } => *field,
                BibFinding::OnlyInBib { .. } => "only",
            })
            .collect();
        assert_eq!(fields, ["title", "doi"]);
    }
}
