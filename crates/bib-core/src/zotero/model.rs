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
    v.get("data")
        .filter(|d| d.is_object())
        .ok_or_else(|| ZoteroError::Parse(format!("{what} without `data` object")))
}

fn text(data: &Value, field: &str) -> String {
    data.get(field).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Trimmed, non-empty string field.
fn non_empty(data: &Value, field: &str) -> Option<String> {
    data.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn key_of(data: &Value, what: &str) -> Result<String, ZoteroError> {
    non_empty(data, "key").ok_or_else(|| ZoteroError::Parse(format!("{what} without `key`")))
}

fn strings(value: Option<&Value>, pick: impl Fn(&Value) -> Option<&str>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(&pick).map(str::to_string).collect())
        .unwrap_or_default()
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
        library_name: v
            .pointer("/library/name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
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
        path: v
            .pointer("/links/enclosure/href")
            .and_then(Value::as_str)
            .and_then(file_url_to_path),
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
    Ok(Group {
        id,
        name: v
            .pointer("/data/name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
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
        assert_eq!(
            a.path,
            Some(PathBuf::from("/home/u/Zotero/storage/ATT00001/Wiest et al. - 2024.pdf"))
        );
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
        assert_eq!(
            parse_group(&group).unwrap(),
            Group {
                id: 6573630,
                name: "Lab".to_string()
            }
        );
    }

    #[test]
    fn missing_data_is_a_parse_error() {
        assert!(matches!(parse_item(&json!({"key": "K"})), Err(ZoteroError::Parse(_))));
    }

    #[test]
    fn file_urls() {
        assert_eq!(
            file_url_to_path("file:///a/b%20c.pdf"),
            Some(PathBuf::from("/a/b c.pdf"))
        );
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
