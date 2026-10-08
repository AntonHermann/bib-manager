# Steps 1–2: Data Model and Zotero Sync — Implementation Plan

**Status: Completed — historical plan.** Spec steps 1–2 were implemented and
merged in
[`8aa96e4`](https://github.com/AntonHermann/bib-manager/commit/8aa96e4e2fda26bed081f1b4ff8c5fe3f75016f8)
(2026-09-21). The [smoke-test report](../../research/15-zotero-sync-smoke-test.md)
records the real-Zotero check and its limitations.

The original plan below is preserved as history, not current instructions.
Its agent directives, checkboxes, and follow-up schedule do not authorize
re-execution. Consult the [ADR register](../../decisions/README.md),
[current spec](../specs/2026-09-16-library-core-typst-design.md), and
[deferred work](../../deferred-work.md) for current truth.

The durable points from "Decisions made while writing this plan" now have
current homes:

- Scope and schema breadth: the spec's status, [§5](../specs/2026-09-16-library-core-typst-design.md#5-data-model),
  and [§15](../specs/2026-09-16-library-core-typst-design.md#15-implementation-order).
- Endpoint behavior and the local user-library convention:
  [§6](../specs/2026-09-16-library-core-typst-design.md#6-zotero-sync).
- Open-only review deduplication:
  [§13](../specs/2026-09-16-library-core-typst-design.md#13-error-handling).
- Deferred offline fallback, cited-key comparison, and project persistence:
  [spec follow-ups](../../deferred-work.md#spec-follow-ups).
- Formatting: [rustfmt.toml](../../../rustfmt.toml) is the current setting;
  the one-time reformat is historical, not a recurring task.

---

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A central SQLite database with versioned migrations and backups, a read-only full sync of all Zotero libraries (user + groups) into it, citation-key resolution across a project's libraries, an in-memory comparison against the project's `.bib` export, and the CLI commands `bib init`, `bib sync`, `bib doctor`, `bib backup`.

**Architecture:** `bib-core` gains `paths`, `db` (open, migrations via `PRAGMA user_version`, `VACUUM INTO` backups), `review` (review queue), `library` (`"user"`/`"group:<id>"`), `project` (`bib.toml`), `zotero` (local API client, model, sync, and a fake server for tests), `resolve` (citation keys across libraries), `bibfile` (`.bib` via `biblatex`), `doctor` (checks). A new crate `bib-cli` builds the binary `bib`. Everything network-facing is tested against an in-process fake of the Zotero local API; one final task runs a read-only smoke test against the real Zotero.

**Tech Stack:** Rust 2024 (rustc 1.95), `rusqlite` 0.40 (`bundled`), `ureq` 3.4 (`json`), `serde_json` 1, `percent-encoding` 2.3, `biblatex` 0.12, `uuid` 1 (`v4`), `sha2` 0.11, `toml` 1.1, `serde`, `anyhow`, `thiserror`, `clap` 4.6, `tempfile` 3.27.

**Spec:** `docs/superpowers/specs/2026-09-16-library-core-typst-design.md` (§2, §5, §6, §10, §12, §13, §15 steps 1–2, §18)

**Follow-up plans:** Plan 3 = spec step 3 (extraction with cascade, `bib index`/`text`/`search`), including the two open points from step 0: measuring whether the suspicion detector fires where `pdf_oxide` silently loses ε, and a synthetic coordinate fixture (offset MediaBox, CropBox, `/Rotate 90`) with the rotated-vs-unrotated span decision.

## Decisions made while writing this plan

- **Scope:** spec steps 1 and 2 only (user decision). The `zotero.sqlite` fallback (§6) is deferred (user decision); without Zotero the tool works from the last synced state and shows its age.
- **Schema breadth:** migration 1 creates only the tables steps 1–2 need plus `review_queue` and `llm_call` (spec §12 requires `llm_call` to exist). Anchors, excerpts, text layers, projects, documents, citations and usages arrive with the plans that use them, as further migrations.
- **Zotero endpoints** (measured 2026-09-17): `/items` returns every item type (603 rows: sources, attachments, notes, annotations) while `/items/top` returns top-level items (168). Sources come from `/items/top` (skipping notes, attachments, annotations); PDFs from `/items?itemType=attachment`, joined via `data.parentItem`; the file path only exists as the percent-encoded `file://` URL in `links.enclosure.href` (`data.path` is `null` for stored files). Pages hold at most 100 items; `Total-Results` gives the total.
- **User library** is stored as kind `user`, `zotero_id = 0` (the local API addresses it as `/users/0`).
- **Review queue deduplication** only applies to open items (partial unique index), so a problem that returns after being resolved is queued again.
- **`.bib` comparison** in this plan reports "only in `.bib`" and metadata drift. "Only in Zotero, citation won't compile" needs the cited keys and comes with the Typst parser (spec step 5).
- **Formatting:** `rustfmt.toml` with `max_width = 120`; the existing code is reformatted once in Task 1.
- **Projects in the database** (`project` table, detecting a project ID at two paths, spec §10) come with the Typst parser plan; in this plan `bib.toml` is only read from disk.

## Global Constraints

- License `MIT OR Apache-2.0`. No AGPL dependency in the workspace.
- Rust edition 2024, `rust-version = "1.95"`.
- **All repository content is English:** code comments, doc comments, user-facing strings, docs, commit messages.
- Zotero is accessed **read-only**: only HTTP GET against the local API. Never write to Zotero.
- Tests must never touch the real database: every test sets `BIB_DATA_DIR` to a temporary directory or uses an in-memory connection. Tests never contact the real Zotero; they use `bib_core::zotero::fake::FakeZotero`.
- Database: single file `bib.db` in the data directory (`$BIB_DATA_DIR`, else `$XDG_DATA_HOME/bib`, else `~/.local/share/bib`), WAL mode, foreign keys on, migrations tracked in `PRAGMA user_version`, automatic backup before migrating an existing database.
- Zotero base URL: `$BIB_ZOTERO_URL`, default `http://127.0.0.1:23119`.
- Do not read or use `~/Documents/seminar_ehr_ss26` (graded work, off limits until after 2026-09-23).
- Do not change the Zed installation or `~/.config/zed`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` must pass at the end of every task.
- Code snippets in this plan are not pre-formatted: run `cargo fmt` before checking and committing.
- Every commit message ends with:
  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE
  ```

## File structure

```
rustfmt.toml                                   max_width = 120 (Task 1)
Cargo.toml                                     new workspace deps, member crates/bib-cli
crates/bib-core/
  Cargo.toml                                   deps, feature `test-support`
  src/lib.rs                                   module list
  src/paths.rs                                 data dir, db path, backup dir (Task 2)
  src/db/mod.rs                                open, pragmas, Opened (Task 2)
  src/db/migrate.rs                            MIGRATIONS, migrate, schema_version (Task 2)
  src/db/backup.rs                             backup, prune_backups (Task 2)
  src/db/migrations/0001_initial.sql           schema version 1 (Task 2)
  src/review.rs                                enqueue, open_count (Task 2)
  src/library.rs                               LibraryRef (Task 3)
  src/project.rs                               ProjectConfig, init_project, find_project_root (Task 3)
  src/zotero/mod.rs                            re-exports (Task 4)
  src/zotero/model.rs                          Item, Attachment, Collection, Group, parsers (Task 4)
  src/zotero/client.rs                         ZoteroClient, ZoteroError, paging (Task 4)
  src/zotero/fake.rs                           FakeZotero + JSON builders, cfg(test | test-support) (Task 4)
  src/zotero/sync.rs                           sync_all, sync_library, data_hash (Task 5)
  src/resolve.rs                               SourceRow, resolve_key, same_work, keyed_sources, conflicts (Task 6)
  src/bibfile.rs                               parse_bib, compare (Task 6)
  src/doctor.rs                                Finding, run, format_age (Task 7)
crates/bib-cli/
  Cargo.toml                                   binary `bib` (Task 8)
  src/main.rs                                  init, sync, doctor, backup (Task 8)
  tests/cli.rs                                 end-to-end against FakeZotero (Task 8)
docs/research/15-zotero-sync-smoke-test.md     real-Zotero smoke test (Task 9)
```

---

### Task 1: Formatting baseline

**Files:**
- Create: `rustfmt.toml`
- Modify: every `*.rs` file that `cargo fmt` reformats (no behavior change)

**Interfaces:**
- Produces: `cargo fmt --check` passes for the whole workspace; later tasks keep it passing.

- [ ] **Step 1: Add the rustfmt configuration**

`rustfmt.toml`:

```toml
max_width = 120
```

- [ ] **Step 2: Reformat**

Run: `cargo fmt --all && cargo fmt --all --check`
Expected: second command prints nothing and exits 0.

Also run: `cargo fmt --check --manifest-path spikes/zed-coexist-ext/Cargo.toml` — if it reports diffs, run `cargo fmt --manifest-path spikes/zed-coexist-ext/Cargo.toml`. The probes under `docs/research/probes` are historical and stay untouched.

- [ ] **Step 3: Verify nothing changed in behavior**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS (same test count as before: 39).

- [ ] **Step 4: Commit**

```bash
git add rustfmt.toml crates tools spikes
git commit -m "Add rustfmt configuration and reformat workspace

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 2: Database — paths, migrations, schema, backups, review queue

**Files:**
- Modify: `Cargo.toml` (workspace deps), `crates/bib-core/Cargo.toml`, `crates/bib-core/src/lib.rs`
- Create: `crates/bib-core/src/paths.rs`, `crates/bib-core/src/db/mod.rs`, `crates/bib-core/src/db/migrate.rs`, `crates/bib-core/src/db/backup.rs`, `crates/bib-core/src/db/migrations/0001_initial.sql`, `crates/bib-core/src/review.rs`

**Interfaces:**
- Produces:
  - `bib_core::paths::{DATA_DIR_ENV, data_dir() -> PathBuf, db_path() -> PathBuf, backup_dir() -> PathBuf}`
  - `bib_core::db::{open(path: &Path, backup_dir: &Path) -> anyhow::Result<(Connection, Opened)>, open_with_migrations(path, backup_dir, migrations: &[&str]) -> anyhow::Result<(Connection, Opened)>, Opened { from_version: u32, to_version: u32, backup: Option<PathBuf> }, MIGRATIONS: &[&str], LATEST_VERSION: u32, migrate(conn: &mut Connection, migrations: &[&str]) -> anyhow::Result<u32>, schema_version(conn: &Connection) -> anyhow::Result<u32>, backup(conn: &Connection, dir: &Path) -> anyhow::Result<PathBuf>, prune_backups(dir: &Path, keep: usize) -> anyhow::Result<Vec<PathBuf>>, KEEP_BACKUPS: usize, Connection}` (`Connection` re-exported from rusqlite)
  - `bib_core::review::{NewReviewItem<'a> { kind: &'a str, node_type: Option<&'a str>, node_id: Option<i64>, message: &'a str, details: &'a serde_json::Value, dedupe_key: &'a str }, enqueue(conn: &Connection, item: &NewReviewItem) -> rusqlite::Result<bool>, open_count(conn: &Connection) -> rusqlite::Result<i64>}`
  - Tables (schema version 1): `library`, `source`, `source_tag`, `collection`, `source_collection`, `attachment`, `sync_run`, `review_queue`, `llm_call` — exact columns in the SQL below; later tasks write to them.

- [ ] **Step 1: Dependencies**

`Cargo.toml` → `[workspace.dependencies]` add:

```toml
rusqlite = { version = "0.40", features = ["bundled"] }
serde_json = "1"
```

`crates/bib-core/Cargo.toml` becomes:

```toml
[package]
name = "bib-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
anyhow.workspace = true
rusqlite.workspace = true
serde_json.workspace = true
unicode-normalization.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

`crates/bib-core/src/lib.rs`:

```rust
//! Core of the bibliography manager.

pub mod db;
pub mod normalize;
pub mod paths;
pub mod review;
```

- [ ] **Step 2: Data paths with tests**

`crates/bib-core/src/paths.rs`:

```rust
//! Locations of the central database and its backups (spec §5).

use std::ffi::OsString;
use std::path::PathBuf;

/// Environment variable that overrides the data directory (tests, experiments).
pub const DATA_DIR_ENV: &str = "BIB_DATA_DIR";

/// `$BIB_DATA_DIR`, else `$XDG_DATA_HOME/bib`, else `~/.local/share/bib`.
pub fn data_dir() -> PathBuf {
    data_dir_from(|name| std::env::var_os(name))
}

pub fn db_path() -> PathBuf {
    data_dir().join("bib.db")
}

pub fn backup_dir() -> PathBuf {
    data_dir().join("backups")
}

fn data_dir_from(var: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    if let Some(dir) = var(DATA_DIR_ENV).filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    if let Some(xdg) = var("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(xdg).join("bib");
    }
    let home = var("HOME").filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/share/bib")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let map: HashMap<String, OsString> = pairs.iter().map(|(k, v)| (k.to_string(), OsString::from(v))).collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn override_wins() {
        let dir = data_dir_from(env(&[("BIB_DATA_DIR", "/tmp/x"), ("XDG_DATA_HOME", "/xdg"), ("HOME", "/home/a")]));
        assert_eq!(dir, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn xdg_then_home() {
        assert_eq!(data_dir_from(env(&[("XDG_DATA_HOME", "/xdg"), ("HOME", "/home/a")])), PathBuf::from("/xdg/bib"));
        assert_eq!(data_dir_from(env(&[("HOME", "/home/a")])), PathBuf::from("/home/a/.local/share/bib"));
    }

    #[test]
    fn empty_values_are_ignored() {
        let dir = data_dir_from(env(&[("BIB_DATA_DIR", ""), ("XDG_DATA_HOME", ""), ("HOME", "/h")]));
        assert_eq!(dir, PathBuf::from("/h/.local/share/bib"));
    }
}
```

Run: `cargo test -p bib-core paths`
Expected: PASS, 3 tests (the module is small enough that test and implementation are written together; the failing state is the missing module until `lib.rs` lists it).

- [ ] **Step 3: Migration runner tests**

`crates/bib-core/src/db/migrate.rs` (tests first, implementation stubbed):

```rust
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
    let _ = (conn, migrations);
    todo!()
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
        let broken = ["CREATE TABLE a (x INTEGER);", "CREATE TABLE c (z INTEGER); THIS IS NOT SQL;"];
        assert!(migrate(&mut conn, &broken).is_err());
        assert_eq!(schema_version(&conn).unwrap(), 1);
        let c_exists: i64 =
            conn.query_row("SELECT count(*) FROM sqlite_master WHERE name = 'c'", [], |r| r.get(0)).unwrap();
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
```

Create `crates/bib-core/src/db/migrations/0001_initial.sql` with the schema from Step 5 now (the `include_str!` needs the file), and `crates/bib-core/src/db/mod.rs` with just:

```rust
//! Central SQLite database: opening, migrations, backups (spec §5, §13).

mod migrate;

pub use migrate::{LATEST_VERSION, MIGRATIONS, migrate, schema_version};
pub use rusqlite::Connection;
```

Run: `cargo test -p bib-core db::migrate`
Expected: FAIL, tests panic with `not yet implemented`.

- [ ] **Step 4: Implement `migrate`**

Replace the stub:

```rust
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
```

Run: `cargo test -p bib-core db::migrate`
Expected: PASS, 4 tests.

- [ ] **Step 5: Schema version 1**

`crates/bib-core/src/db/migrations/0001_initial.sql`:

```sql
-- Schema version 1: Zotero libraries and sources, sync runs, review queue, LLM call log (spec §5, §6, §12).

CREATE TABLE library (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('user', 'group')),
    -- 0 for the local user library (addressed as /users/0 by the local API), the group id otherwise.
    zotero_id INTEGER NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    UNIQUE (kind, zotero_id)
);

CREATE TABLE source (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    item_key TEXT NOT NULL,
    citation_key TEXT,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    year INTEGER,
    doi TEXT,
    -- Zotero item `data` object as JSON; `data_hash` is SHA-256 of its canonical form without `version`.
    data TEXT NOT NULL,
    data_hash TEXT NOT NULL,
    date_modified TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retired')),
    first_seen TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_seen TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE (library_id, item_key)
);
CREATE INDEX source_citation_key ON source (citation_key);

CREATE TABLE source_tag (
    source_id INTEGER NOT NULL REFERENCES source (id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (source_id, tag)
);

CREATE TABLE collection (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    collection_key TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_key TEXT,
    UNIQUE (library_id, collection_key)
);

CREATE TABLE source_collection (
    source_id INTEGER NOT NULL REFERENCES source (id) ON DELETE CASCADE,
    collection_id INTEGER NOT NULL REFERENCES collection (id) ON DELETE CASCADE,
    PRIMARY KEY (source_id, collection_id)
);

CREATE TABLE attachment (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    item_key TEXT NOT NULL,
    -- NULL for standalone attachments without a parent source.
    source_id INTEGER REFERENCES source (id),
    content_type TEXT NOT NULL,
    link_mode TEXT NOT NULL,
    -- NULL when Zotero has no local file.
    path TEXT,
    -- Filled by extraction (plan 3); reset when the path changes.
    checksum TEXT,
    page_count INTEGER,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retired')),
    UNIQUE (library_id, item_key)
);
CREATE INDEX attachment_source ON attachment (source_id);

CREATE TABLE sync_run (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'ok', 'failed')),
    error TEXT,
    sources_new INTEGER NOT NULL DEFAULT 0,
    sources_changed INTEGER NOT NULL DEFAULT 0,
    sources_retired INTEGER NOT NULL DEFAULT 0,
    sources_reactivated INTEGER NOT NULL DEFAULT 0,
    attachments INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE review_queue (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    -- Node convention (spec §5): table name plus row id of the affected entry.
    node_type TEXT,
    node_id INTEGER,
    message TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    dedupe_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    resolved_at TEXT,
    resolution TEXT
);
-- The same open problem is queued only once; after resolution it may be queued again.
CREATE UNIQUE INDEX review_queue_open_dedupe ON review_queue (dedupe_key) WHERE resolved_at IS NULL;

-- Empty in this version; every later LLM feature must log here (spec §12).
CREATE TABLE llm_call (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    purpose TEXT NOT NULL,
    model TEXT NOT NULL,
    model_version TEXT NOT NULL,
    parameters TEXT NOT NULL,
    prompt TEXT NOT NULL,
    input_checksum TEXT NOT NULL,
    policy_checksum TEXT NOT NULL,
    output TEXT NOT NULL,
    cloud INTEGER NOT NULL CHECK (cloud IN (0, 1)),
    -- For cloud calls: which of the user's own text left the device.
    data_sent TEXT
);
```

- [ ] **Step 6: Review queue with tests**

`crates/bib-core/src/review.rs`:

```rust
//! Review queue: everything a human needs to look at (spec §5, §13). Nothing unclear is silently dropped.

use rusqlite::{Connection, params};

pub struct NewReviewItem<'a> {
    pub kind: &'a str,
    pub node_type: Option<&'a str>,
    pub node_id: Option<i64>,
    pub message: &'a str,
    pub details: &'a serde_json::Value,
    /// Identifies the problem; an open item with the same key is not queued twice.
    pub dedupe_key: &'a str,
}

/// Queues an item unless an open item with the same `dedupe_key` exists. Returns whether a new item was added.
pub fn enqueue(conn: &Connection, item: &NewReviewItem) -> rusqlite::Result<bool> {
    let inserted = conn.execute(
        "INSERT INTO review_queue (kind, node_type, node_id, message, details, dedupe_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (dedupe_key) WHERE resolved_at IS NULL DO NOTHING",
        params![item.kind, item.node_type, item.node_id, item.message, item.details.to_string(), item.dedupe_key],
    )?;
    Ok(inserted == 1)
}

pub fn open_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM review_queue WHERE resolved_at IS NULL", [], |row| row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{MIGRATIONS, migrate};

    fn conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn
    }

    fn item<'a>(details: &'a serde_json::Value) -> NewReviewItem<'a> {
        NewReviewItem {
            kind: "test",
            node_type: Some("source"),
            node_id: Some(1),
            message: "look at this",
            details,
            dedupe_key: "test:1",
        }
    }

    #[test]
    fn open_items_are_deduplicated() {
        let conn = conn();
        let details = serde_json::json!({"a": 1});
        assert!(enqueue(&conn, &item(&details)).unwrap());
        assert!(!enqueue(&conn, &item(&details)).unwrap());
        assert_eq!(open_count(&conn).unwrap(), 1);
    }

    #[test]
    fn resolved_problem_can_be_queued_again() {
        let conn = conn();
        let details = serde_json::json!({});
        enqueue(&conn, &item(&details)).unwrap();
        conn.execute("UPDATE review_queue SET resolved_at = 'now', resolution = 'fixed'", []).unwrap();
        assert!(enqueue(&conn, &item(&details)).unwrap());
        assert_eq!(open_count(&conn).unwrap(), 1);
    }
}
```

Run: `cargo test -p bib-core review`
Expected: PASS, 2 tests.

- [ ] **Step 7: Schema tests**

Append to `crates/bib-core/src/db/mod.rs` (below the `pub use` lines):

```rust
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
                .query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1", [table], |r| {
                    r.get(0)
                })
                .unwrap();
            assert_eq!(n, 1, "table {table}");
        }
        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
    }

    #[test]
    fn source_identity_is_library_plus_item_key() {
        let conn = conn();
        conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('user', 0), ('group', 7)", []).unwrap();
        insert_source(&conn, 1, "AAAA1111").unwrap();
        insert_source(&conn, 2, "AAAA1111").unwrap();
        assert!(insert_source(&conn, 1, "AAAA1111").is_err());
    }

    #[test]
    fn foreign_keys_and_checks_are_enforced() {
        let conn = conn();
        assert!(insert_source(&conn, 99, "X").is_err(), "unknown library must be rejected");
        conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('user', 0)", []).unwrap();
        insert_source(&conn, 1, "X").unwrap();
        assert!(conn.execute("UPDATE source SET status = 'deleted'", []).is_err());
        assert!(conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('team', 1)", []).is_err());
    }

    #[test]
    fn tags_are_deleted_with_their_source() {
        let conn = conn();
        conn.execute("INSERT INTO library (kind, zotero_id) VALUES ('user', 0)", []).unwrap();
        insert_source(&conn, 1, "X").unwrap();
        conn.execute("INSERT INTO source_tag (source_id, tag) VALUES (1, 'A_core')", []).unwrap();
        conn.execute("DELETE FROM source WHERE id = 1", []).unwrap();
        let n: i64 = conn.query_row("SELECT count(*) FROM source_tag", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }
}
```

Run: `cargo test -p bib-core db::schema_tests`
Expected: PASS, 4 tests.

- [ ] **Step 8: Backups and `open` — tests**

`crates/bib-core/src/db/backup.rs`:

```rust
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
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("bib-") && n.ends_with(".db")))
        .collect();
    backups.sort();
    let excess = backups.len().saturating_sub(keep);
    let removed: Vec<PathBuf> = backups.drain(..excess).collect();
    for path in &removed {
        std::fs::remove_file(path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(removed)
}
```

Replace `crates/bib-core/src/db/mod.rs` above the `schema_tests` module with:

```rust
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
    let _ = (path, backup_dir, migrations, Duration::ZERO);
    todo!()
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
        assert_eq!(opened, Opened { from_version: 0, to_version: LATEST_VERSION, backup: None });
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
        let names: Vec<_> = removed.iter().map(|p| p.file_name().unwrap().to_str().unwrap().to_string()).collect();
        assert_eq!(names, ["bib-20260910T120000Z-v1.db", "bib-20260911T120000Z-v1.db"]);
        assert!(dir.path().join("notes.txt").exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 11);
    }
}
```

Run: `cargo test -p bib-core db::open_tests`
Expected: FAIL: three `open` tests panic with `not yet implemented`; the prune test passes.

- [ ] **Step 9: Implement `open_with_migrations`**

Replace the stub:

```rust
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
    anyhow::ensure!(mode.eq_ignore_ascii_case("wal"), "could not enable WAL (journal_mode = {mode})");
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
    Ok((conn, Opened { from_version, to_version, backup }))
}
```

Run: `cargo test -p bib-core`
Expected: PASS (all bib-core tests including the 12 normalization tests + 1 final-sigma test).

- [ ] **Step 10: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

```bash
git add Cargo.toml Cargo.lock crates/bib-core
git commit -m "Add SQLite database with migrations, backups and review queue

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 3: Library references and project configuration (`bib.toml`)

**Files:**
- Modify: `Cargo.toml`, `crates/bib-core/Cargo.toml`, `crates/bib-core/src/lib.rs`
- Create: `crates/bib-core/src/library.rs`, `crates/bib-core/src/project.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `bib_core::library::LibraryRef { User, Group(i64) }` — `Copy`, `Eq`, `Hash`, `Ord`; `FromStr` (`"user"`, `"group:<id>"` with id > 0) with error `ParseLibraryRefError(String)`; `Display` back to the same strings; `serde::Serialize` as that string; `fn api_prefix(self) -> String` (`"/users/0"`, `"/groups/<id>"`); `fn db_key(self) -> (&'static str, i64)` (`("user", 0)`, `("group", id)`); `fn from_db(kind: &str, zotero_id: i64) -> Option<LibraryRef>`
  - `bib_core::project::{CONFIG_FILE = "bib.toml", ProjectConfig { id: String, name: String, documents: Vec<DocumentConfig>, zotero: ZoteroConfig, bibliography: Option<BibliographyConfig>, ai: AiConfig }, DocumentConfig { path: PathBuf, kind: DocumentKind }, DocumentKind { Paper, Slides }, ZoteroConfig { libraries: Vec<String> }, BibliographyConfig { path: PathBuf, managed_by: String }, AiConfig { allowed: Vec<String>, cloud: bool, logging: String }}`
  - `ProjectConfig::parse(text: &str) -> anyhow::Result<ProjectConfig>`, `ProjectConfig::load(root: &Path) -> anyhow::Result<ProjectConfig>`, `ProjectConfig::libraries(&self) -> anyhow::Result<Vec<LibraryRef>>` (resolution order)
  - `bib_core::project::find_project_root(start: &Path) -> Option<PathBuf>`, `bib_core::project::init_project(dir: &Path) -> anyhow::Result<PathBuf>`

- [ ] **Step 1: Dependencies**

`Cargo.toml` → `[workspace.dependencies]` add:

```toml
uuid = { version = "1", features = ["v4"] }
```

`crates/bib-core/Cargo.toml` → `[dependencies]` add (keep the existing entries, alphabetical):

```toml
serde.workspace = true
thiserror.workspace = true
toml.workspace = true
uuid.workspace = true
```

`crates/bib-core/src/lib.rs`:

```rust
//! Core of the bibliography manager.

pub mod db;
pub mod library;
pub mod normalize;
pub mod paths;
pub mod project;
pub mod review;
```

- [ ] **Step 2: `LibraryRef` with tests**

`crates/bib-core/src/library.rs`:

```rust
//! Zotero library references as written in `bib.toml`: `"user"` or `"group:<id>"` (spec §6).

use std::fmt;
use std::str::FromStr;

/// A Zotero library. Sources are identified by library plus item key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LibraryRef {
    User,
    Group(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid library reference {0:?}: expected \"user\" or \"group:<id>\"")]
pub struct ParseLibraryRefError(pub String);

impl LibraryRef {
    /// Path prefix in the local API.
    pub fn api_prefix(self) -> String {
        match self {
            LibraryRef::User => "/users/0".to_string(),
            LibraryRef::Group(id) => format!("/groups/{id}"),
        }
    }

    /// `(library.kind, library.zotero_id)` in the database.
    pub fn db_key(self) -> (&'static str, i64) {
        match self {
            LibraryRef::User => ("user", 0),
            LibraryRef::Group(id) => ("group", id),
        }
    }

    pub fn from_db(kind: &str, zotero_id: i64) -> Option<LibraryRef> {
        match kind {
            "user" => Some(LibraryRef::User),
            "group" => Some(LibraryRef::Group(zotero_id)),
            _ => None,
        }
    }
}

impl fmt::Display for LibraryRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryRef::User => f.write_str("user"),
            LibraryRef::Group(id) => write!(f, "group:{id}"),
        }
    }
}

impl FromStr for LibraryRef {
    type Err = ParseLibraryRefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "user" {
            return Ok(LibraryRef::User);
        }
        s.strip_prefix("group:")
            .and_then(|id| id.parse::<i64>().ok())
            .filter(|id| *id > 0)
            .map(LibraryRef::Group)
            .ok_or_else(|| ParseLibraryRefError(s.to_string()))
    }
}

impl serde::Serialize for LibraryRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_displays() {
        for text in ["user", "group:6573630"] {
            assert_eq!(text.parse::<LibraryRef>().unwrap().to_string(), text);
        }
        assert_eq!("group:7".parse::<LibraryRef>().unwrap(), LibraryRef::Group(7));
    }

    #[test]
    fn rejects_malformed_references() {
        for text in ["", "users", "group:", "group:abc", "group:-1", "group:0", "Group:7"] {
            assert_eq!(text.parse::<LibraryRef>(), Err(ParseLibraryRefError(text.to_string())), "{text:?}");
        }
    }

    #[test]
    fn maps_to_api_and_database() {
        assert_eq!(LibraryRef::User.api_prefix(), "/users/0");
        assert_eq!(LibraryRef::Group(7).api_prefix(), "/groups/7");
        assert_eq!(LibraryRef::Group(7).db_key(), ("group", 7));
        assert_eq!(LibraryRef::from_db("user", 0), Some(LibraryRef::User));
        assert_eq!(LibraryRef::from_db("team", 1), None);
    }

    #[test]
    fn serializes_as_string() {
        assert_eq!(serde_json::to_string(&LibraryRef::Group(7)).unwrap(), "\"group:7\"");
    }
}
```

Run: `cargo test -p bib-core library`
Expected: PASS, 4 tests.

- [ ] **Step 3: Project configuration tests**

`crates/bib-core/src/project.rs` (types and tests; functions stubbed):

```rust
//! Project configuration `bib.toml` (spec §10). The file is hand-written and versioned with the project.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::library::LibraryRef;

pub const CONFIG_FILE: &str = "bib.toml";

/// Sections not modelled yet (`[export]`, `[diagnostics]`) are accepted and ignored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub documents: Vec<DocumentConfig>,
    #[serde(default)]
    pub zotero: ZoteroConfig,
    pub bibliography: Option<BibliographyConfig>,
    #[serde(default)]
    pub ai: AiConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DocumentConfig {
    pub path: PathBuf,
    pub kind: DocumentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentKind {
    Paper,
    Slides,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ZoteroConfig {
    /// Resolution order for citation keys.
    #[serde(default = "default_libraries")]
    pub libraries: Vec<String>,
}

impl Default for ZoteroConfig {
    fn default() -> Self {
        Self { libraries: default_libraries() }
    }
}

fn default_libraries() -> Vec<String> {
    vec!["user".to_string()]
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BibliographyConfig {
    pub path: PathBuf,
    #[serde(default = "default_managed_by")]
    pub managed_by: String,
}

fn default_managed_by() -> String {
    "zotero".to_string()
}

/// `[ai]` policy (spec §12). Parsed now; enforced once LLM features exist.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub cloud: bool,
    #[serde(default = "default_logging")]
    pub logging: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self { allowed: Vec::new(), cloud: false, logging: default_logging() }
    }
}

fn default_logging() -> String {
    "required".to_string()
}

impl ProjectConfig {
    pub fn parse(text: &str) -> anyhow::Result<ProjectConfig> {
        let _ = text;
        todo!()
    }

    pub fn load(root: &Path) -> anyhow::Result<ProjectConfig> {
        let path = root.join(CONFIG_FILE);
        let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("invalid {}", path.display()))
    }

    /// Libraries in resolution order.
    pub fn libraries(&self) -> anyhow::Result<Vec<LibraryRef>> {
        todo!()
    }
}

/// Nearest directory at or above `start` that contains `bib.toml`.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let _ = start;
    todo!()
}

/// Writes a new `bib.toml` into `dir` and returns its path. Fails if the file exists.
pub fn init_project(dir: &Path) -> anyhow::Result<PathBuf> {
    let _ = dir;
    bail!("not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_EXAMPLE: &str = r#"
id = "0193f2a1-0000-4000-8000-000000000000"
name = "EHR Privacy Seminar"

[[documents]]
path = "paper.typ"
kind = "paper"

[[documents]]
path = "presentation/slides.typ"
kind = "slides"

[zotero]
libraries = ["user", "group:6573630"]

[bibliography]
path = "ehr_privacy.bib"
managed_by = "zotero"

[export]
excerpts = "notes/quote_verification.json"

[diagnostics]
unverified_quote = "error"

[ai]
allowed = ["retrieval", "verification"]
cloud = false
logging = "required"
"#;

    #[test]
    fn parses_the_spec_example() {
        let config = ProjectConfig::parse(SPEC_EXAMPLE).unwrap();
        assert_eq!(config.name, "EHR Privacy Seminar");
        assert_eq!(config.documents.len(), 2);
        assert_eq!(config.documents[1].kind, DocumentKind::Slides);
        assert_eq!(config.libraries().unwrap(), vec![LibraryRef::User, LibraryRef::Group(6573630)]);
        assert_eq!(config.bibliography.unwrap().path, PathBuf::from("ehr_privacy.bib"));
        assert_eq!(config.ai.allowed, ["retrieval", "verification"]);
    }

    #[test]
    fn minimal_file_uses_defaults() {
        let config = ProjectConfig::parse("id = \"x\"\nname = \"n\"\n").unwrap();
        assert_eq!(config.libraries().unwrap(), vec![LibraryRef::User]);
        assert_eq!(config.bibliography, None);
        assert_eq!(config.ai, AiConfig::default());
    }

    #[test]
    fn invalid_library_is_an_error() {
        let err = ProjectConfig::parse("id = \"x\"\nname = \"n\"\n[zotero]\nlibraries = [\"group:abc\"]\n").unwrap_err();
        assert!(format!("{err:#}").contains("group:abc"), "{err:#}");
    }

    #[test]
    fn empty_library_list_and_empty_id_are_errors() {
        assert!(ProjectConfig::parse("id = \"x\"\nname = \"n\"\n[zotero]\nlibraries = []\n").is_err());
        assert!(ProjectConfig::parse("id = \" \"\nname = \"n\"\n").is_err());
    }

    #[test]
    fn init_writes_a_parseable_file_once() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("my \"quoted\" project");
        std::fs::create_dir(&project).unwrap();
        let path = init_project(&project).unwrap();
        let config = ProjectConfig::load(&project).unwrap();
        assert_eq!(config.name, "my \"quoted\" project");
        assert_eq!(config.id.len(), 36);
        assert_eq!(config.libraries().unwrap(), vec![LibraryRef::User]);
        assert!(path.ends_with(CONFIG_FILE));
        assert!(init_project(&project).is_err());
    }

    #[test]
    fn project_root_is_found_from_a_subdirectory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE), "id = \"x\"\nname = \"n\"\n").unwrap();
        let nested = dir.path().join("chapters/a");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(find_project_root(&nested).as_deref(), Some(dir.path()));
        let outside = tempfile::tempdir().unwrap();
        assert_eq!(find_project_root(outside.path()), None);
    }
}
```

Run: `cargo test -p bib-core project`
Expected: FAIL (`not yet implemented` / `not implemented`).

- [ ] **Step 4: Implement**

Replace the stubbed functions:

```rust
impl ProjectConfig {
    pub fn parse(text: &str) -> anyhow::Result<ProjectConfig> {
        let config: ProjectConfig = toml::from_str(text)?;
        if config.id.trim().is_empty() {
            bail!("`id` must not be empty");
        }
        config.libraries()?;
        Ok(config)
    }

    pub fn load(root: &Path) -> anyhow::Result<ProjectConfig> {
        let path = root.join(CONFIG_FILE);
        let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("invalid {}", path.display()))
    }

    pub fn libraries(&self) -> anyhow::Result<Vec<LibraryRef>> {
        let libraries = self.zotero.libraries.iter().map(|s| s.parse()).collect::<Result<Vec<LibraryRef>, _>>()?;
        if libraries.is_empty() {
            bail!("[zotero] libraries must name at least one library");
        }
        Ok(libraries)
    }
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|dir| dir.join(CONFIG_FILE).is_file()).map(Path::to_path_buf)
}

pub fn init_project(dir: &Path) -> anyhow::Result<PathBuf> {
    let path = dir.join(CONFIG_FILE);
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("project");
    let text = skeleton(&uuid::Uuid::new_v4().to_string(), name);
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

fn skeleton(id: &str, name: &str) -> String {
    format!(
        r#"id = "{id}"
name = {name}

# Typst entry points; further files follow from #include.
# [[documents]]
# path = "paper.typ"
# kind = "paper"

[zotero]
# Resolution order for citation keys: "user" and/or "group:<id>".
libraries = ["user"]

# [bibliography]
# path = "references.bib"
# managed_by = "zotero"

[ai]
allowed = []
cloud = false
logging = "required"
"#,
        name = toml_string(name)
    )
}

/// A TOML basic string literal.
fn toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
```

Run: `cargo test -p bib-core project`
Expected: PASS, 6 tests.

- [ ] **Step 5: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

```bash
git add Cargo.toml Cargo.lock crates/bib-core
git commit -m "Add library references and bib.toml project configuration

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 4: Zotero local API — model, client, fake server

**Files:**
- Modify: `Cargo.toml`, `crates/bib-core/Cargo.toml`, `crates/bib-core/src/lib.rs`
- Create: `crates/bib-core/src/zotero/mod.rs`, `crates/bib-core/src/zotero/model.rs`, `crates/bib-core/src/zotero/client.rs`, `crates/bib-core/src/zotero/fake.rs`

**Interfaces:**
- Consumes: `LibraryRef::api_prefix` (Task 3).
- Produces:
  - `bib_core::zotero::Item { key, item_type, citation_key: Option<String>, title, year: Option<i32>, doi: Option<String>, date_modified, tags: Vec<String>, collections: Vec<String>, library_name, data: serde_json::Value }` with `fn is_source(&self) -> bool`
  - `bib_core::zotero::Attachment { key, parent_key: Option<String>, content_type, link_mode, path: Option<PathBuf> }` with `fn is_pdf(&self) -> bool`
  - `bib_core::zotero::Collection { key, name, parent_key: Option<String> }`, `bib_core::zotero::Group { id: i64, name: String }`
  - `bib_core::zotero::{parse_item, parse_attachment, parse_collection, parse_group}(v: &Value) -> Result<_, ZoteroError>`, `file_url_to_path(href: &str) -> Option<PathBuf>`, `year_from_date(date: &str) -> Option<i32>`
  - `bib_core::zotero::{ZoteroClient, ZoteroError, DEFAULT_URL, URL_ENV}`; `ZoteroClient::new(base_url: &str)`, `ZoteroClient::from_env()`, `fn base_url(&self) -> &str`, `fn groups(&self) -> Result<Vec<Group>, ZoteroError>`, `fn top_items(&self, LibraryRef) -> Result<Vec<Item>, ZoteroError>`, `fn attachments(&self, LibraryRef) -> Result<Vec<Attachment>, ZoteroError>`, `fn collections(&self, LibraryRef) -> Result<Vec<Collection>, ZoteroError>`
  - `ZoteroError::{Unreachable { url: String, message: String }, Http { status: u16, url: String }, Parse(String)}`
  - `bib_core::zotero::fake` (only with `cfg(test)` or feature `test-support`): `FakeZotero::start() -> FakeZotero`, `fn url(&self) -> &str`, `fn set(&self, route: &str, items: Vec<Value>)`, `fn user_library(&self, items, attachments, collections)`; JSON builders `item(key, citation_key, title, date, doi) -> Value`, `note(key) -> Value`, `attachment(key, parent, file_url) -> Value`, `collection(key, name, parent: Option<&str>) -> Value`, `group(id, name) -> Value`

- [ ] **Step 1: Dependencies and module skeleton**

`Cargo.toml` → `[workspace.dependencies]` add:

```toml
percent-encoding = "2.3"
ureq = { version = "3.4", features = ["json"] }
```

`crates/bib-core/Cargo.toml`: add to `[dependencies]` `percent-encoding.workspace = true` and `ureq.workspace = true`, and add

```toml
[features]
# Exposes `zotero::fake` to other crates' tests.
test-support = []
```

`crates/bib-core/src/lib.rs` gains `pub mod zotero;` (alphabetical, after `review`).

`crates/bib-core/src/zotero/mod.rs`:

```rust
//! Read-only access to the Zotero local API (spec §6).

mod client;
#[cfg(any(test, feature = "test-support"))]
pub mod fake;
mod model;

pub use client::{DEFAULT_URL, URL_ENV, ZoteroClient, ZoteroError};
pub use model::{
    Attachment, Collection, Group, Item, NON_SOURCE_TYPES, file_url_to_path, parse_attachment, parse_collection,
    parse_group, parse_item, year_from_date,
};
```

- [ ] **Step 2: Model with tests**

`crates/bib-core/src/zotero/model.rs`:

```rust
//! Zotero JSON (API v3, `format=json`) to typed values. Only the fields the tool uses are typed;
//! the complete `data` object of an item is kept.

use std::path::PathBuf;

use serde_json::Value;

use super::ZoteroError;

/// Top-level item types that are not sources.
pub const NON_SOURCE_TYPES: &[&str] = &["attachment", "note", "annotation"];

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub key: String,
    pub item_type: String,
    pub citation_key: Option<String>,
    pub title: String,
    pub year: Option<i32>,
    pub doi: Option<String>,
    pub date_modified: String,
    pub tags: Vec<String>,
    pub collections: Vec<String>,
    /// Library name as reported by Zotero.
    pub library_name: String,
    /// The complete `data` object.
    pub data: Value,
}

impl Item {
    pub fn is_source(&self) -> bool {
        !NON_SOURCE_TYPES.contains(&self.item_type.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attachment {
    pub key: String,
    pub parent_key: Option<String>,
    pub content_type: String,
    pub link_mode: String,
    /// Local file, if Zotero has one.
    pub path: Option<PathBuf>,
}

impl Attachment {
    pub fn is_pdf(&self) -> bool {
        self.content_type == "application/pdf"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Collection {
    pub key: String,
    pub name: String,
    pub parent_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: i64,
    pub name: String,
}

fn data_of<'a>(v: &'a Value, what: &str) -> Result<&'a Value, ZoteroError> {
    v.get("data").filter(|d| d.is_object()).ok_or_else(|| ZoteroError::Parse(format!("{what} without `data` object")))
}

fn text(data: &Value, field: &str) -> String {
    data.get(field).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Trimmed, non-empty string field.
fn non_empty(data: &Value, field: &str) -> Option<String> {
    data.get(field).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn key_of(data: &Value, what: &str) -> Result<String, ZoteroError> {
    non_empty(data, "key").ok_or_else(|| ZoteroError::Parse(format!("{what} without `key`")))
}

fn strings(value: Option<&Value>, pick: impl Fn(&Value) -> Option<&str>) -> Vec<String> {
    value.and_then(Value::as_array).map(|a| a.iter().filter_map(&pick).map(str::to_string).collect()).unwrap_or_default()
}

pub fn parse_item(v: &Value) -> Result<Item, ZoteroError> {
    let data = data_of(v, "item")?;
    Ok(Item {
        key: key_of(data, "item")?,
        item_type: text(data, "itemType"),
        citation_key: non_empty(data, "citationKey"),
        title: text(data, "title"),
        year: year_from_date(&text(data, "date")),
        doi: non_empty(data, "DOI"),
        date_modified: text(data, "dateModified"),
        tags: strings(data.get("tags"), |t| t.get("tag").and_then(Value::as_str)),
        collections: strings(data.get("collections"), Value::as_str),
        library_name: v.pointer("/library/name").and_then(Value::as_str).unwrap_or_default().to_string(),
        data: data.clone(),
    })
}

pub fn parse_attachment(v: &Value) -> Result<Attachment, ZoteroError> {
    let data = data_of(v, "attachment")?;
    Ok(Attachment {
        key: key_of(data, "attachment")?,
        parent_key: non_empty(data, "parentItem"),
        content_type: text(data, "contentType"),
        link_mode: text(data, "linkMode"),
        // `data.path` is null for stored files; the local API exposes the file as the `enclosure` link.
        path: v.pointer("/links/enclosure/href").and_then(Value::as_str).and_then(file_url_to_path),
    })
}

pub fn parse_collection(v: &Value) -> Result<Collection, ZoteroError> {
    let data = data_of(v, "collection")?;
    Ok(Collection {
        key: key_of(data, "collection")?,
        name: text(data, "name"),
        // `parentCollection` is `false` for top-level collections.
        parent_key: non_empty(data, "parentCollection"),
    })
}

pub fn parse_group(v: &Value) -> Result<Group, ZoteroError> {
    let id = v
        .get("id")
        .or_else(|| v.pointer("/data/id"))
        .and_then(Value::as_i64)
        .ok_or_else(|| ZoteroError::Parse("group without `id`".to_string()))?;
    Ok(Group { id, name: v.pointer("/data/name").and_then(Value::as_str).unwrap_or_default().to_string() })
}

/// `file:///a/b%20c.pdf` → `/a/b c.pdf`. Other URLs → `None`.
pub fn file_url_to_path(href: &str) -> Option<PathBuf> {
    let encoded = href.strip_prefix("file://")?;
    let decoded = percent_encoding::percent_decode_str(encoded).decode_utf8().ok()?;
    decoded.starts_with('/').then(|| PathBuf::from(decoded.as_ref()))
}

/// First standalone run of four digits in a Zotero date string ("2006", "March 2016", "2021-05-03").
pub fn year_from_date(date: &str) -> Option<i32> {
    let bytes = date.as_bytes();
    (0..bytes.len().saturating_sub(3))
        .find(|&i| {
            bytes[i..i + 4].iter().all(u8::is_ascii_digit)
                && (i == 0 || !bytes[i - 1].is_ascii_digit())
                && bytes.get(i + 4).is_none_or(|b| !b.is_ascii_digit())
        })
        .and_then(|i| date[i..i + 4].parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_a_journal_article() {
        let v = json!({
            "key": "ABCD1234", "version": 0,
            "library": {"type": "user", "id": 1, "name": "My Library"},
            "data": {
                "key": "ABCD1234", "version": 0, "itemType": "journalArticle",
                "title": "Differential Privacy", "date": "July 2006", "DOI": "10.1007/11787006_1",
                "citationKey": "dwork2006", "tags": [{"tag": "A_core"}, {"tag": "dp", "type": 1}],
                "collections": ["COLL0001"], "dateModified": "2026-09-01T10:00:00Z"
            }
        });
        let item = parse_item(&v).unwrap();
        assert_eq!(item.key, "ABCD1234");
        assert_eq!(item.citation_key.as_deref(), Some("dwork2006"));
        assert_eq!(item.year, Some(2006));
        assert_eq!(item.doi.as_deref(), Some("10.1007/11787006_1"));
        assert_eq!(item.tags, ["A_core", "dp"]);
        assert_eq!(item.collections, ["COLL0001"]);
        assert_eq!(item.library_name, "My Library");
        assert!(item.is_source());
    }

    #[test]
    fn empty_citation_key_and_doi_are_none() {
        let v = json!({"key": "K", "data": {"key": "K", "itemType": "book", "citationKey": " ", "DOI": ""}});
        let item = parse_item(&v).unwrap();
        assert_eq!((item.citation_key, item.doi), (None, None));
    }

    #[test]
    fn notes_and_attachments_are_not_sources() {
        for item_type in ["note", "attachment", "annotation"] {
            let v = json!({"key": "K", "data": {"key": "K", "itemType": item_type}});
            assert!(!parse_item(&v).unwrap().is_source(), "{item_type}");
        }
    }

    #[test]
    fn attachment_path_comes_from_the_enclosure_link() {
        let v = json!({
            "key": "ATT00001",
            "links": {"enclosure": {"href": "file:///home/u/Zotero/storage/ATT00001/Wiest%20et%20al.%20-%202024.pdf",
                                    "type": "application/pdf"}},
            "data": {"key": "ATT00001", "itemType": "attachment", "parentItem": "ABCD1234",
                     "linkMode": "imported_url", "contentType": "application/pdf", "path": null}
        });
        let a = parse_attachment(&v).unwrap();
        assert_eq!(a.path, Some(PathBuf::from("/home/u/Zotero/storage/ATT00001/Wiest et al. - 2024.pdf")));
        assert_eq!(a.parent_key.as_deref(), Some("ABCD1234"));
        assert!(a.is_pdf());
    }

    #[test]
    fn attachment_without_file_has_no_path() {
        let v = json!({"key": "A", "data": {"key": "A", "contentType": "text/html", "linkMode": "linked_url"}});
        let a = parse_attachment(&v).unwrap();
        assert_eq!(a.path, None);
        assert!(!a.is_pdf());
    }

    #[test]
    fn collections_and_groups() {
        let top = json!({"key": "C1", "data": {"key": "C1", "name": "Privacy", "parentCollection": false}});
        let child = json!({"key": "C2", "data": {"key": "C2", "name": "DP", "parentCollection": "C1"}});
        assert_eq!(parse_collection(&top).unwrap().parent_key, None);
        assert_eq!(parse_collection(&child).unwrap().parent_key.as_deref(), Some("C1"));
        let group = json!({"id": 6573630, "version": 0, "data": {"id": 6573630, "name": "Lab"}});
        assert_eq!(parse_group(&group).unwrap(), Group { id: 6573630, name: "Lab".to_string() });
    }

    #[test]
    fn missing_data_is_a_parse_error() {
        assert!(matches!(parse_item(&json!({"key": "K"})), Err(ZoteroError::Parse(_))));
    }

    #[test]
    fn file_urls() {
        assert_eq!(file_url_to_path("file:///a/b%20c.pdf"), Some(PathBuf::from("/a/b c.pdf")));
        assert_eq!(file_url_to_path("https://example.org/a.pdf"), None);
        assert_eq!(file_url_to_path("file://relative"), None);
    }

    #[test]
    fn years() {
        assert_eq!(year_from_date("2021-05-03"), Some(2021));
        assert_eq!(year_from_date("March 2016"), Some(2016));
        assert_eq!(year_from_date("vol. 12345, 1999"), Some(1999));
        assert_eq!(year_from_date("n.d."), None);
        assert_eq!(year_from_date(""), None);
    }
}
```

`client.rs` must exist for this to compile; create it in Step 3 before running the tests.

- [ ] **Step 3: Client (stubbed) and fake server**

`crates/bib-core/src/zotero/client.rs`:

```rust
//! HTTP client for the local API with paging (`start`/`limit`, `Total-Results`).

use std::time::Duration;

use serde_json::Value;

use super::model::{Attachment, Collection, Group, Item, parse_attachment, parse_collection, parse_group, parse_item};
use crate::library::LibraryRef;

pub const DEFAULT_URL: &str = "http://127.0.0.1:23119";
pub const URL_ENV: &str = "BIB_ZOTERO_URL";
/// Maximum page size of the Zotero API.
const PAGE_SIZE: usize = 100;

#[derive(Debug, thiserror::Error)]
pub enum ZoteroError {
    #[error("Zotero is not reachable at {url}: {message}")]
    Unreachable { url: String, message: String },
    #[error("Zotero answered HTTP {status} for {url}")]
    Http { status: u16, url: String },
    #[error("unexpected Zotero response: {0}")]
    Parse(String),
}

pub struct ZoteroClient {
    agent: ureq::Agent,
    base_url: String,
}

impl ZoteroClient {
    pub fn new(base_url: &str) -> ZoteroClient {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        ZoteroClient { agent, base_url: base_url.trim_end_matches('/').to_string() }
    }

    /// `$BIB_ZOTERO_URL` or [`DEFAULT_URL`].
    pub fn from_env() -> ZoteroClient {
        ZoteroClient::new(&std::env::var(URL_ENV).unwrap_or_else(|_| DEFAULT_URL.to_string()))
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Group libraries of the local user.
    pub fn groups(&self) -> Result<Vec<Group>, ZoteroError> {
        self.get_all("/api/users/0/groups")?.iter().map(parse_group).collect()
    }

    /// Top-level items: sources, but also standalone notes and attachments (see [`Item::is_source`]).
    pub fn top_items(&self, library: LibraryRef) -> Result<Vec<Item>, ZoteroError> {
        self.get_all(&format!("/api{}/items/top", library.api_prefix()))?.iter().map(parse_item).collect()
    }

    /// All attachment items, child and standalone.
    pub fn attachments(&self, library: LibraryRef) -> Result<Vec<Attachment>, ZoteroError> {
        self.get_all(&format!("/api{}/items?itemType=attachment", library.api_prefix()))?
            .iter()
            .map(parse_attachment)
            .collect()
    }

    pub fn collections(&self, library: LibraryRef) -> Result<Vec<Collection>, ZoteroError> {
        self.get_all(&format!("/api{}/collections", library.api_prefix()))?.iter().map(parse_collection).collect()
    }

    /// Fetches every page of a JSON array endpoint.
    fn get_all(&self, path_and_query: &str) -> Result<Vec<Value>, ZoteroError> {
        let _ = (path_and_query, PAGE_SIZE);
        todo!()
    }
}
```

`crates/bib-core/src/zotero/fake.rs`:

```rust
//! In-process fake of the Zotero local API for tests. Serves JSON arrays per route with `start`/`limit`
//! paging and a `Total-Results` header. Routes are paths without query, except that an `itemType`
//! parameter is part of the route (`/api/users/0/items?itemType=attachment`).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

type Routes = Arc<Mutex<HashMap<String, Vec<Value>>>>;

pub struct FakeZotero {
    url: String,
    routes: Routes,
}

impl FakeZotero {
    pub fn start() -> FakeZotero {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake Zotero");
        let url = format!("http://{}", listener.local_addr().expect("local address"));
        let routes: Routes = Arc::default();
        let shared = Arc::clone(&routes);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let routes = Arc::clone(&shared);
                std::thread::spawn(move || {
                    let _ = serve(stream, &routes);
                });
            }
        });
        FakeZotero { url, routes }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn set(&self, route: &str, items: Vec<Value>) {
        self.routes.lock().unwrap().insert(route.to_string(), items);
    }

    /// Sets all routes the sync reads for the user library, and an empty group list.
    pub fn user_library(&self, items: Vec<Value>, attachments: Vec<Value>, collections: Vec<Value>) {
        self.set("/api/users/0/groups", Vec::new());
        self.set("/api/users/0/items/top", items);
        self.set("/api/users/0/items?itemType=attachment", attachments);
        self.set("/api/users/0/collections", collections);
    }
}

fn serve(stream: TcpStream, routes: &Routes) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" {
            break;
        }
    }
    let target = request_line.split_whitespace().nth(1).unwrap_or("/").to_string();
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    let param = |name: &str| query.split('&').find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='));
    let route = match param("itemType") {
        Some(item_type) => format!("{path}?itemType={item_type}"),
        None => path.to_string(),
    };
    let number = |name: &str, default: usize| param(name).and_then(|v| v.parse().ok()).unwrap_or(default);
    let response = match routes.lock().unwrap().get(&route) {
        Some(items) => {
            let start = number("start", 0).min(items.len());
            let end = (start + number("limit", 25)).min(items.len());
            let body = Value::Array(items[start..end].to_vec()).to_string();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTotal-Results: {}\r\nLast-Modified-Version: 0\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                items.len(),
                body.len()
            )
        }
        None => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
    };
    let mut stream = stream;
    stream.write_all(response.as_bytes())
}

const DATE: &str = "2026-01-01T00:00:00Z";

/// A journal article in the user library, shaped like the real local API response.
pub fn item(key: &str, citation_key: &str, title: &str, date: &str, doi: &str) -> Value {
    json!({
        "key": key, "version": 0,
        "library": {"type": "user", "id": 1, "name": "My Library"},
        "data": {
            "key": key, "version": 0, "itemType": "journalArticle", "title": title, "date": date, "DOI": doi,
            "citationKey": citation_key, "creators": [], "tags": [], "collections": [], "relations": {},
            "dateAdded": DATE, "dateModified": DATE
        }
    })
}

/// A standalone note (appears among top-level items, is not a source).
pub fn note(key: &str) -> Value {
    json!({"key": key, "version": 0, "data": {"key": key, "version": 0, "itemType": "note", "note": "<p>n</p>",
            "tags": [], "relations": {}, "dateAdded": DATE, "dateModified": DATE}})
}

/// A stored PDF attachment of `parent`; `file_url` like `file:///tmp/a%20b.pdf`.
pub fn attachment(key: &str, parent: &str, file_url: &str) -> Value {
    json!({
        "key": key, "version": 0,
        "links": {"enclosure": {"href": file_url, "type": "application/pdf", "title": "file.pdf", "length": 1}},
        "data": {
            "key": key, "version": 0, "itemType": "attachment", "parentItem": parent, "linkMode": "imported_url",
            "contentType": "application/pdf", "filename": "file.pdf", "tags": [], "relations": {},
            "dateAdded": DATE, "dateModified": DATE
        }
    })
}

pub fn collection(key: &str, name: &str, parent: Option<&str>) -> Value {
    let parent = parent.map_or(Value::Bool(false), Value::from);
    json!({"key": key, "version": 0, "data": {"key": key, "version": 0, "name": name, "parentCollection": parent,
            "relations": {}}})
}

pub fn group(id: i64, name: &str) -> Value {
    json!({"id": id, "version": 0, "data": {"id": id, "version": 0, "name": name, "description": ""}})
}
```

Run: `cargo test -p bib-core zotero::model`
Expected: PASS, 9 tests.

- [ ] **Step 4: Client tests**

Append to `client.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::zotero::fake::{self, FakeZotero};

    #[test]
    fn pages_through_all_results() {
        let zotero = FakeZotero::start();
        let items = (0..250).map(|i| fake::item(&format!("K{i:07}"), "", "t", "2020", "")).collect();
        zotero.set("/api/users/0/items/top", items);
        let fetched = ZoteroClient::new(zotero.url()).top_items(LibraryRef::User).unwrap();
        assert_eq!(fetched.len(), 250);
        assert_eq!(fetched[0].key, "K0000000");
        assert_eq!(fetched[249].key, "K0000249");
    }

    #[test]
    fn uses_group_paths_and_the_attachment_filter() {
        let zotero = FakeZotero::start();
        zotero.set("/api/groups/7/items?itemType=attachment", vec![fake::attachment("A1", "P1", "file:///x.pdf")]);
        zotero.set("/api/groups/7/collections", vec![fake::collection("C1", "Lab", None)]);
        zotero.set("/api/users/0/groups", vec![fake::group(7, "Lab")]);
        let client = ZoteroClient::new(zotero.url());
        assert_eq!(client.attachments(LibraryRef::Group(7)).unwrap()[0].key, "A1");
        assert_eq!(client.collections(LibraryRef::Group(7)).unwrap()[0].name, "Lab");
        assert_eq!(client.groups().unwrap(), vec![Group { id: 7, name: "Lab".to_string() }]);
    }

    #[test]
    fn empty_result_is_fine() {
        let zotero = FakeZotero::start();
        zotero.set("/api/users/0/collections", Vec::new());
        assert!(ZoteroClient::new(zotero.url()).collections(LibraryRef::User).unwrap().is_empty());
    }

    #[test]
    fn http_errors_carry_the_status() {
        let zotero = FakeZotero::start();
        let err = ZoteroClient::new(zotero.url()).top_items(LibraryRef::Group(9)).unwrap_err();
        assert!(matches!(err, ZoteroError::Http { status: 404, .. }), "{err}");
    }

    #[test]
    fn closed_port_is_unreachable() {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let err = ZoteroClient::new(&format!("http://127.0.0.1:{port}/")).groups().unwrap_err();
        assert!(matches!(err, ZoteroError::Unreachable { .. }), "{err}");
        assert!(err.to_string().contains("not reachable"));
    }
}
```

Run: `cargo test -p bib-core zotero::client`
Expected: FAIL, `not yet implemented`.

- [ ] **Step 5: Implement paging**

Replace `get_all`:

```rust
    fn get_all(&self, path_and_query: &str) -> Result<Vec<Value>, ZoteroError> {
        let separator = if path_and_query.contains('?') { '&' } else { '?' };
        let mut all = Vec::new();
        loop {
            let url =
                format!("{}{path_and_query}{separator}format=json&limit={PAGE_SIZE}&start={}", self.base_url, all.len());
            let mut response = self
                .agent
                .get(&url)
                .call()
                .map_err(|e| ZoteroError::Unreachable { url: self.base_url.clone(), message: e.to_string() })?;
            let status = response.status().as_u16();
            if status != 200 {
                return Err(ZoteroError::Http { status, url });
            }
            let total = response
                .headers()
                .get("Total-Results")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<usize>().ok());
            let page: Vec<Value> =
                response.body_mut().read_json().map_err(|e| ZoteroError::Parse(format!("{url}: {e}")))?;
            let page_len = page.len();
            all.extend(page);
            let done = match total {
                Some(total) => all.len() >= total,
                None => page_len < PAGE_SIZE,
            };
            if done || page_len == 0 {
                return Ok(all);
            }
        }
    }
```

Run: `cargo test -p bib-core zotero`
Expected: PASS, 14 tests.

- [ ] **Step 6: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS. (Clippy may flag the long `format!` line in `fake.rs`; `cargo fmt` keeps string literals intact, which is fine.)

```bash
git add Cargo.toml Cargo.lock crates/bib-core
git commit -m "Add read-only Zotero local API client with test fake

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 5: Full sync into the database

**Files:**
- Modify: `crates/bib-core/Cargo.toml`, `crates/bib-core/src/zotero/mod.rs`
- Create: `crates/bib-core/src/zotero/sync.rs`

**Interfaces:**
- Consumes: `db::{MIGRATIONS, migrate}`, tables from Task 2; `review::{enqueue, NewReviewItem}`; `LibraryRef` (Task 3); `ZoteroClient`, `Item`, `Attachment`, `Collection`, `fake` (Task 4).
- Produces:
  - `bib_core::zotero::sync::{SyncCounts { sources_new, sources_changed, sources_retired, sources_reactivated, attachments, skipped_top_level: usize }, LibraryReport { library: LibraryRef, name: String, counts: SyncCounts }}` (both `serde::Serialize`)
  - `sync_all(conn: &mut Connection, client: &ZoteroClient) -> anyhow::Result<Vec<LibraryReport>>` (user library first, then groups in API order)
  - `sync_library(conn: &mut Connection, client: &ZoteroClient, library: LibraryRef, name: Option<String>) -> anyhow::Result<LibraryReport>`
  - `last_successful_sync_age(conn: &Connection) -> rusqlite::Result<Option<i64>>` (seconds since the newest `ok` run)
  - `data_hash(data: &serde_json::Value) -> String`
  - Errors from the client stay `ZoteroError` inside the `anyhow::Error` (callers use `downcast_ref::<ZoteroError>()`).
  - Review item kind `source_retired`, dedupe key `source_retired:<library>:<item_key>`, `node_type = "source"`.

- [ ] **Step 1: Dependency and module**

`crates/bib-core/Cargo.toml` → `[dependencies]` add `sha2.workspace = true`.
`crates/bib-core/src/zotero/mod.rs`: add `pub mod sync;` below `mod model;`.

- [ ] **Step 2: Tests**

`crates/bib-core/src/zotero/sync.rs` (public API stubbed, tests complete):

```rust
//! Full sync of Zotero libraries (spec §6). The local API reports `version = 0` everywhere and has no
//! `/deleted` endpoint, so every run fetches everything and diffs against the database: new, changed
//! (hash of the item data), gone (retired, never deleted, and queued for review because excerpts may depend on
//! the source).

use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, Transaction, params};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{Attachment, Collection, Item, ZoteroClient};
use crate::library::LibraryRef;
use crate::review::{NewReviewItem, enqueue};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SyncCounts {
    pub sources_new: usize,
    pub sources_changed: usize,
    pub sources_retired: usize,
    pub sources_reactivated: usize,
    /// PDF attachments seen in this run.
    pub attachments: usize,
    /// Top-level notes and standalone attachments, which are not sources.
    pub skipped_top_level: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LibraryReport {
    pub library: LibraryRef,
    pub name: String,
    pub counts: SyncCounts,
}

pub fn sync_all(conn: &mut Connection, client: &ZoteroClient) -> anyhow::Result<Vec<LibraryReport>> {
    let _ = (conn, client);
    todo!()
}

pub fn sync_library(
    conn: &mut Connection,
    client: &ZoteroClient,
    library: LibraryRef,
    name: Option<String>,
) -> anyhow::Result<LibraryReport> {
    let _ = (conn, client, library, name);
    todo!()
}

pub fn last_successful_sync_age(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    let _ = conn;
    todo!()
}

/// SHA-256 of the canonical JSON (sorted keys) of `data` without `version`.
pub fn data_hash(data: &Value) -> String {
    let _ = data;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{MIGRATIONS, migrate};
    use crate::review::open_count;
    use crate::zotero::fake::{self, FakeZotero};
    use serde_json::json;

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn
    }

    fn count(conn: &Connection, sql: &str) -> i64 {
        conn.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    fn tagged(mut item: Value, tag: &str, collection: &str) -> Value {
        item["data"]["tags"] = json!([{"tag": tag}]);
        item["data"]["collections"] = json!([collection]);
        item
    }

    fn sync_user(conn: &mut Connection, zotero: &FakeZotero) -> SyncCounts {
        sync_library(conn, &ZoteroClient::new(zotero.url()), LibraryRef::User, None).unwrap().counts
    }

    #[test]
    fn first_sync_stores_sources_tags_collections_and_pdfs() {
        let zotero = FakeZotero::start();
        let mut html = fake::attachment("ATTHTML1", "DWORK001", "file:///tmp/page.html");
        html["data"]["contentType"] = json!("text/html");
        zotero.user_library(
            vec![
                tagged(fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", ""), "A_core", "COLL0001"),
                fake::item("ABADI001", "abadi2016", "Deep Learning with Differential Privacy", "2016", ""),
                fake::note("NOTE0001"),
            ],
            vec![fake::attachment("ATTPDF01", "DWORK001", "file:///tmp/zotero/a%20b.pdf"), html],
            vec![fake::collection("COLL0001", "Privacy", None)],
        );
        let mut conn = db();
        let counts = sync_user(&mut conn, &zotero);
        assert_eq!(
            counts,
            SyncCounts { sources_new: 2, attachments: 1, skipped_top_level: 1, ..SyncCounts::default() }
        );
        assert_eq!(count(&conn, "SELECT count(*) FROM source WHERE status = 'active'"), 2);
        assert_eq!(count(&conn, "SELECT count(*) FROM source_tag WHERE tag = 'A_core'"), 1);
        assert_eq!(count(&conn, "SELECT count(*) FROM source_collection"), 1);
        let (path, source_key): (String, String) = conn
            .query_row(
                "SELECT a.path, s.item_key FROM attachment a JOIN source s ON s.id = a.source_id",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((path.as_str(), source_key.as_str()), ("/tmp/zotero/a b.pdf", "DWORK001"));
        let (name, status): (String, String) = conn
            .query_row("SELECT l.name, r.status FROM sync_run r JOIN library l ON l.id = r.library_id", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!((name.as_str(), status.as_str()), ("My Library", "ok"));
    }

    #[test]
    fn unchanged_resync_changes_nothing_and_changed_items_are_updated() {
        let zotero = FakeZotero::start();
        zotero.user_library(vec![fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", "")], vec![], vec![]);
        let mut conn = db();
        sync_user(&mut conn, &zotero);
        assert_eq!(sync_user(&mut conn, &zotero), SyncCounts::default());

        zotero.set("/api/users/0/items/top", vec![fake::item("DWORK001", "dwork2006", "Differential privacy", "2006", "")]);
        assert_eq!(sync_user(&mut conn, &zotero).sources_changed, 1);
        let title: String = conn.query_row("SELECT title FROM source", [], |r| r.get(0)).unwrap();
        assert_eq!(title, "Differential privacy");
    }

    #[test]
    fn vanished_items_are_retired_queued_once_and_reactivated() {
        let zotero = FakeZotero::start();
        let item = fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", "");
        zotero.user_library(vec![item.clone()], vec![], vec![]);
        let mut conn = db();
        sync_user(&mut conn, &zotero);

        zotero.set("/api/users/0/items/top", vec![]);
        assert_eq!(sync_user(&mut conn, &zotero).sources_retired, 1);
        assert_eq!(sync_user(&mut conn, &zotero).sources_retired, 0);
        assert_eq!(count(&conn, "SELECT count(*) FROM source WHERE status = 'retired'"), 1);
        assert_eq!(count(&conn, "SELECT count(*) FROM source"), 1, "sources are never deleted");
        assert_eq!(open_count(&conn).unwrap(), 1);

        zotero.set("/api/users/0/items/top", vec![item]);
        assert_eq!(sync_user(&mut conn, &zotero).sources_reactivated, 1);
        assert_eq!(count(&conn, "SELECT count(*) FROM source WHERE status = 'active'"), 1);
    }

    #[test]
    fn extraction_results_survive_unless_the_file_changes() {
        let zotero = FakeZotero::start();
        zotero.user_library(
            vec![fake::item("DWORK001", "dwork2006", "t", "2006", "")],
            vec![fake::attachment("ATTPDF01", "DWORK001", "file:///tmp/a.pdf")],
            vec![],
        );
        let mut conn = db();
        sync_user(&mut conn, &zotero);
        conn.execute("UPDATE attachment SET checksum = 'abc', page_count = 3", []).unwrap();
        sync_user(&mut conn, &zotero);
        assert_eq!(count(&conn, "SELECT count(*) FROM attachment WHERE checksum = 'abc'"), 1);

        zotero.set("/api/users/0/items?itemType=attachment", vec![fake::attachment("ATTPDF01", "DWORK001", "file:///tmp/b.pdf")]);
        sync_user(&mut conn, &zotero);
        assert_eq!(count(&conn, "SELECT count(*) FROM attachment WHERE checksum IS NULL AND page_count IS NULL"), 1);

        zotero.set("/api/users/0/items?itemType=attachment", vec![]);
        sync_user(&mut conn, &zotero);
        assert_eq!(count(&conn, "SELECT count(*) FROM attachment WHERE status = 'retired'"), 1);
    }

    #[test]
    fn sync_all_covers_group_libraries_with_the_same_item_keys() {
        let zotero = FakeZotero::start();
        zotero.user_library(vec![fake::item("SAME0001", "dwork2006", "Differential Privacy", "2006", "")], vec![], vec![]);
        zotero.set("/api/users/0/groups", vec![fake::group(42, "Lab")]);
        zotero.set("/api/groups/42/items/top", vec![fake::item("SAME0001", "dwork2006", "Differential Privacy", "2006", "")]);
        zotero.set("/api/groups/42/items?itemType=attachment", vec![]);
        zotero.set("/api/groups/42/collections", vec![]);
        let mut conn = db();
        let reports = sync_all(&mut conn, &ZoteroClient::new(zotero.url())).unwrap();
        let libraries: Vec<_> = reports.iter().map(|r| (r.library, r.name.as_str())).collect();
        assert_eq!(libraries, [(LibraryRef::User, "My Library"), (LibraryRef::Group(42), "Lab")]);
        assert_eq!(count(&conn, "SELECT count(*) FROM source"), 2);
        assert!(last_successful_sync_age(&conn).unwrap().is_some_and(|age| (0..60).contains(&age)));
    }

    #[test]
    fn failed_fetch_records_a_failed_run_and_keeps_the_error_type() {
        let zotero = FakeZotero::start();
        let mut conn = db();
        let err = sync_library(&mut conn, &ZoteroClient::new(zotero.url()), LibraryRef::User, None).unwrap_err();
        assert!(err.downcast_ref::<crate::zotero::ZoteroError>().is_some(), "{err:#}");
        let (status, error): (String, String) =
            conn.query_row("SELECT status, error FROM sync_run", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(status, "failed");
        assert!(error.contains("404"), "{error}");
        assert_eq!(last_successful_sync_age(&conn).unwrap(), None);
    }

    #[test]
    fn data_hash_ignores_key_order_and_version() {
        let a = json!({"title": "x", "version": 3, "creators": [{"lastName": "D", "firstName": "C"}]});
        let b = json!({"creators": [{"firstName": "C", "lastName": "D"}], "version": 0, "title": "x"});
        assert_eq!(data_hash(&a), data_hash(&b));
        assert_ne!(data_hash(&a), data_hash(&json!({"title": "y"})));
        assert_eq!(data_hash(&a).len(), 64);
    }
}
```

Run: `cargo test -p bib-core zotero::sync`
Expected: FAIL, `not yet implemented`.

- [ ] **Step 3: Implement**

Replace the four stubs and add the private helpers:

```rust
pub fn sync_all(conn: &mut Connection, client: &ZoteroClient) -> anyhow::Result<Vec<LibraryReport>> {
    let groups = client.groups()?;
    let mut targets = vec![(LibraryRef::User, None)];
    targets.extend(groups.into_iter().map(|group| (LibraryRef::Group(group.id), Some(group.name))));
    targets.into_iter().map(|(library, name)| sync_library(conn, client, library, name)).collect()
}

pub fn sync_library(
    conn: &mut Connection,
    client: &ZoteroClient,
    library: LibraryRef,
    name: Option<String>,
) -> anyhow::Result<LibraryReport> {
    let library_id = upsert_library(conn, library, name.as_deref().unwrap_or(""))?;
    conn.execute("INSERT INTO sync_run (library_id) VALUES (?1)", [library_id])?;
    let run_id = conn.last_insert_rowid();
    let result = fetch(client, library, name).and_then(|snapshot| apply(conn, library, library_id, &snapshot));
    let (status, error, counts) = match &result {
        Ok(report) => ("ok", None, report.counts.clone()),
        Err(err) => ("failed", Some(format!("{err:#}")), SyncCounts::default()),
    };
    conn.execute(
        "UPDATE sync_run SET finished_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), status = ?2, error = ?3,
             sources_new = ?4, sources_changed = ?5, sources_retired = ?6, sources_reactivated = ?7, attachments = ?8
         WHERE id = ?1",
        params![
            run_id,
            status,
            error,
            counts.sources_new as i64,
            counts.sources_changed as i64,
            counts.sources_retired as i64,
            counts.sources_reactivated as i64,
            counts.attachments as i64
        ],
    )?;
    result
}

pub fn last_successful_sync_age(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT CAST((julianday('now') - julianday(max(finished_at))) * 86400 AS INTEGER)
         FROM sync_run WHERE status = 'ok'",
        [],
        |row| row.get(0),
    )
}

pub fn data_hash(data: &Value) -> String {
    let mut data = data.clone();
    if let Some(object) = data.as_object_mut() {
        object.remove("version");
    }
    let mut canonical = String::new();
    write_canonical(&data, &mut canonical);
    Sha256::digest(canonical.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                write_canonical(&map[key], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// Everything fetched for one library, before touching the database.
struct Snapshot {
    name: String,
    items: Vec<Item>,
    attachments: Vec<Attachment>,
    collections: Vec<Collection>,
}

fn fetch(client: &ZoteroClient, library: LibraryRef, name: Option<String>) -> anyhow::Result<Snapshot> {
    let items = client.top_items(library)?;
    let attachments = client.attachments(library)?;
    let collections = client.collections(library)?;
    let name = name.or_else(|| items.first().map(|item| item.library_name.clone())).unwrap_or_default();
    Ok(Snapshot { name, items, attachments, collections })
}

fn upsert_library(conn: &Connection, library: LibraryRef, name: &str) -> rusqlite::Result<i64> {
    let (kind, zotero_id) = library.db_key();
    conn.execute(
        "INSERT INTO library (kind, zotero_id, name) VALUES (?1, ?2, ?3)
         ON CONFLICT (kind, zotero_id) DO UPDATE
         SET name = CASE WHEN excluded.name = '' THEN library.name ELSE excluded.name END",
        params![kind, zotero_id, name],
    )?;
    conn.query_row("SELECT id FROM library WHERE kind = ?1 AND zotero_id = ?2", params![kind, zotero_id], |row| {
        row.get(0)
    })
}

/// Writes a snapshot in one transaction.
fn apply(
    conn: &mut Connection,
    library: LibraryRef,
    library_id: i64,
    snapshot: &Snapshot,
) -> anyhow::Result<LibraryReport> {
    let tx = conn.transaction()?;
    upsert_library(&tx, library, &snapshot.name)?;
    let mut counts = SyncCounts::default();
    let collection_ids = sync_collections(&tx, library_id, &snapshot.collections)?;
    let source_ids = sync_sources(&tx, library, library_id, &snapshot.items, &collection_ids, &mut counts)?;
    sync_attachments(&tx, library_id, &snapshot.attachments, &source_ids, &mut counts)?;
    tx.commit()?;
    Ok(LibraryReport { library, name: snapshot.name.clone(), counts })
}

fn sync_collections(
    tx: &Transaction,
    library_id: i64,
    collections: &[Collection],
) -> rusqlite::Result<HashMap<String, i64>> {
    let mut ids = HashMap::new();
    for collection in collections {
        tx.execute(
            "INSERT INTO collection (library_id, collection_key, name, parent_key) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (library_id, collection_key) DO UPDATE SET name = excluded.name, parent_key = excluded.parent_key",
            params![library_id, collection.key, collection.name, collection.parent_key],
        )?;
        let id: i64 = tx.query_row(
            "SELECT id FROM collection WHERE library_id = ?1 AND collection_key = ?2",
            params![library_id, collection.key],
            |row| row.get(0),
        )?;
        ids.insert(collection.key.clone(), id);
    }
    let stored: Vec<(i64, String)> = tx
        .prepare("SELECT id, collection_key FROM collection WHERE library_id = ?1")?
        .query_map([library_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;
    for (id, key) in stored {
        if !ids.contains_key(&key) {
            // Memberships cascade; collections are not referenced by user data.
            tx.execute("DELETE FROM collection WHERE id = ?1", [id])?;
        }
    }
    Ok(ids)
}

struct StoredSource {
    id: i64,
    data_hash: String,
    retired: bool,
}

fn sync_sources(
    tx: &Transaction,
    library: LibraryRef,
    library_id: i64,
    items: &[Item],
    collection_ids: &HashMap<String, i64>,
    counts: &mut SyncCounts,
) -> anyhow::Result<HashMap<String, i64>> {
    let stored: HashMap<String, StoredSource> = tx
        .prepare("SELECT item_key, id, data_hash, status FROM source WHERE library_id = ?1")?
        .query_map([library_id], |row| {
            let status: String = row.get(3)?;
            Ok((row.get(0)?, StoredSource { id: row.get(1)?, data_hash: row.get(2)?, retired: status == "retired" }))
        })?
        .collect::<Result<_, _>>()?;

    let mut seen = HashMap::new();
    for item in items {
        if !item.is_source() {
            counts.skipped_top_level += 1;
            continue;
        }
        let hash = data_hash(&item.data);
        let data = item.data.to_string();
        let id = match stored.get(&item.key) {
            None => {
                tx.execute(
                    "INSERT INTO source (library_id, item_key, citation_key, item_type, title, year, doi, data, data_hash,
                                         date_modified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        library_id,
                        item.key,
                        item.citation_key,
                        item.item_type,
                        item.title,
                        item.year,
                        item.doi,
                        data,
                        hash,
                        item.date_modified
                    ],
                )?;
                counts.sources_new += 1;
                tx.last_insert_rowid()
            }
            Some(existing) => {
                if existing.data_hash != hash {
                    counts.sources_changed += 1;
                }
                if existing.retired {
                    counts.sources_reactivated += 1;
                }
                tx.execute(
                    "UPDATE source SET citation_key = ?2, item_type = ?3, title = ?4, year = ?5, doi = ?6, data = ?7,
                         data_hash = ?8, date_modified = ?9, status = 'active',
                         last_seen = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE id = ?1",
                    params![
                        existing.id,
                        item.citation_key,
                        item.item_type,
                        item.title,
                        item.year,
                        item.doi,
                        data,
                        hash,
                        item.date_modified
                    ],
                )?;
                existing.id
            }
        };
        tx.execute("DELETE FROM source_tag WHERE source_id = ?1", [id])?;
        for tag in &item.tags {
            tx.execute("INSERT OR IGNORE INTO source_tag (source_id, tag) VALUES (?1, ?2)", params![id, tag])?;
        }
        tx.execute("DELETE FROM source_collection WHERE source_id = ?1", [id])?;
        for collection_id in item.collections.iter().filter_map(|key| collection_ids.get(key)) {
            tx.execute(
                "INSERT OR IGNORE INTO source_collection (source_id, collection_id) VALUES (?1, ?2)",
                params![id, collection_id],
            )?;
        }
        seen.insert(item.key.clone(), id);
    }

    for (key, existing) in &stored {
        if existing.retired || seen.contains_key(key) {
            continue;
        }
        tx.execute("UPDATE source SET status = 'retired' WHERE id = ?1", [existing.id])?;
        counts.sources_retired += 1;
        enqueue(
            tx,
            &NewReviewItem {
                kind: "source_retired",
                node_type: Some("source"),
                node_id: Some(existing.id),
                message: &format!(
                    "Zotero item {library}:{key} is no longer in Zotero (deleted, merged or moved); check excerpts that depend on it"
                ),
                details: &serde_json::json!({"library": library.to_string(), "item_key": key}),
                dedupe_key: &format!("source_retired:{library}:{key}"),
            },
        )?;
    }
    Ok(seen)
}

fn sync_attachments(
    tx: &Transaction,
    library_id: i64,
    attachments: &[Attachment],
    source_ids: &HashMap<String, i64>,
    counts: &mut SyncCounts,
) -> rusqlite::Result<()> {
    let mut seen = HashSet::new();
    for attachment in attachments.iter().filter(|a| a.is_pdf()) {
        let source_id = attachment.parent_key.as_ref().and_then(|key| source_ids.get(key)).copied();
        let path = attachment.path.as_ref().map(|p| p.to_string_lossy().into_owned());
        // All SET expressions see the old row, so checksum and page count survive only an unchanged path.
        tx.execute(
            "INSERT INTO attachment (library_id, item_key, source_id, content_type, link_mode, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (library_id, item_key) DO UPDATE SET
                 source_id = excluded.source_id,
                 content_type = excluded.content_type,
                 link_mode = excluded.link_mode,
                 checksum = CASE WHEN attachment.path IS excluded.path THEN attachment.checksum END,
                 page_count = CASE WHEN attachment.path IS excluded.path THEN attachment.page_count END,
                 path = excluded.path,
                 status = 'active'",
            params![library_id, attachment.key, source_id, attachment.content_type, attachment.link_mode, path],
        )?;
        seen.insert(attachment.key.as_str());
        counts.attachments += 1;
    }
    let active: Vec<(i64, String)> = tx
        .prepare("SELECT id, item_key FROM attachment WHERE library_id = ?1 AND status = 'active'")?
        .query_map([library_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;
    for (id, key) in active {
        if !seen.contains(key.as_str()) {
            tx.execute("UPDATE attachment SET status = 'retired' WHERE id = ?1", [id])?;
        }
    }
    Ok(())
}
```

If the borrow checker rejects `.prepare(...)?.query_map(...)?.collect()` in one expression (the statement is a temporary), bind the statement to a local (`let mut stmt = tx.prepare(...)?;`) and collect from it inside a block.

Run: `cargo test -p bib-core zotero::sync`
Expected: PASS, 7 tests.

- [ ] **Step 4: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

```bash
git add crates/bib-core Cargo.lock
git commit -m "Add full Zotero sync with retirement and review queue entries

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 6: Citation-key resolution and `.bib` comparison

**Files:**
- Modify: `Cargo.toml`, `crates/bib-core/Cargo.toml`, `crates/bib-core/src/lib.rs`
- Create: `crates/bib-core/src/resolve.rs`, `crates/bib-core/src/bibfile.rs`

**Interfaces:**
- Consumes: tables `source`, `library` (Task 2), `LibraryRef::from_db` (Task 3), `normalize::normalize` (plan 1).
- Produces:
  - `bib_core::resolve::SourceRow { id: i64, library: LibraryRef, item_key: String, citation_key: Option<String>, title: String, year: Option<i32>, doi: Option<String> }`
  - `bib_core::resolve::Resolution { NotFound, Found { source: SourceRow, duplicates: Vec<SourceRow> }, Conflict(Vec<SourceRow>) }`
  - `resolve_key(conn: &Connection, key: &str, order: &[LibraryRef]) -> rusqlite::Result<Resolution>`
  - `same_work(a: &SourceRow, b: &SourceRow) -> bool`
  - `keyed_sources(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<SourceRow>>` (one winning source per citation key, sorted by key)
  - `conflicts(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<(String, Vec<SourceRow>)>>` (sorted by key)
  - `bib_core::bibfile::{BibEntry { key: String, title: Option<String>, year: Option<i32>, doi: Option<String> }, parse_bib(src: &str) -> Result<Vec<BibEntry>, String>, BibFinding { OnlyInBib { key: String }, MetadataDiffers { key: String, field: &'static str, bib: String, zotero: String } }, compare(bib: &[BibEntry], zotero: &[SourceRow]) -> Vec<BibFinding>}`

- [ ] **Step 1: Dependency and modules**

`Cargo.toml` → `[workspace.dependencies]` add `biblatex = "0.12"`.
`crates/bib-core/Cargo.toml` → `[dependencies]` add `biblatex.workspace = true`.
`crates/bib-core/src/lib.rs` gains `pub mod bibfile;` and `pub mod resolve;` (keep the list alphabetical).

- [ ] **Step 2: Resolution tests**

`crates/bib-core/src/resolve.rs`:

```rust
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
    Found { source: SourceRow, duplicates: Vec<SourceRow> },
    /// Different works share the key; in resolution order.
    Conflict(Vec<SourceRow>),
}

pub fn resolve_key(conn: &Connection, key: &str, order: &[LibraryRef]) -> rusqlite::Result<Resolution> {
    let _ = (conn, key, order);
    todo!()
}

pub fn same_work(a: &SourceRow, b: &SourceRow) -> bool {
    let _ = (a, b);
    todo!()
}

pub fn keyed_sources(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<SourceRow>> {
    let _ = (conn, order);
    todo!()
}

pub fn conflicts(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<(String, Vec<SourceRow>)>> {
    let _ = (conn, order);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{MIGRATIONS, migrate};
    use rusqlite::params;

    const USER: LibraryRef = LibraryRef::User;
    const LAB: LibraryRef = LibraryRef::Group(42);
    const OTHER: LibraryRef = LibraryRef::Group(99);

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn.execute("INSERT INTO library (id, kind, zotero_id) VALUES (1, 'user', 0), (2, 'group', 42), (3, 'group', 99)", [])
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
        assert_eq!(resolve_key(&conn, "dwork2006", &[USER, LAB]).unwrap(), Resolution::NotFound);
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
        assert!(same_work(&row("A", Some(1), Some("10.1/X")), &row("B", Some(2), Some("10.1/x"))), "DOI decides");
        assert!(same_work(&row("Deep  Learning–Privacy", Some(2016), None), &row("deep learning-privacy", Some(2016), Some("10.1/y"))));
        assert!(!same_work(&row("Deep Learning", Some(2016), None), &row("Deep Learning", Some(2017), None)));
        assert!(!same_work(&row("Deep Learning", None, None), &row("Deep Learning", None, None)), "year required");
        assert!(!same_work(&row("", Some(2016), None), &row("", Some(2016), None)), "empty titles never match");
    }

    #[test]
    fn keyed_sources_and_conflicts_cover_the_project_libraries() {
        let conn = db();
        add(&conn, 1, "U1", "dwork2006", "Differential Privacy", 2006, None);
        add(&conn, 2, "G1", "dwork2006", "Differential Privacy", 2006, None);
        add(&conn, 1, "U2", "smith2020", "Graph Databases", 2020, None);
        add(&conn, 2, "G2", "smith2020", "Protein Folding", 2020, None);
        add(&conn, 2, "G3", "abadi2016", "Deep Learning with Differential Privacy", 2016, None);
        add(&conn, 3, "O1", "other2021", "Elsewhere", 2021, None);
        conn.execute("INSERT INTO source (library_id, item_key, item_type, data, data_hash) VALUES (1, 'NOKEY', 'book', '{}', 'h')", [])
            .unwrap();

        let winners = keyed_sources(&conn, &[USER, LAB]).unwrap();
        let summary: Vec<_> = winners.iter().map(|r| (r.citation_key.as_deref().unwrap(), r.item_key.as_str())).collect();
        assert_eq!(summary, [("abadi2016", "G3"), ("dwork2006", "U1"), ("smith2020", "U2")]);

        let found = conflicts(&conn, &[USER, LAB]).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "smith2020");
        assert_eq!(keys(&found[0].1), [(USER, "U2"), (LAB, "G2")]);
    }
}
```

Run: `cargo test -p bib-core resolve`
Expected: FAIL, `not yet implemented`.

- [ ] **Step 3: Implement resolution**

Replace the four stubs and add helpers:

```rust
pub fn resolve_key(conn: &Connection, key: &str, order: &[LibraryRef]) -> rusqlite::Result<Resolution> {
    Ok(decide(rank(query_sources(conn, Some(key))?, order)))
}

pub fn same_work(a: &SourceRow, b: &SourceRow) -> bool {
    let doi_match = matches!((&a.doi, &b.doi), (Some(x), Some(y)) if x.eq_ignore_ascii_case(y));
    let title = normalize(&a.title).text;
    let title_year_match = !title.is_empty() && title == normalize(&b.title).text && a.year.is_some() && a.year == b.year;
    doi_match || title_year_match
}

pub fn keyed_sources(conn: &Connection, order: &[LibraryRef]) -> rusqlite::Result<Vec<SourceRow>> {
    Ok(grouped(conn, order)?.into_values().filter_map(|rows| rows.into_iter().next()).collect())
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
        Resolution::Found { source: first, duplicates: candidates }
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
        .filter_map(|row| order.iter().position(|library| *library == row.library).map(|position| (position, row)))
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
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, format!("unknown library kind {kind:?}").into())
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
```

Run: `cargo test -p bib-core resolve`
Expected: PASS, 6 tests.

- [ ] **Step 4: `.bib` tests**

`crates/bib-core/src/bibfile.rs`:

```rust
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
    MetadataDiffers { key: String, field: &'static str, bib: String, zotero: String },
}

pub fn parse_bib(src: &str) -> Result<Vec<BibEntry>, String> {
    let _ = src;
    todo!()
}

/// Findings sorted by key. Keys only in Zotero are not reported here: whether that matters depends on the
/// document's citations (Typst parser, spec step 5).
pub fn compare(bib: &[BibEntry], zotero: &[SourceRow]) -> Vec<BibFinding> {
    let _ = (bib, zotero);
    todo!()
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
                BibEntry { key: "nodate".into(), title: Some("Untitled draft".into()), year: None, doi: None },
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
            source("dwork2006", "Differential privacy", Some(2006), Some("10.1007/11787006_1")),
            source("abadi2016", "Deep Learning with Differential Privacy", Some(2015), None),
            source("unused2020", "Cited nowhere", Some(2020), None),
        ];
        assert_eq!(
            compare(&bib, &zotero),
            [
                BibFinding::MetadataDiffers { key: "abadi2016".into(), field: "year", bib: "2016".into(), zotero: "2015".into() },
                BibFinding::OnlyInBib { key: "nodate".into() },
            ]
        );
    }

    #[test]
    fn differing_title_and_doi_are_reported() {
        let bib = parse_bib(BIB).unwrap();
        let zotero = [source("dwork2006", "Calibrating Noise", Some(2006), Some("10.1007/OTHER"))];
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
```

Run: `cargo test -p bib-core bibfile`
Expected: FAIL, `not yet implemented`.

- [ ] **Step 5: Implement `.bib` reading and comparison**

```rust
pub fn parse_bib(src: &str) -> Result<Vec<BibEntry>, String> {
    let bibliography = Bibliography::parse(src).map_err(|e| e.to_string())?;
    Ok(bibliography
        .iter()
        .map(|entry| BibEntry {
            key: entry.key.clone(),
            title: entry.title().ok().map(|chunks| chunks.format_verbatim()),
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

pub fn compare(bib: &[BibEntry], zotero: &[SourceRow]) -> Vec<BibFinding> {
    let zotero_by_key: BTreeMap<&str, &SourceRow> =
        zotero.iter().filter_map(|s| s.citation_key.as_deref().map(|key| (key, s))).collect();
    let bib_by_key: BTreeMap<&str, &BibEntry> = bib.iter().map(|e| (e.key.as_str(), e)).collect();
    let mut findings = Vec::new();
    for (key, entry) in bib_by_key {
        let Some(source) = zotero_by_key.get(key) else {
            findings.push(BibFinding::OnlyInBib { key: key.to_string() });
            continue;
        };
        let mut differs = |field: &'static str, bib: String, zotero: String| {
            findings.push(BibFinding::MetadataDiffers { key: key.to_string(), field, bib, zotero })
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
```

Run: `cargo test -p bib-core bibfile`
Expected: PASS, 4 tests. If `format_verbatim` keeps the braces of `{P}rivacy`, strip `{` and `}` from the title before storing it and note that in the report; do not change the test.

- [ ] **Step 6: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

```bash
git add Cargo.toml Cargo.lock crates/bib-core
git commit -m "Add citation-key resolution across libraries and .bib comparison

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 7: Doctor checks

**Files:**
- Modify: `crates/bib-core/src/lib.rs`
- Create: `crates/bib-core/src/doctor.rs`

**Interfaces:**
- Consumes: `db::schema_version`, tables (Task 2); `review::{enqueue, open_count, NewReviewItem}`; `project::ProjectConfig` (Task 3); `resolve::{conflicts, keyed_sources, SourceRow}`, `bibfile::{parse_bib, compare, BibFinding}` (Task 6).
- Produces:
  - `bib_core::doctor::{Severity { Info, Warning, Error }` (ordered, serializes lowercase)`, Finding { severity: Severity, code: &'static str, message: String }` (`Serialize`)`, Project<'a> { root: &'a Path, config: &'a ProjectConfig }, DoctorInput<'a> { conn: &'a Connection, db_path: &'a Path, zotero: Result<(), String>, project: Option<Project<'a>> }, run(input: &DoctorInput) -> anyhow::Result<Vec<Finding>>, format_age(seconds: i64) -> String}`
  - Finding codes: `database`, `zotero`, `zotero_unreachable`, `never_synced`, `library`, `missing_citation_key`, `missing_pdf_file`, `pdf_without_file`, `retired_sources`, `review_open`, `project`, `library_unknown`, `key_conflict`, `bib_unreadable`, `bib_only`, `metadata_drift`
  - Review item kind `key_conflict`, dedupe key `key_conflict:<project id>:<citation key>`.

- [ ] **Step 1: Tests**

`crates/bib-core/src/lib.rs` gains `pub mod doctor;`.

`crates/bib-core/src/doctor.rs`:

```rust
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
    let _ = input;
    todo!()
}

/// "42 s", "5 min", "3 h", "12 d".
pub fn format_age(seconds: i64) -> String {
    let _ = seconds;
    todo!()
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
        DoctorInput { conn, db_path: Path::new("/data/bib.db"), zotero: Ok(()), project }
    }

    fn codes(findings: &[Finding]) -> Vec<(&'static str, Severity)> {
        findings.iter().map(|f| (f.code, f.severity)).collect()
    }

    fn synced_user_library(conn: &Connection) {
        conn.execute("INSERT INTO library (id, kind, zotero_id, name) VALUES (1, 'user', 0, 'My Library')", []).unwrap();
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
        assert_eq!(codes(&findings), [("database", Severity::Info), ("zotero", Severity::Info), ("never_synced", Severity::Warning)]);
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
        assert!(library.message.starts_with("user \"My Library\": 1 active sources, last sync"), "{}", library.message);
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
        conn.execute("INSERT INTO review_queue (kind, message, dedupe_key) VALUES ('x', 'm', 'x:1')", []).unwrap();
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
            let findings = run(&input(&conn, Some(Project { root: dir.path(), config: &config }))).unwrap();
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
        let queued: i64 = conn.query_row("SELECT count(*) FROM review_queue WHERE kind = 'key_conflict'", [], |r| r.get(0)).unwrap();
        assert_eq!(queued, 1, "conflict is queued once across runs");
    }

    #[test]
    fn unreadable_bib_file_is_an_error() {
        let conn = db();
        let dir = tempfile::tempdir().unwrap();
        let config = project_config(Some("missing.bib"));
        let findings = run(&input(&conn, Some(Project { root: dir.path(), config: &config }))).unwrap();
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
```

Run: `cargo test -p bib-core doctor`
Expected: FAIL, `not yet implemented`.

- [ ] **Step 2: Implement**

Replace `run` and `format_age`, add helpers:

```rust
pub fn run(input: &DoctorInput) -> anyhow::Result<Vec<Finding>> {
    let conn = input.conn;
    let mut findings = Vec::new();
    let mut push = |severity: Severity, code: &'static str, message: String| {
        findings.push(Finding { severity, code, message })
    };

    let version = crate::db::schema_version(conn)?;
    push(Severity::Info, "database", format!("{} (schema version {version})", input.db_path.display()));
    match &input.zotero {
        Ok(()) => push(Severity::Info, "zotero", "local API reachable".to_string()),
        Err(message) => {
            push(Severity::Warning, "zotero_unreachable", format!("{message}; working from the last synced state"))
        }
    }

    let libraries = library_status(conn)?;
    if libraries.is_empty() {
        push(Severity::Warning, "never_synced", "no Zotero library synced yet; run `bib sync`".to_string());
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
            None => push(Severity::Warning, "never_synced", format!("{} has never been synced successfully", library.library)),
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
            format!("{} source(s) without citation key, not citable: {}", without_key.len(), list(&without_key)),
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
            format!("{} PDF attachment(s) point to missing files: {}", missing.len(), list(&missing)),
        );
    }
    let without_file = count(conn, "SELECT count(*) FROM attachment WHERE status = 'active' AND path IS NULL")?;
    if without_file > 0 {
        push(Severity::Info, "pdf_without_file", format!("{without_file} PDF attachment(s) have no local file in Zotero"));
    }
    let retired = count(conn, "SELECT count(*) FROM source WHERE status = 'retired'")?;
    if retired > 0 {
        push(Severity::Info, "retired_sources", format!("{retired} source(s) no longer in Zotero are kept as retired"));
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
        statuses.push(LibraryStatus { library, name, active_sources, age });
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
    push(Severity::Info, "project", format!("{} ({})", config.name, project.root.display()));
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

    let Some(bibliography) = &config.bibliography else { return Ok(()) };
    let path = project.root.join(&bibliography.path);
    let entries = match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|text| parse_bib(&text)) {
        Ok(entries) => entries,
        Err(message) => {
            push(Severity::Error, "bib_unreadable", format!("{}: {message}", path.display()));
            return Ok(());
        }
    };
    for finding in compare(&entries, &keyed_sources(conn, &order)?) {
        match finding {
            BibFinding::OnlyInBib { key } => push(
                Severity::Warning,
                "bib_only",
                format!("{key} is in {} but not in Zotero; add it in Zotero", bibliography.path.display()),
            ),
            BibFinding::MetadataDiffers { key, field, bib, zotero } => push(
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
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
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
```

`push` is a closure capturing `findings` mutably, so `findings` can only be returned after its last use. If the borrow checker objects to `Ok(findings)` while `push` is alive, end the closure's scope explicitly (`drop(push);`) right before returning. The `(SELECT … max(r.finished_at)) … WHERE status = 'ok'` subquery yields NULL for libraries without a successful run, which maps to `age: None`. `ORDER BY l.kind = 'group'` lists the user library first.

Run: `cargo test -p bib-core doctor`
Expected: PASS, 7 tests.

- [ ] **Step 3: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

```bash
git add crates/bib-core
git commit -m "Add doctor checks for database, sync state and project

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 8: CLI `bib` — init, sync, doctor, backup

**Files:**
- Modify: `Cargo.toml` (member `crates/bib-cli`)
- Create: `crates/bib-cli/Cargo.toml`, `crates/bib-cli/src/main.rs`, `crates/bib-cli/tests/cli.rs`

**Interfaces:**
- Consumes: `paths::{db_path, backup_dir}`, `db::{open, backup, prune_backups, KEEP_BACKUPS, Connection, Opened}` (Task 2); `project::{CONFIG_FILE, ProjectConfig, find_project_root, init_project}` (Task 3); `zotero::{ZoteroClient, ZoteroError}`, `zotero::fake` (Task 4); `zotero::sync::{sync_all, last_successful_sync_age, LibraryReport}` (Task 5); `doctor::{run, format_age, DoctorInput, Project, Severity, Finding}` (Task 7).
- Produces: binary `bib` with subcommands `init [--no-project]`, `sync`, `doctor`, `backup` and global `--json`. Exit codes: 0 success, 1 doctor found warnings or errors, 2 failure. With `--json`, stdout carries exactly one JSON document; failures print `{"error": "..."}` to stdout.

- [ ] **Step 1: Crate**

`Cargo.toml` → `members` add `"crates/bib-cli"` (after `"crates/bib-core"`).

`crates/bib-cli/Cargo.toml`:

```toml
[package]
name = "bib-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[[bin]]
name = "bib"
path = "src/main.rs"

[dependencies]
anyhow.workspace = true
bib-core = { path = "../bib-core" }
clap.workspace = true
serde_json.workspace = true

[dev-dependencies]
bib-core = { path = "../bib-core", features = ["test-support"] }
serde_json.workspace = true
tempfile.workspace = true
```

- [ ] **Step 2: End-to-end tests**

`crates/bib-cli/tests/cli.rs`:

```rust
//! Runs the real `bib` binary against a temporary data directory and the fake Zotero.

use std::path::Path;
use std::process::{Command, Output};

use bib_core::zotero::fake::{self, FakeZotero};
use serde_json::Value;

struct Env {
    data: tempfile::TempDir,
    project: tempfile::TempDir,
    zotero_url: String,
}

impl Env {
    fn new(zotero_url: &str) -> Env {
        Env { data: tempfile::tempdir().unwrap(), project: tempfile::tempdir().unwrap(), zotero_url: zotero_url.to_string() }
    }

    fn bib(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_bib"))
            .args(args)
            .current_dir(self.project.path())
            .env("BIB_DATA_DIR", self.data.path())
            .env("BIB_ZOTERO_URL", &self.zotero_url)
            .env_remove("XDG_DATA_HOME")
            .output()
            .expect("run bib")
    }
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn closed_port_url() -> String {
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    format!("http://127.0.0.1:{port}")
}

fn write_config(dir: &Path, extra: &str) {
    std::fs::write(dir.join("bib.toml"), format!("id = \"p1\"\nname = \"Test\"\n{extra}")).unwrap();
}

#[test]
fn init_creates_database_and_project_file_once() {
    let env = Env::new(&closed_port_url());
    let first = env.bib(&["init"]);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert!(env.data.path().join("bib.db").exists());
    let config = std::fs::read_to_string(env.project.path().join("bib.toml")).unwrap();
    assert!(config.contains("libraries = [\"user\"]"));

    let second = env.bib(&["init", "--json"]);
    assert!(second.status.success());
    let report = json(&second);
    assert_eq!(report["created"], false);
    assert_eq!(report["schema_version"], 1);
}

#[test]
fn init_without_project_leaves_the_directory_alone() {
    let env = Env::new(&closed_port_url());
    assert!(env.bib(&["init", "--no-project"]).status.success());
    assert!(!env.project.path().join("bib.toml").exists());
}

#[test]
fn sync_then_doctor() {
    let zotero = FakeZotero::start();
    zotero.user_library(
        vec![fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", ""), fake::item("NOKEY001", "", "Untitled", "2020", "")],
        vec![],
        vec![],
    );
    let env = Env::new(zotero.url());

    let sync = env.bib(&["sync", "--json"]);
    assert!(sync.status.success(), "{}", String::from_utf8_lossy(&sync.stdout));
    let report = json(&sync);
    assert_eq!(report["libraries"][0]["library"], "user");
    assert_eq!(report["libraries"][0]["counts"]["sources_new"], 2);

    let doctor = env.bib(&["doctor", "--json"]);
    assert_eq!(doctor.status.code(), Some(1), "warnings exit with 1");
    let findings = json(&doctor);
    let codes: Vec<&str> = findings.as_array().unwrap().iter().map(|f| f["code"].as_str().unwrap()).collect();
    assert!(codes.contains(&"missing_citation_key"), "{codes:?}");
    assert!(codes.contains(&"library"), "{codes:?}");
    assert!(!codes.contains(&"zotero_unreachable"), "{codes:?}");

    let text = env.bib(&["doctor"]);
    assert!(String::from_utf8_lossy(&text.stdout).contains("[warning] missing_citation_key:"));
}

#[test]
fn doctor_compares_the_project_bib_file() {
    let zotero = FakeZotero::start();
    zotero.user_library(vec![fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", "")], vec![], vec![]);
    let env = Env::new(zotero.url());
    write_config(env.project.path(), "[bibliography]\npath = \"refs.bib\"\n");
    std::fs::write(env.project.path().join("refs.bib"), "@article{ghost2019, title = {Not in Zotero}}\n").unwrap();
    let nested = env.project.path().join("chapters");
    std::fs::create_dir(&nested).unwrap();
    assert!(env.bib(&["sync"]).status.success());

    let doctor = Command::new(env!("CARGO_BIN_EXE_bib"))
        .args(["doctor", "--json"])
        .current_dir(&nested)
        .env("BIB_DATA_DIR", env.data.path())
        .env("BIB_ZOTERO_URL", zotero.url())
        .output()
        .unwrap();
    let codes: Vec<String> =
        json(&doctor).as_array().unwrap().iter().map(|f| f["code"].as_str().unwrap().to_string()).collect();
    assert!(codes.contains(&"project".to_string()), "project found from a subdirectory: {codes:?}");
    assert!(codes.contains(&"bib_only".to_string()), "{codes:?}");
}

#[test]
fn sync_without_zotero_fails_with_the_last_sync_age() {
    let env = Env::new(&closed_port_url());
    let output = env.bib(&["sync"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not reachable"), "{stderr}");
    assert!(stderr.contains("Last successful sync: never"), "{stderr}");

    let json_output = env.bib(&["sync", "--json"]);
    assert_eq!(json_output.status.code(), Some(2));
    assert!(json(&json_output)["error"].as_str().unwrap().contains("not reachable"));
}

#[test]
fn invalid_project_file_is_a_failure() {
    let env = Env::new(&closed_port_url());
    write_config(env.project.path(), "[zotero]\nlibraries = [\"group:abc\"]\n");
    let output = env.bib(&["doctor"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("group:abc"));
}

#[test]
fn backup_writes_a_copy() {
    let env = Env::new(&closed_port_url());
    let output = env.bib(&["backup", "--json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let path = json(&output)["backup"].as_str().unwrap().to_string();
    assert!(Path::new(&path).starts_with(env.data.path().join("backups")));
    assert!(Path::new(&path).exists());
}
```

`crates/bib-cli/src/main.rs` (temporary, so the test binary builds):

```rust
fn main() {}
```

Run: `cargo test -p bib-cli`
Expected: FAIL — all tests fail (the binary does nothing and exits 0 with empty output).

- [ ] **Step 3: Implement the CLI**

`crates/bib-cli/src/main.rs`:

```rust
//! The `bib` command line (spec §10). Exit codes: 0 success, 1 problems found, 2 failure.

use std::path::Path;
use std::process::ExitCode;

use bib_core::db::{self, Connection, Opened};
use bib_core::doctor::{self, DoctorInput, Finding, Project, Severity};
use bib_core::paths;
use bib_core::project::{CONFIG_FILE, ProjectConfig, find_project_root, init_project};
use bib_core::zotero::sync::{LibraryReport, last_successful_sync_age, sync_all};
use bib_core::zotero::{ZoteroClient, ZoteroError};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(name = "bib", version, about = "Bibliography manager: Zotero sync, quote verification, Typst integration")]
struct Cli {
    /// Print one JSON document instead of text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create or migrate the database and write a `bib.toml` into the current directory.
    Init {
        /// Only set up the database.
        #[arg(long)]
        no_project: bool,
    },
    /// Read all Zotero libraries (read-only) and update the database.
    Sync,
    /// Check the database, the Zotero sync and the current project.
    Doctor,
    /// Write a consistent copy of the database into the backup directory.
    Backup,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(code) => code,
        Err(err) => {
            if cli.json {
                println!("{}", json!({"error": format!("{err:#}")}));
            } else {
                eprintln!("error: {err:#}");
            }
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> anyhow::Result<ExitCode> {
    let db_path = paths::db_path();
    let (mut conn, opened) = db::open(&db_path, &paths::backup_dir())?;
    if let Some(backup) = &opened.backup
        && !cli.json
    {
        eprintln!("backed up the database to {} before migrating", backup.display());
    }
    match &cli.command {
        Command::Init { no_project } => init(cli, &db_path, &opened, *no_project),
        Command::Sync => sync(cli, &mut conn),
        Command::Doctor => doctor(cli, &conn, &db_path),
        Command::Backup => backup(cli, &conn),
    }
}

fn init(cli: &Cli, db_path: &Path, opened: &Opened, no_project: bool) -> anyhow::Result<ExitCode> {
    let dir = std::env::current_dir()?;
    let config_path = dir.join(CONFIG_FILE);
    let created = if no_project || config_path.exists() {
        false
    } else {
        init_project(&dir)?;
        true
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "database": db_path,
                "schema_version": opened.to_version,
                "project_file": (!no_project).then_some(&config_path),
                "created": created,
            })
        );
    } else {
        println!("database {} (schema version {})", db_path.display(), opened.to_version);
        if created {
            println!("created {}", config_path.display());
        } else if !no_project {
            println!("{} already exists, left unchanged", config_path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn sync(cli: &Cli, conn: &mut Connection) -> anyhow::Result<ExitCode> {
    let client = ZoteroClient::from_env();
    let reports = match sync_all(conn, &client) {
        Ok(reports) => reports,
        Err(err) if matches!(err.downcast_ref::<ZoteroError>(), Some(ZoteroError::Unreachable { .. })) => {
            let last = match last_successful_sync_age(conn)? {
                Some(age) => format!("{} ago", doctor::format_age(age)),
                None => "never".to_string(),
            };
            anyhow::bail!(
                "{err}. Last successful sync: {last}. Start Zotero and enable \"Allow other applications on this computer to communicate with Zotero\" (Settings → Advanced)."
            );
        }
        Err(err) => return Err(err),
    };
    if cli.json {
        println!("{}", json!({ "libraries": reports }));
    } else {
        for report in &reports {
            print_report(report);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_report(report: &LibraryReport) {
    let c = &report.counts;
    println!(
        "{} \"{}\": {} new, {} changed, {} retired, {} reactivated, {} PDF attachments",
        report.library, report.name, c.sources_new, c.sources_changed, c.sources_retired, c.sources_reactivated, c.attachments
    );
}

fn doctor(cli: &Cli, conn: &Connection, db_path: &Path) -> anyhow::Result<ExitCode> {
    let zotero = ZoteroClient::from_env().groups().map(|_| ()).map_err(|e| e.to_string());
    let root = find_project_root(&std::env::current_dir()?);
    let config = root.as_deref().map(ProjectConfig::load).transpose()?;
    let project = root.as_deref().zip(config.as_ref()).map(|(root, config)| Project { root, config });
    let findings = doctor::run(&DoctorInput { conn, db_path, zotero, project })?;
    if cli.json {
        println!("{}", serde_json::to_string(&findings)?);
    } else {
        findings.iter().for_each(print_finding);
    }
    let problems = findings.iter().any(|f| f.severity >= Severity::Warning);
    Ok(if problems { ExitCode::from(1) } else { ExitCode::SUCCESS })
}

fn print_finding(finding: &Finding) {
    let label = match finding.severity {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    };
    println!("[{label}] {}: {}", finding.code, finding.message);
}

fn backup(cli: &Cli, conn: &Connection) -> anyhow::Result<ExitCode> {
    let dir = paths::backup_dir();
    let path = db::backup(conn, &dir)?;
    db::prune_backups(&dir, db::KEEP_BACKUPS)?;
    if cli.json {
        println!("{}", json!({ "backup": path }));
    } else {
        println!("backup written to {}", path.display());
    }
    Ok(ExitCode::SUCCESS)
}
```

Run: `cargo test -p bib-cli`
Expected: PASS, 7 tests.

- [ ] **Step 4: Manual check of the help text**

Run: `cargo run -q -p bib-cli -- --help`
Expected: lists `init`, `sync`, `doctor`, `backup` and `--json`.

- [ ] **Step 5: Checks and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

```bash
git add Cargo.toml Cargo.lock crates/bib-cli
git commit -m "Add bib CLI with init, sync, doctor and backup

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 9: Read-only smoke test against the real Zotero, and spec update

Runs the finished tool against the user's real Zotero with a throwaway data directory. Only counts are recorded; no titles, keys or paths from the user's library go into the repository.

**Files:**
- Create: `docs/research/15-zotero-sync-smoke-test.md`
- Modify: `docs/superpowers/specs/2026-09-16-library-core-typst-design.md` (§6, §10, §17, §18)

**Interfaces:**
- Consumes: binary `bib` (Task 8).
- Produces: measured sync numbers for plan 3; spec reflects the decisions of this plan.

- [ ] **Step 1: Check that Zotero is running**

Run: `curl -s -o /dev/null -w "%{http_code}\n" 'http://127.0.0.1:23119/api/users/0/items/top?limit=1'`
Expected: `200`. If not, stop and ask the user to start Zotero (do not start or configure it yourself) — then continue.

- [ ] **Step 2: Run the tool with a throwaway data directory**

```bash
export BIB_DATA_DIR=$(mktemp -d)
cargo build --release -p bib-cli
BIB=target/release/bib
$BIB init --no-project
time $BIB sync
$BIB sync --json > "$BIB_DATA_DIR/second-sync.json"
$BIB doctor --json > "$BIB_DATA_DIR/doctor.json"; echo "doctor exit $?"
```

Expected: first sync lists `user` and every group library; the second sync reports 0 new, 0 changed, 0 retired for every library (a real change in Zotero between the two runs is the only acceptable exception — then note it).

- [ ] **Step 3: Measure a project with all libraries**

```bash
PROJECT=$(mktemp -d)
GROUPS=$(sqlite3 "$BIB_DATA_DIR/bib.db" "SELECT group_concat('\"group:' || zotero_id || '\"', ', ') FROM library WHERE kind = 'group'")
printf 'id = "smoke"\nname = "Smoke test"\n[zotero]\nlibraries = ["user", %s]\n' "$GROUPS" > "$PROJECT/bib.toml"
(cd "$PROJECT" && $OLDPWD/$BIB doctor --json > "$BIB_DATA_DIR/doctor-project.json"; echo "exit $?")
```

If `sqlite3` is not installed, read the group ids from the first sync's text output instead.

Collect only counts with `sqlite3`/`jq` or a short Python snippet: libraries; active sources per library; sources without citation key; PDF attachments with a path, without a path, with a missing file; skipped top-level items; citation keys shared across libraries (`SELECT count(*) FROM (SELECT citation_key FROM source WHERE citation_key IS NOT NULL GROUP BY citation_key HAVING count(DISTINCT library_id) > 1)`); `key_conflict` findings in the project doctor run (keys that are different works); duration of the first sync.

- [ ] **Step 4: Write the protocol**

`docs/research/15-zotero-sync-smoke-test.md` (fill every `<…>` with the measured value; no titles, item keys, citation keys or file paths):

```markdown
# Zotero sync smoke test (plan 2, task 9)

Date: <date>. Zotero <version from `X-Zotero-Version` header>, `bib` at commit <short sha>. Read-only; data
directory was a temporary directory and has been deleted.

| Measure | Value |
|---|---|
| Libraries (user + groups) | <n> (<n> groups) |
| Active sources per library | user <n>, groups <n, n, …> |
| Top-level items skipped (notes, standalone attachments) | <n> |
| Sources without citation key | <n> |
| PDF attachments: with local file / without file / file missing | <n> / <n> / <n> |
| Citation keys in more than one library | <n> (spec §17 measured 17) |
| Of those, different works (`key_conflict`) | <n> |
| First sync duration | <s> |
| Second sync: new / changed / retired | <n> / <n> / <n> |

## Observations

<Anything unexpected: parse errors, item types, slow endpoints, differences from spec §17.>
```

- [ ] **Step 5: Update the spec**

In `docs/superpowers/specs/2026-09-16-library-core-typst-design.md`:

- §6, first bullet list: after "Full sync instead of incremental", add a bullet: "**Endpoints:** sources from `/items/top` (skipping notes, standalone attachments and annotations), PDFs from `/items?itemType=attachment` joined via `parentItem`, collections from `/collections`, groups from `/users/0/groups`; at most 100 items per page, `Total-Results` gives the total (`/items` alone returns every item type: 603 rows against 168 top-level items)."
- §6 "Fallback" bullet: append "Deferred (plan 2): until then, the tool works from the last synced state."
- §10 Configuration: after the sentence about `~/.config/bib/config.toml`, add "Environment: `BIB_DATA_DIR` overrides the data directory (default `$XDG_DATA_HOME/bib` or `~/.local/share/bib`), `BIB_ZOTERO_URL` the Zotero address (default `http://127.0.0.1:23119`)."
- §17: add a row "Zotero sync smoke test (plan 2), `docs/research/15-zotero-sync-smoke-test.md`" with the key numbers (libraries, sources, keys in several libraries, conflicts, sync duration).
- §18: add "`zotero.sqlite` fallback when Zotero isn't running: deferred from plan 2." and "\"Key only in Zotero, citation won't compile\" (§6) needs the cited keys and is implemented with the Typst parser (step 5)."

- [ ] **Step 6: Clean up and commit**

```bash
rm -rf "$BIB_DATA_DIR" "$PROJECT"
unset BIB_DATA_DIR
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs/research/15-zotero-sync-smoke-test.md docs/superpowers/specs/2026-09-16-library-core-typst-design.md
git commit -m "Record Zotero sync smoke test and update spec

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```
