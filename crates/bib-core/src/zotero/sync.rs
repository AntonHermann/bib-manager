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
    let groups = client.groups()?;
    let mut targets = vec![(LibraryRef::User, None)];
    targets.extend(
        groups
            .into_iter()
            .map(|group| (LibraryRef::Group(group.id), Some(group.name))),
    );
    // Every target is attempted regardless of earlier failures: sync_library upserts the `library`
    // row and a `sync_run` row internally, so skipping a library here (as an eager-stop `collect`
    // would) leaves it entirely invisible to `bib doctor` and permanently blocks every library
    // ordered after it.
    let mut reports = Vec::with_capacity(targets.len());
    let mut failures = Vec::new();
    for (library, name) in targets {
        match sync_library(conn, client, library, name) {
            Ok(report) => reports.push(report),
            Err(err) => failures.push((library, err)),
        }
    }
    if failures.is_empty() {
        return Ok(reports);
    }
    let detail = failures
        .iter()
        .map(|(library, err)| format!("{library}: {err:#}"))
        .collect::<Vec<_>>()
        .join("; ");
    anyhow::bail!(
        "sync failed for {} of {} libraries: {detail}",
        failures.len(),
        reports.len() + failures.len()
    )
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
    Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
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
    let name = name
        .or_else(|| items.first().map(|item| item.library_name.clone()))
        .unwrap_or_default();
    Ok(Snapshot {
        name,
        items,
        attachments,
        collections,
    })
}

