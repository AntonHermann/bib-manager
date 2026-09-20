//! `bib doctor`: health of the database, the Zotero sync and the current project (spec §6, §10).

use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

use crate::bibfile::{BibFinding, compare, parse_bib};
use crate::library::LibraryRef;
use crate::project::ProjectConfig;
use crate::resolve::{SourceRow, conflicts, keyed_sources};
use crate::review::{NewReviewItem, enqueue, open_count};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
}

pub struct Project<'a> {
    pub root: &'a Path,
    pub config: &'a ProjectConfig,
}

pub struct DoctorInput<'a> {
    pub conn: &'a Connection,
    pub db_path: &'a Path,
    /// Outcome of probing the Zotero local API.
    pub zotero: Result<(), String>,
    pub project: Option<Project<'a>>,
}

/// Lists at most this many entries inside one finding.
const LIST_LIMIT: usize = 10;

pub fn run(input: &DoctorInput) -> anyhow::Result<Vec<Finding>> {
    let conn = input.conn;
    let mut findings = Vec::new();
    let mut push = |severity: Severity, code: &'static str, message: String| {
        findings.push(Finding {
            severity,
            code,
            message,
        })
    };

    let version = crate::db::schema_version(conn)?;
    push(
        Severity::Info,
        "database",
        format!("{} (schema version {version})", input.db_path.display()),
    );
    match &input.zotero {
        Ok(()) => push(Severity::Info, "zotero", "local API reachable".to_string()),
        Err(message) => push(
            Severity::Warning,
            "zotero_unreachable",
            format!("{message}; working from the last synced state"),
        ),
    }

    let libraries = library_status(conn)?;
    if libraries.is_empty() {
        push(
            Severity::Warning,
            "never_synced",
            "no Zotero library synced yet; run `bib sync`".to_string(),
        );
    }
    for library in &libraries {
        match library.age {
            Some(age) => push(
                Severity::Info,
                "library",
                format!(
                    "{} \"{}\": {} active sources, last sync {} ago",
                    library.library,
                    library.name,
                    library.active_sources,
                    format_age(age)
                ),
            ),
            None => push(
                Severity::Warning,
                "never_synced",
                format!("{} has never been synced successfully", library.library),
            ),
        }
    }

    let without_key = strings(
        conn,
        "SELECT l.kind, l.zotero_id, s.item_key, s.title FROM source s JOIN library l ON l.id = s.library_id
         WHERE s.status = 'active' AND s.citation_key IS NULL ORDER BY l.id, s.item_key",
        |library, key, title| format!("{library}:{key} \"{title}\""),
    )?;
    if !without_key.is_empty() {
        push(
            Severity::Warning,
            "missing_citation_key",
            format!(
                "{} source(s) without citation key, not citable: {}",
                without_key.len(),
                list(&without_key)
            ),
        );
    }

    let paths: Vec<String> = conn
        .prepare("SELECT path FROM attachment WHERE status = 'active' AND path IS NOT NULL ORDER BY path")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    let missing: Vec<String> = paths.into_iter().filter(|p| !Path::new(p).exists()).collect();
    if !missing.is_empty() {
        push(
            Severity::Warning,
            "missing_pdf_file",
            format!(
                "{} PDF attachment(s) point to missing files: {}",
                missing.len(),
                list(&missing)
            ),
        );
    }
    let without_file = count(
        conn,
        "SELECT count(*) FROM attachment WHERE status = 'active' AND path IS NULL",
    )?;
    if without_file > 0 {
        push(
            Severity::Info,
            "pdf_without_file",
            format!("{without_file} PDF attachment(s) have no local file in Zotero"),
        );
    }
    let retired = count(conn, "SELECT count(*) FROM source WHERE status = 'retired'")?;
    if retired > 0 {
        push(
            Severity::Info,
            "retired_sources",
            format!("{retired} source(s) no longer in Zotero are kept as retired"),
        );
    }

    if let Some(project) = &input.project {
        check_project(conn, project, &libraries, &mut push)?;
    }

    // Last, so that items queued by the project checks are counted.
    let open = open_count(conn)?;
    if open > 0 {
        push(Severity::Warning, "review_open", format!("{open} open review item(s)"));
    }
    Ok(findings)
}

