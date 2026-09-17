//! Consistent database copies via `VACUUM INTO` (spec §5).

use std::path::{Path, PathBuf};

use anyhow::Context;
use rusqlite::Connection;

/// Number of backups kept by `prune_backups` callers.
pub const KEEP_BACKUPS: usize = 10;

/// Writes a copy of the database into `dir` as `bib-<UTC timestamp>-v<schema version>.db` and returns its path.
pub fn backup(conn: &Connection, dir: &Path) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let stamp: String = conn.query_row("SELECT strftime('%Y%m%dT%H%M%SZ', 'now')", [], |row| row.get(0))?;
    let version = super::schema_version(conn)?;
    let mut path = dir.join(format!("bib-{stamp}-v{version}.db"));
    let mut n = 1;
    while path.exists() {
        path = dir.join(format!("bib-{stamp}-v{version}-{n}.db"));
        n += 1;
    }
    let target = path.to_str().context("backup path is not valid UTF-8")?;
    conn.execute("VACUUM INTO ?1", [target]).context("VACUUM INTO")?;
    Ok(path)
}

/// Deletes all but the `keep` newest backups (file names sort by time). Returns the deleted paths.
pub fn prune_backups(dir: &Path, keep: usize) -> anyhow::Result<Vec<PathBuf>> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("bib-") && n.ends_with(".db"))
        })
        .collect();
    backups.sort();
    let excess = backups.len().saturating_sub(keep);
    let removed: Vec<PathBuf> = backups.drain(..excess).collect();
    for path in &removed {
        std::fs::remove_file(path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(removed)
}
