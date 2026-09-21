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
        ZoteroClient {
            agent,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
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
        self.get_all(&format!("/api{}/items/top", library.api_prefix()))?
            .iter()
            .map(parse_item)
            .collect()
    }

    /// All attachment items, child and standalone.
    pub fn attachments(&self, library: LibraryRef) -> Result<Vec<Attachment>, ZoteroError> {
        self.get_all(&format!("/api{}/items?itemType=attachment", library.api_prefix()))?
            .iter()
            .map(parse_attachment)
            .collect()
    }

    pub fn collections(&self, library: LibraryRef) -> Result<Vec<Collection>, ZoteroError> {
        self.get_all(&format!("/api{}/collections", library.api_prefix()))?
            .iter()
            .map(parse_collection)
            .collect()
    }

    /// Fetches every page of a JSON array endpoint.
    fn get_all(&self, path_and_query: &str) -> Result<Vec<Value>, ZoteroError> {
        let separator = if path_and_query.contains('?') { '&' } else { '?' };
        let mut all = Vec::new();
        loop {
            let url = format!(
                "{}{path_and_query}{separator}format=json&limit={PAGE_SIZE}&start={}",
                self.base_url,
                all.len()
            );
            let mut response = self.agent.get(&url).call().map_err(|e| ZoteroError::Unreachable {
                url: self.base_url.clone(),
                message: e.to_string(),
            })?;
            let status = response.status().as_u16();
            if status != 200 {
                return Err(ZoteroError::Http { status, url });
            }
            let total = response
                .headers()
                .get("Total-Results")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<usize>().ok());
            let page: Vec<Value> = response
                .body_mut()
                .read_json()
                .map_err(|e| ZoteroError::Parse(format!("{url}: {e}")))?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zotero::fake::{self, FakeZotero};

    #[test]
    fn pages_through_all_results() {
        let zotero = FakeZotero::start();
        let items = (0..250)
            .map(|i| fake::item(&format!("K{i:07}"), "", "t", "2020", ""))
            .collect();
        zotero.set("/api/users/0/items/top", items);
        let fetched = ZoteroClient::new(zotero.url()).top_items(LibraryRef::User).unwrap();
        assert_eq!(fetched.len(), 250);
        assert_eq!(fetched[0].key, "K0000000");
        assert_eq!(fetched[249].key, "K0000249");
    }

    #[test]
    fn uses_group_paths_and_the_attachment_filter() {
        let zotero = FakeZotero::start();
        zotero.set(
            "/api/groups/7/items?itemType=attachment",
            vec![fake::attachment("A1", "P1", "file:///x.pdf")],
        );
        zotero.set("/api/groups/7/collections", vec![fake::collection("C1", "Lab", None)]);
        zotero.set("/api/users/0/groups", vec![fake::group(7, "Lab")]);
        let client = ZoteroClient::new(zotero.url());
        assert_eq!(client.attachments(LibraryRef::Group(7)).unwrap()[0].key, "A1");
        assert_eq!(client.collections(LibraryRef::Group(7)).unwrap()[0].name, "Lab");
        assert_eq!(
            client.groups().unwrap(),
            vec![Group {
                id: 7,
                name: "Lab".to_string()
            }]
        );
    }

    #[test]
    fn empty_result_is_fine() {
        let zotero = FakeZotero::start();
        zotero.set("/api/users/0/collections", Vec::new());
        assert!(
            ZoteroClient::new(zotero.url())
                .collections(LibraryRef::User)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn http_errors_carry_the_status() {
        let zotero = FakeZotero::start();
        let err = ZoteroClient::new(zotero.url())
            .top_items(LibraryRef::Group(9))
            .unwrap_err();
        assert!(matches!(err, ZoteroError::Http { status: 404, .. }), "{err}");
    }

    #[test]
    fn closed_port_is_unreachable() {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let err = ZoteroClient::new(&format!("http://127.0.0.1:{port}/"))
            .groups()
            .unwrap_err();
        assert!(matches!(err, ZoteroError::Unreachable { .. }), "{err}");
        assert!(err.to_string().contains("not reachable"));
    }
}
