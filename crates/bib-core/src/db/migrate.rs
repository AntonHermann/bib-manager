//! Versioned schema migrations tracked in `PRAGMA user_version`.

use anyhow::{Context, bail};
use rusqlite::Connection;

/// Migration `i` brings the schema from version `i` to `i + 1`. Never edit a released migration; add a new one.
pub const MIGRATIONS: &[&str] = &[include_str!("migrations/0001_initial.sql")];
pub const LATEST_VERSION: u32 = MIGRATIONS.len() as u32;

pub fn schema_version(conn: &Connection) -> anyhow::Result<u32> {
    Ok(conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

/// Applies all pending migrations, each in its own transaction, and returns the resulting version.
pub fn migrate(conn: &mut Connection, migrations: &[&str]) -> anyhow::Result<u32> {
    let current = schema_version(conn)?;
    let latest = migrations.len() as u32;
    if current > latest {
        bail!("database schema version {current} is newer than this program supports ({latest})");
    }
    for (index, sql) in migrations.iter().enumerate().skip(current as usize) {
        let version = index as u32 + 1;
        let tx = conn.transaction()?;
        tx.execute_batch(sql).with_context(|| format!("migration {version}"))?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    schema_version(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MIGRATIONS: &[&str] = &["CREATE TABLE a (x INTEGER);", "CREATE TABLE b (y INTEGER);"];

    #[test]
    fn applies_pending_migrations_in_order() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert_eq!(migrate(&mut conn, &TEST_MIGRATIONS[..1]).unwrap(), 1);
        assert_eq!(migrate(&mut conn, TEST_MIGRATIONS).unwrap(), 2);
        conn.execute("INSERT INTO b (y) VALUES (1)", []).unwrap();
    }

    #[test]
    fn migrating_twice_is_a_no_op() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, TEST_MIGRATIONS).unwrap();
        assert_eq!(migrate(&mut conn, TEST_MIGRATIONS).unwrap(), 2);
    }

    #[test]
    fn failing_migration_rolls_back_and_keeps_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        let broken = [
            "CREATE TABLE a (x INTEGER);",
            "CREATE TABLE c (z INTEGER); THIS IS NOT SQL;",
        ];
        assert!(migrate(&mut conn, &broken).is_err());
        assert_eq!(schema_version(&conn).unwrap(), 1);
        let c_exists: i64 = conn
            .query_row("SELECT count(*) FROM sqlite_master WHERE name = 'c'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(c_exists, 0);
    }

    #[test]
    fn newer_database_is_rejected() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 5).unwrap();
        let err = migrate(&mut conn, TEST_MIGRATIONS).unwrap_err();
        assert!(err.to_string().contains("newer"), "{err}");
    }
}
