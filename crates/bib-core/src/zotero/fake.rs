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
    let param = |name: &str| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
    };
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