fn upsert_library(conn: &Connection, library: LibraryRef, name: &str) -> rusqlite::Result<i64> {
    let (kind, zotero_id) = library.db_key();
    conn.execute(
        "INSERT INTO library (kind, zotero_id, name) VALUES (?1, ?2, ?3)
         ON CONFLICT (kind, zotero_id) DO UPDATE
         SET name = CASE WHEN excluded.name = '' THEN library.name ELSE excluded.name END",
        params![kind, zotero_id, name],
    )?;
    conn.query_row(
        "SELECT id FROM library WHERE kind = ?1 AND zotero_id = ?2",
        params![kind, zotero_id],
        |row| row.get(0),
    )
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
    Ok(LibraryReport {
        library,
        name: snapshot.name.clone(),
        counts,
    })
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
            Ok((
                row.get(0)?,
                StoredSource {
                    id: row.get(1)?,
                    data_hash: row.get(2)?,
                    retired: status == "retired",
                },
            ))
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
            tx.execute(
                "INSERT OR IGNORE INTO source_tag (source_id, tag) VALUES (?1, ?2)",
                params![id, tag],
            )?;
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
        let source_id = attachment
            .parent_key
            .as_ref()
            .and_then(|key| source_ids.get(key))
            .copied();
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
            params![
                library_id,
                attachment.key,
                source_id,
                attachment.content_type,
                attachment.link_mode,
                path
            ],
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
        sync_library(conn, &ZoteroClient::new(zotero.url()), LibraryRef::User, None)
            .unwrap()
            .counts
    }

    #[test]
    fn first_sync_stores_sources_tags_collections_and_pdfs() {
        let zotero = FakeZotero::start();
        let mut html = fake::attachment("ATTHTML1", "DWORK001", "file:///tmp/page.html");
        html["data"]["contentType"] = json!("text/html");
        zotero.user_library(
            vec![
                tagged(
                    fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", ""),
                    "A_core",
                    "COLL0001",
                ),
                fake::item(
                    "ABADI001",
                    "abadi2016",
                    "Deep Learning with Differential Privacy",
                    "2016",
                    "",
                ),
                fake::note("NOTE0001"),
            ],
            vec![
                fake::attachment("ATTPDF01", "DWORK001", "file:///tmp/zotero/a%20b.pdf"),
                html,
            ],
            vec![fake::collection("COLL0001", "Privacy", None)],
        );
        let mut conn = db();
        let counts = sync_user(&mut conn, &zotero);
        assert_eq!(
            counts,
            SyncCounts {
                sources_new: 2,
                attachments: 1,
                skipped_top_level: 1,
                ..SyncCounts::default()
            }
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
        assert_eq!(
            (path.as_str(), source_key.as_str()),
            ("/tmp/zotero/a b.pdf", "DWORK001")
        );
        let (name, status): (String, String) = conn
            .query_row(
                "SELECT l.name, r.status FROM sync_run r JOIN library l ON l.id = r.library_id",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((name.as_str(), status.as_str()), ("My Library", "ok"));
    }

    #[test]
    fn unchanged_resync_changes_nothing_and_changed_items_are_updated() {
        let zotero = FakeZotero::start();
        zotero.user_library(
            vec![fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", "")],
            vec![],
            vec![],
        );
        let mut conn = db();
        sync_user(&mut conn, &zotero);
        assert_eq!(sync_user(&mut conn, &zotero), SyncCounts::default());

        zotero.set(
            "/api/users/0/items/top",
            vec![fake::item("DWORK001", "dwork2006", "Differential privacy", "2006", "")],
        );
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
        assert_eq!(
            count(&conn, "SELECT count(*) FROM source"),
            1,
            "sources are never deleted"
        );
        assert_eq!(open_count(&conn).unwrap(), 1);
        let source_id: i64 = conn
            .query_row("SELECT id FROM source WHERE item_key = 'DWORK001'", [], |r| r.get(0))
            .unwrap();
        let (kind, node_type, node_id, dedupe_key): (String, String, i64, String) = conn
            .query_row(
                "SELECT kind, node_type, node_id, dedupe_key FROM review_queue WHERE resolved_at IS NULL",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(kind, "source_retired");
        assert_eq!(node_type, "source");
        assert_eq!(node_id, source_id);
        assert_eq!(
            dedupe_key,
            format!("source_retired:{}:{}", LibraryRef::User, "DWORK001")
        );

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
        conn.execute("UPDATE attachment SET checksum = 'abc', page_count = 3", [])
            .unwrap();
        sync_user(&mut conn, &zotero);
        assert_eq!(
            count(&conn, "SELECT count(*) FROM attachment WHERE checksum = 'abc'"),
            1
        );

        zotero.set(
            "/api/users/0/items?itemType=attachment",
            vec![fake::attachment("ATTPDF01", "DWORK001", "file:///tmp/b.pdf")],
        );
        sync_user(&mut conn, &zotero);
        assert_eq!(
            count(
                &conn,
                "SELECT count(*) FROM attachment WHERE checksum IS NULL AND page_count IS NULL"
            ),
            1
        );

        zotero.set("/api/users/0/items?itemType=attachment", vec![]);
        sync_user(&mut conn, &zotero);
        assert_eq!(
            count(&conn, "SELECT count(*) FROM attachment WHERE status = 'retired'"),
            1
        );
    }

    #[test]
    fn sync_all_covers_group_libraries_with_the_same_item_keys() {
        let zotero = FakeZotero::start();
        zotero.user_library(
            vec![fake::item("SAME0001", "dwork2006", "Differential Privacy", "2006", "")],
            vec![],
            vec![],
        );
        zotero.set("/api/users/0/groups", vec![fake::group(42, "Lab")]);
        zotero.set(
            "/api/groups/42/items/top",
            vec![fake::item("SAME0001", "dwork2006", "Differential Privacy", "2006", "")],
        );
        zotero.set("/api/groups/42/items?itemType=attachment", vec![]);
        zotero.set("/api/groups/42/collections", vec![]);
        let mut conn = db();
        let reports = sync_all(&mut conn, &ZoteroClient::new(zotero.url())).unwrap();
        let libraries: Vec<_> = reports.iter().map(|r| (r.library, r.name.as_str())).collect();
        assert_eq!(
            libraries,
            [(LibraryRef::User, "My Library"), (LibraryRef::Group(42), "Lab")]
        );
        assert_eq!(count(&conn, "SELECT count(*) FROM source"), 2);
        assert!(
            last_successful_sync_age(&conn)
                .unwrap()
                .is_some_and(|age| (0..60).contains(&age))
        );
    }

    #[test]
    fn failed_fetch_records_a_failed_run_and_keeps_the_error_type() {
        let zotero = FakeZotero::start();
        let mut conn = db();
        let err = sync_library(&mut conn, &ZoteroClient::new(zotero.url()), LibraryRef::User, None).unwrap_err();
        assert!(err.downcast_ref::<crate::zotero::ZoteroError>().is_some(), "{err:#}");
        let (status, error): (String, String) = conn
            .query_row("SELECT status, error FROM sync_run", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(status, "failed");
        assert!(error.contains("404"), "{error}");
        assert_eq!(last_successful_sync_age(&conn).unwrap(), None);
    }

    #[test]
    fn sync_all_attempts_every_library_even_after_an_earlier_failure() {
        let zotero = FakeZotero::start();
        zotero.set("/api/users/0/groups", vec![fake::group(42, "Lab")]);
        // The user library's `items/top` route is left unset (404), so the first target fails;
        // the group's routes are all set, so the second target must still be attempted and succeed.
        zotero.set(
            "/api/groups/42/items/top",
            vec![fake::item("GROUP001", "dwork2006", "Differential Privacy", "2006", "")],
        );
        zotero.set("/api/groups/42/items?itemType=attachment", vec![]);
        zotero.set("/api/groups/42/collections", vec![]);
        let mut conn = db();
        let err = sync_all(&mut conn, &ZoteroClient::new(zotero.url())).unwrap_err();
        assert!(err.to_string().contains("user"), "{err:#}");
        assert_eq!(
            count(&conn, "SELECT count(*) FROM library"),
            2,
            "the second library's row must exist even though the first library failed"
        );
        assert_eq!(count(&conn, "SELECT count(*) FROM sync_run"), 2);
        let (kind, status): (String, String) = conn
            .query_row(
                "SELECT l.kind, r.status FROM sync_run r JOIN library l ON l.id = r.library_id WHERE l.kind = 'group'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((kind.as_str(), status.as_str()), ("group", "ok"));
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