/// "42 s", "5 min", "3 h", "12 d".
pub fn format_age(seconds: i64) -> String {
    let s = seconds.max(0);
    match s {
        0..60 => format!("{s} s"),
        60..3600 => format!("{} min", s / 60),
        3600..86400 => format!("{} h", s / 3600),
        _ => format!("{} d", s / 86400),
    }
}

struct LibraryStatus {
    library: LibraryRef,
    name: String,
    active_sources: i64,
    /// Seconds since the last successful sync.
    age: Option<i64>,
}

fn library_status(conn: &Connection) -> anyhow::Result<Vec<LibraryStatus>> {
    let mut stmt = conn.prepare(
        "SELECT l.kind, l.zotero_id, l.name,
                (SELECT count(*) FROM source s WHERE s.library_id = l.id AND s.status = 'active'),
                (SELECT CAST((julianday('now') - julianday(max(r.finished_at))) * 86400 AS INTEGER)
                 FROM sync_run r WHERE r.library_id = l.id AND r.status = 'ok')
         FROM library l ORDER BY l.kind = 'group', l.zotero_id",
    )?;
    let rows = stmt.query_map([], |row| {
        let kind: String = row.get(0)?;
        Ok((kind, row.get::<_, i64>(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?;
    let mut statuses = Vec::new();
    for row in rows {
        let (kind, zotero_id, name, active_sources, age) = row?;
        let library = LibraryRef::from_db(&kind, zotero_id)
            .ok_or_else(|| anyhow::anyhow!("unknown library kind {kind:?} in database"))?;
        statuses.push(LibraryStatus {
            library,
            name,
            active_sources,
            age,
        });
    }
    Ok(statuses)
}

fn check_project(
    conn: &Connection,
    project: &Project,
    libraries: &[LibraryStatus],
    push: &mut impl FnMut(Severity, &'static str, String),
) -> anyhow::Result<()> {
    let config = project.config;
    push(
        Severity::Info,
        "project",
        format!("{} ({})", config.name, project.root.display()),
    );
    let order = config.libraries()?;
    for library in &order {
        if !libraries.iter().any(|known| known.library == *library) {
            push(
                Severity::Warning,
                "library_unknown",
                format!("{library} from bib.toml is not in the database; run `bib sync` or fix [zotero] libraries"),
            );
        }
    }

    for (key, sources) in conflicts(conn, &order)? {
        let described: Vec<String> = sources.iter().map(describe).collect();
        let message = format!("citation key {key} names different works: {}", described.join("; "));
        push(Severity::Warning, "key_conflict", message.clone());
        enqueue(
            conn,
            &NewReviewItem {
                kind: "key_conflict",
                node_type: None,
                node_id: None,
                message: &message,
                details: &serde_json::json!({
                    "project": config.id,
                    "key": key,
                    "sources": sources.iter().map(|s| format!("{}:{}", s.library, s.item_key)).collect::<Vec<_>>(),
                }),
                dedupe_key: &format!("key_conflict:{}:{key}", config.id),
            },
        )?;
    }

    let Some(bibliography) = &config.bibliography else {
        return Ok(());
    };
    let path = project.root.join(&bibliography.path);
    let entries = match std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|text| parse_bib(&text))
    {
        Ok(entries) => entries,
        Err(message) => {
            push(
                Severity::Error,
                "bib_unreadable",
                format!("{}: {message}", path.display()),
            );
            return Ok(());
        }
    };
    for finding in compare(&entries, &keyed_sources(conn, &order)?) {
        match finding {
            BibFinding::OnlyInBib { key } => push(
                Severity::Warning,
                "bib_only",
                format!(
                    "{key} is in {} but not in Zotero; add it in Zotero",
                    bibliography.path.display()
                ),
            ),
            BibFinding::MetadataDiffers {
                key,
                field,
                bib,
                zotero,
            } => push(
                Severity::Warning,
                "metadata_drift",
                format!("{key}: {field} differs (.bib: {bib:?}, Zotero: {zotero:?})"),
            ),
        }
    }
    Ok(())
}

fn describe(source: &SourceRow) -> String {
    let year = source.year.map_or_else(String::new, |y| format!(" ({y})"));
    format!("{}:{} \"{}\"{year}", source.library, source.item_key, source.title)
}

fn count(conn: &Connection, sql: &str) -> rusqlite::Result<i64> {
    conn.query_row(sql, [], |row| row.get(0))
}

/// Rows of (kind, zotero_id, item key, title) formatted with `format`.
fn strings(
    conn: &Connection,
    sql: &str,
    format: impl Fn(LibraryRef, &str, &str) -> String,
) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (kind, zotero_id, key, title) = row?;
        let library = LibraryRef::from_db(&kind, zotero_id)
            .ok_or_else(|| anyhow::anyhow!("unknown library kind {kind:?} in database"))?;
        out.push(format(library, &key, &title));
    }
    Ok(out)
}

fn list(items: &[String]) -> String {
    let mut text = items.iter().take(LIST_LIMIT).cloned().collect::<Vec<_>>().join(", ");
    if items.len() > LIST_LIMIT {
        text.push_str(&format!(", and {} more", items.len() - LIST_LIMIT));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{MIGRATIONS, migrate};

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn
    }

    fn input<'a>(conn: &'a Connection, project: Option<Project<'a>>) -> DoctorInput<'a> {
        DoctorInput {
            conn,
            db_path: Path::new("/data/bib.db"),
            zotero: Ok(()),
            project,
        }
    }

    fn codes(findings: &[Finding]) -> Vec<(&'static str, Severity)> {
        findings.iter().map(|f| (f.code, f.severity)).collect()
    }

    fn synced_user_library(conn: &Connection) {
        conn.execute(
            "INSERT INTO library (id, kind, zotero_id, name) VALUES (1, 'user', 0, 'My Library')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO sync_run (library_id, finished_at, status) VALUES (1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), 'ok')", [])
            .unwrap();
    }

    fn add_source(conn: &Connection, item_key: &str, citation_key: Option<&str>, title: &str, year: i32) {
        conn.execute(
            "INSERT INTO source (library_id, item_key, citation_key, item_type, title, year, data, data_hash)
             VALUES (1, ?1, ?2, 'journalArticle', ?3, ?4, '{}', 'h')",
            rusqlite::params![item_key, citation_key, title, year],
        )
        .unwrap();
    }

    #[test]
    fn empty_database_needs_a_sync() {
        let conn = db();
        let findings = run(&input(&conn, None)).unwrap();
        assert_eq!(
            codes(&findings),
            [
                ("database", Severity::Info),
                ("zotero", Severity::Info),
                ("never_synced", Severity::Warning)
            ]
        );
        assert!(findings[0].message.contains("/data/bib.db"));
    }

    #[test]
    fn unreachable_zotero_is_a_warning() {
        let conn = db();
        let mut input = input(&conn, None);
        input.zotero = Err("Zotero is not reachable at http://127.0.0.1:23119: refused".to_string());
        let findings = run(&input).unwrap();
        assert!(codes(&findings).contains(&("zotero_unreachable", Severity::Warning)));
    }

    #[test]
    fn healthy_library_only_has_infos() {
        let conn = db();
        synced_user_library(&conn);
        add_source(&conn, "U1", Some("dwork2006"), "Differential Privacy", 2006);
        let findings = run(&input(&conn, None)).unwrap();
        assert!(findings.iter().all(|f| f.severity == Severity::Info), "{findings:?}");
        let library = findings.iter().find(|f| f.code == "library").unwrap();
        assert!(
            library
                .message
                .starts_with("user \"My Library\": 1 active sources, last sync"),
            "{}",
            library.message
        );
    }

    #[test]
    fn library_problems_are_warnings() {
        let conn = db();
        synced_user_library(&conn);
        add_source(&conn, "NOKEY001", None, "Untitled", 2020);
        add_source(&conn, "U1", Some("dwork2006"), "Differential Privacy", 2006);
        conn.execute(
            "INSERT INTO attachment (library_id, item_key, source_id, content_type, link_mode, path)
             VALUES (1, 'A1', 2, 'application/pdf', 'imported_url', '/nonexistent/dwork.pdf'),
                    (1, 'A2', 2, 'application/pdf', 'imported_url', NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO review_queue (kind, message, dedupe_key) VALUES ('x', 'm', 'x:1')",
            [],
        )
        .unwrap();
        let findings = run(&input(&conn, None)).unwrap();
        let found = codes(&findings);
        for expected in [
            ("missing_citation_key", Severity::Warning),
            ("missing_pdf_file", Severity::Warning),
            ("pdf_without_file", Severity::Info),
            ("review_open", Severity::Warning),
        ] {
            assert!(found.contains(&expected), "{expected:?} in {found:?}");
        }
        let missing = findings.iter().find(|f| f.code == "missing_citation_key").unwrap();
        assert!(missing.message.contains("user:NOKEY001"), "{}", missing.message);
    }

    fn project_config(bib: Option<&str>) -> ProjectConfig {
        let mut text = "id = \"p1\"\nname = \"Test\"\n[zotero]\nlibraries = [\"user\", \"group:42\"]\n".to_string();
        if let Some(path) = bib {
            text.push_str(&format!("[bibliography]\npath = \"{path}\"\n"));
        }
        ProjectConfig::parse(&text).unwrap()
    }

    #[test]
    fn project_checks_libraries_conflicts_and_bib_file() {
        let conn = db();
        synced_user_library(&conn);
        add_source(&conn, "U1", Some("dwork2006"), "Differential Privacy", 2006);
        add_source(&conn, "U2", Some("smith2020"), "Graph Databases", 2020);
        add_source(&conn, "U3", Some("smith2020"), "Protein Folding", 2020);
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("refs.bib"),
            "@article{dwork2006, title = {Differential Privacy}, year = {2005}}\n@article{ghost2019, title = {Not in Zotero}}\n",
        )
        .unwrap();
        let config = project_config(Some("refs.bib"));

        for _ in 0..2 {
            let findings = run(&input(
                &conn,
                Some(Project {
                    root: dir.path(),
                    config: &config,
                }),
            ))
            .unwrap();
            let found = codes(&findings);
            for expected in [
                ("project", Severity::Info),
                ("library_unknown", Severity::Warning),
                ("key_conflict", Severity::Warning),
                ("bib_only", Severity::Warning),
                ("metadata_drift", Severity::Warning),
            ] {
                assert!(found.contains(&expected), "{expected:?} in {found:?}");
            }
            let unknown = findings.iter().find(|f| f.code == "library_unknown").unwrap();
            assert!(unknown.message.contains("group:42"));
        }
        let queued: i64 = conn
            .query_row(
                "SELECT count(*) FROM review_queue WHERE kind = 'key_conflict'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(queued, 1, "conflict is queued once across runs");
    }

    #[test]
    fn unreadable_bib_file_is_an_error() {
        let conn = db();
        let dir = tempfile::tempdir().unwrap();
        let config = project_config(Some("missing.bib"));
        let findings = run(&input(
            &conn,
            Some(Project {
                root: dir.path(),
                config: &config,
            }),
        ))
        .unwrap();
        assert!(codes(&findings).contains(&("bib_unreadable", Severity::Error)));
    }

    #[test]
    fn ages() {
        assert_eq!(format_age(-5), "0 s");
        assert_eq!(format_age(42), "42 s");
        assert_eq!(format_age(300), "5 min");
        assert_eq!(format_age(3 * 3600 + 5), "3 h");
        assert_eq!(format_age(12 * 86400), "12 d");
    }
}
