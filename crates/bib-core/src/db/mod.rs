//! Central SQLite database: opening, migrations, backups (spec §5, §13).

mod backup;
mod migrate;

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;

pub use backup::{KEEP_BACKUPS, backup, prune_backups};
pub use migrate::{LATEST_VERSION, MIGRATIONS, migrate, schema_version};
pub use rusqlite::Connection;

/// What `open` did besides opening.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub from_version: u32,
    pub to_version: u32,
    /// Backup written before migrating an existing database.
    pub backup: Option<PathBuf>,
}

/// Opens or creates the database, enables WAL and foreign keys, and migrates to the latest schema.
pub fn open(path: &Path, backup_dir: &Path) -> anyhow::Result<(Connection, Opened)> {
    open_with_migrations(path, backup_dir, MIGRATIONS)
}

/// Like [`open`] with an explicit migration list. An existing database with pending migrations is backed up
/// into `backup_dir` first.
pub fn open_with_migrations(
    path: &Path,
    backup_dir: &Path,
    migrations: &[&str],
) -> anyhow::Result<(Connection, Opened)> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut conn = Connection::open(path).with_context(|| format!("opening {}", path.display()))?;
    conn.busy_timeout(Duration::from_secs(5))?;
    let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    anyhow::ensure!(
        mode.eq_ignore_ascii_case("wal"),
        "could not enable WAL (journal_mode = {mode})"
    );
    conn.pragma_update(None, "foreign_keys", true)?;

    let from_version = schema_version(&conn)?;
    let pending = from_version > 0 && (from_version as usize) < migrations.len();
    let backup = if pending {
        let file = backup(&conn, backup_dir)?;
        prune_backups(backup_dir, KEEP_BACKUPS)?;
        Some(file)
    } else {
        None
    };
    let to_version = migrate(&mut conn, migrations)?;
    Ok((
        conn,
        Opened {
            from_version,
            to_version,
            backup,
        },
    ))
}

#[cfg(test)]
mod open_tests {
    use super::*;

    const TEST_MIGRATIONS: &[&str] = &["CREATE TABLE a (x INTEGER);", "CREATE TABLE b (y INTEGER);"];

    #[test]
    fn creates_database_with_latest_schema_wal_and_foreign_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data/bib.db");
        let (conn, opened) = open(&path, &dir.path().join("backups")).unwrap();
        assert_eq!(
            opened,
            Opened {
                from_version: 0,
                to_version: LATEST_VERSION,
                backup: None
            }
        );
        let mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
        assert_eq!(mode, "wal");
        let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn reopening_a_current_database_makes_no_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bib.db");
        let backups = dir.path().join("backups");
        drop(open(&path, &backups).unwrap());
        let (_, opened) = open(&path, &backups).unwrap();
        assert_eq!(opened.from_version, LATEST_VERSION);
        assert_eq!(opened.backup, None);
        assert!(!backups.exists());
    }

    #[test]
    fn pending_migrations_are_preceded_by_a_backup_of_the_old_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bib.db");
        let backups = dir.path().join("backups");
        drop(open_with_migrations(&path, &backups, &TEST_MIGRATIONS[..1]).unwrap());
        let (_, opened) = open_with_migrations(&path, &backups, TEST_MIGRATIONS).unwrap();
        assert_eq!((opened.from_version, opened.to_version), (1, 2));
        let backup = opened.backup.expect("backup written");
        let copy = Connection::open(&backup).unwrap();
        assert_eq!(schema_version(&copy).unwrap(), 1);
    }

    #[test]
    fn prune_keeps_the_newest_backups_and_ignores_other_files() {
        let dir = tempfile::tempdir().unwrap();
        for day in 10..22 {
            std::fs::write(dir.path().join(format!("bib-202609{day}T120000Z-v1.db")), b"").unwrap();
        }
        std::fs::write(dir.path().join("notes.txt"), b"").unwrap();
        let removed = prune_backups(dir.path(), 10).unwrap();
        let names: Vec<_> = removed
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, ["bib-20260910T120000Z-v1.db", "bib-20260911T120000Z-v1.db"]);
        assert!(dir.path().join("notes.txt").exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 11);
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;

    fn conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn
    }

    fn insert_source(conn: &Connection, library_id: i64, key: &str) -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO source (library_id, item_key, item_type, data, data_hash) VALUES (?1, ?2, 'book', '{}', 'h')",
            rusqlite::params![library_id, key],
        )
    }

    #[test]
    fn all_tables_exist() {
        let conn = conn();
        for table in [
            "library",
            "source",
            "source_tag",
            "collection",
            "source_collection",
            "attachment",
            "sync_run",
            "review_queue",
            "llm_call",
        ] {
            let n: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "table {table}");
        }
        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
    }

    #[test]
    fn source_identity_is_library_plus_item_key() {
        let conn = conn();
        conn.execute(
            "INSERT INTO library (kind, zotero_id) VALUES ('user', 0), ('group', 7)",
            [],
        )
        .unwrap();
        insert_source(&conn, 1, "AAAA1111").unwrap();
        insert_source(&conn, 2, "AAAA1111").unwrap();
        assert!(insert_source(&conn, 1, "AAAA1111").is_err());
    }

    #[test]
    fn foreign_keys_and_checks_are_enforced() {
        let conn = conn();
        assert!(
            insert_source(&conn, 99, "X").is_err(),
            "unknown library must be rejected"
        );
        conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('user', 0)", [])
            .unwrap();
        insert_source(&conn, 1, "X").unwrap();
        assert!(conn.execute("UPDATE source SET status = 'deleted'", []).is_err());
        assert!(
            conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('team', 1)", [])
                .is_err()
        );
    }

    #[test]
    fn tags_are_deleted_with_their_source() {
        let conn = conn();
        conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('user', 0)", [])
            .unwrap();
        insert_source(&conn, 1, "X").unwrap();
        conn.execute("INSERT INTO source_tag (source_id, tag) VALUES (1, 'A_core')", [])
            .unwrap();
        conn.execute("DELETE FROM source WHERE id = 1", []).unwrap();
        let n: i64 = conn
            .query_row("SELECT count(*) FROM source_tag", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
    }
}
