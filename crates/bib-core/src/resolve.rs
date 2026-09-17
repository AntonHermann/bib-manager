//! Resolving citation keys across a project's libraries (spec §6). Keys are not unique across libraries:
//! a key resolves in the order of `[zotero] libraries`; several sources count as one work if their DOI
//! matches or their normalized title and year match, and the first library wins silently. Otherwise the key
//! is a conflict for the review queue.

use std::collections::BTreeMap;

use rusqlite::{Connection, Row};

use crate::library::LibraryRef;
use crate::normalize::normalize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRow {
    pub id: i64,
    pub library: LibraryRef,
    pub item_key: String,
    pub citation_key: Option<String>,
    pub title: String,
    pub year: Option<i32>,
    pub doi: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    NotFound,
    /// One source, or several that are the same work; `duplicates` lists the ones that lost.
    Found {
        source: SourceRow,
        duplicates: Vec<SourceRow>,
    },
    /// Different works share the key; in resolution order.
    Conflict(Vec<SourceRow>),
}

pub fn resolve_key(conn: &Connection, key: &str, order: &[LibraryRef]) -> rusqlite::Result<Resolution> {
    Ok(decide(rank(query_sources(conn, Some(key))?, order)))
}

pub fn same_work(a: &SourceRow, b: &SourceRow) -> bool {
    let doi_match = matches!((&a.doi, &b.doi), (Some(x), Some(y)) if x.eq_ignore_ascii_case(y));
    let title = normalize(&a.title).text;
    let title_year_match =
        !title.is_empty() && title == normalize(&b.title).text && a.year.is_some() && a.year == b.year;
    doi_match || title_year_match
}

