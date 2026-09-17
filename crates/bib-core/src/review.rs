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
        params![
            item.kind,
            item.node_type,
            item.node_id,
            item.message,
            item.details.to_string(),
            item.dedupe_key
        ],
    )?;
    Ok(inserted == 1)
}

pub fn open_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT count(*) FROM review_queue WHERE resolved_at IS NULL",
        [],
        |row| row.get(0),
    )
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
        conn.execute("UPDATE review_queue SET resolved_at = 'now', resolution = 'fixed'", [])
            .unwrap();
        assert!(enqueue(&conn, &item(&details)).unwrap());
        assert_eq!(open_count(&conn).unwrap(), 1);
    }
}