pub fn keyed_sources(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<SourceRow>> {
    Ok(grouped(conn, order)?
        .into_values()
        .filter_map(|rows| rows.into_iter().next())
        .collect())
}

pub fn conflicts(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<(String, Vec<SourceRow>)>> {
    Ok(grouped(conn, order)?
        .into_iter()
        .filter_map(|(key, rows)| match decide(rows) {
            Resolution::Conflict(rows) => Some((key, rows)),
            _ => None,
        })
        .collect())
}

fn decide(mut candidates: Vec<SourceRow>) -> Resolution {
    if candidates.is_empty() {
        return Resolution::NotFound;
    }
    let first = candidates.remove(0);
    if candidates.iter().all(|other| same_work(&first, other)) {
        Resolution::Found {
            source: first,
            duplicates: candidates,
        }
    } else {
        candidates.insert(0, first);
        Resolution::Conflict(candidates)
    }
}

/// Active sources with a citation key in the project's libraries, grouped by key, each group in resolution order.
fn grouped(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<BTreeMap<String, Vec<SourceRow>>> {
    let mut groups: BTreeMap<String, Vec<SourceRow>> = BTreeMap::new();
    for row in rank(query_sources(conn, None)?, order) {
        if let Some(key) = row.citation_key.clone() {
            groups.entry(key).or_default().push(row);
        }
    }
    Ok(groups)
}

/// Keeps rows from `order`'s libraries, sorted by library position, then by row id.
fn rank(rows: Vec<SourceRow>, order: &[LibraryRef]) -> Vec<SourceRow> {
    let mut ranked: Vec<(usize, SourceRow)> = rows
        .into_iter()
        .filter_map(|row| {
            order
                .iter()
                .position(|library| *library == row.library)
                .map(|position| (position, row))
        })
        .collect();
    ranked.sort_by_key(|(position, row)| (*position, row.id));
    ranked.into_iter().map(|(_, row)| row).collect()
}

/// Active sources with a citation key, optionally only one key.
fn query_sources(conn: &Connection, key: Option<&str>) -> rusqlite::Result<Vec<SourceRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, l.kind, l.zotero_id, s.item_key, s.citation_key, s.title, s.year, s.doi
         FROM source s JOIN library l ON l.id = s.library_id
         WHERE s.status = 'active' AND s.citation_key IS NOT NULL AND (?1 IS NULL OR s.citation_key = ?1)",
    )?;
    stmt.query_map([key], row_to_source)?.collect()
}

fn row_to_source(row: &Row) -> rusqlite::Result<SourceRow> {
    let kind: String = row.get(1)?;
    let library = LibraryRef::from_db(&kind, row.get(2)?).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            format!("unknown library kind {kind:?}").into(),
        )
    })?;
    Ok(SourceRow {
        id: row.get(0)?,
        library,
        item_key: row.get(3)?,
        citation_key: row.get(4)?,
        title: row.get(5)?,
        year: row.get(6)?,
        doi: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{MIGRATIONS, migrate};
    use rusqlite::params;

    const USER: LibraryRef = LibraryRef::User;
    const LAB: LibraryRef = LibraryRef::Group(42);
    #[allow(dead_code)]
    const OTHER: LibraryRef = LibraryRef::Group(99);

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn.execute(
            "INSERT INTO library (id, kind, zotero_id) VALUES (1, 'user', 0), (2, 'group', 42), (3, 'group', 99)",
            [],
        )
        .unwrap();
        conn
    }

    fn add(conn: &Connection, library_id: i64, item_key: &str, key: &str, title: &str, year: i32, doi: Option<&str>) {
        conn.execute(
            "INSERT INTO source (library_id, item_key, citation_key, item_type, title, year, doi, data, data_hash)
             VALUES (?1, ?2, ?3, 'journalArticle', ?4, ?5, ?6, '{}', 'h')",
            params![library_id, item_key, key, title, year, doi],
        )
        .unwrap();
    }

    fn keys(rows: &[SourceRow]) -> Vec<(LibraryRef, &str)> {
        rows.iter().map(|r| (r.library, r.item_key.as_str())).collect()
    }

    #[test]
    fn unknown_key_and_unlisted_library_are_not_found() {
        let conn = db();
        add(&conn, 3, "O1", "dwork2006", "Differential Privacy", 2006, None);
        assert_eq!(
            resolve_key(&conn, "dwork2006", &[USER, LAB]).unwrap(),
            Resolution::NotFound
        );
        assert_eq!(resolve_key(&conn, "nobody2020", &[USER]).unwrap(), Resolution::NotFound);
    }

    #[test]
    fn same_work_resolves_to_the_first_library() {
        let conn = db();
        add(&conn, 2, "G1", "dwork2006", "Differential privacy", 2006, None);
        add(&conn, 1, "U1", "dwork2006", "Differential Privacy", 2006, None);
        let Resolution::Found { source, duplicates } = resolve_key(&conn, "dwork2006", &[USER, LAB]).unwrap() else {
            panic!("expected Found")
        };
        assert_eq!((source.library, source.item_key.as_str()), (USER, "U1"));
        assert_eq!(keys(&duplicates), [(LAB, "G1")]);
        let Resolution::Found { source, .. } = resolve_key(&conn, "dwork2006", &[LAB, USER]).unwrap() else {
            panic!("expected Found")
        };
        assert_eq!(source.library, LAB);
    }

    #[test]
    fn different_works_are_a_conflict_in_resolution_order() {
        let conn = db();
        add(&conn, 1, "U1", "smith2020", "Graph Databases", 2020, None);
        add(&conn, 2, "G1", "smith2020", "Protein Folding", 2020, None);
        let Resolution::Conflict(rows) = resolve_key(&conn, "smith2020", &[LAB, USER]).unwrap() else {
            panic!("expected Conflict")
        };
        assert_eq!(keys(&rows), [(LAB, "G1"), (USER, "U1")]);
    }

    #[test]
    fn retired_sources_do_not_resolve() {
        let conn = db();
        add(&conn, 1, "U1", "dwork2006", "Differential Privacy", 2006, None);
        conn.execute("UPDATE source SET status = 'retired'", []).unwrap();
        assert_eq!(resolve_key(&conn, "dwork2006", &[USER]).unwrap(), Resolution::NotFound);
    }

    fn row(title: &str, year: Option<i32>, doi: Option<&str>) -> SourceRow {
        SourceRow {
            id: 1,
            library: USER,
            item_key: "K".into(),
            citation_key: Some("k".into()),
            title: title.into(),
            year,
            doi: doi.map(Into::into),
        }
    }

    #[test]
    fn same_work_rules() {
        assert!(
            same_work(&row("A", Some(1), Some("10.1/X")), &row("B", Some(2), Some("10.1/x"))),
            "DOI decides"
        );
        assert!(same_work(
            &row("Deep  Learning–Privacy", Some(2016), None),
            &row("deep learning-privacy", Some(2016), Some("10.1/y"))
        ));
        assert!(!same_work(
            &row("Deep Learning", Some(2016), None),
            &row("Deep Learning", Some(2017), None)
        ));
        assert!(
            !same_work(&row("Deep Learning", None, None), &row("Deep Learning", None, None)),
            "year required"
        );
        assert!(
            !same_work(&row("", Some(2016), None), &row("", Some(2016), None)),
            "empty titles never match"
        );
    }

    #[test]
    fn keyed_sources_and_conflicts_cover_the_project_libraries() {
        let conn = db();
        add(&conn, 1, "U1", "dwork2006", "Differential Privacy", 2006, None);
        add(&conn, 2, "G1", "dwork2006", "Differential Privacy", 2006, None);
        add(&conn, 1, "U2", "smith2020", "Graph Databases", 2020, None);
        add(&conn, 2, "G2", "smith2020", "Protein Folding", 2020, None);
        add(
            &conn,
            2,
            "G3",
            "abadi2016",
            "Deep Learning with Differential Privacy",
            2016,
            None,
        );
        add(&conn, 3, "O1", "other2021", "Elsewhere", 2021, None);
        conn.execute("INSERT INTO source (library_id, item_key, item_type, data, data_hash) VALUES (1, 'NOKEY', 'book', '{}', 'h')", [])
            .unwrap();

        let winners = keyed_sources(&conn, &[USER, LAB]).unwrap();
        let summary: Vec<_> = winners
            .iter()
            .map(|r| (r.citation_key.as_deref().unwrap(), r.item_key.as_str()))
            .collect();
        assert_eq!(summary, [("abadi2016", "G3"), ("dwork2006", "U1"), ("smith2020", "U2")]);

        let found = conflicts(&conn, &[USER, LAB]).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "smith2020");
        assert_eq!(keys(&found[0].1), [(USER, "U2"), (LAB, "G2")]);
    }
}
