//! Read-only access to the Zotero local API (spec §6).

mod client;
#[cfg(any(test, feature = "test-support"))]
pub mod fake;
mod model;
pub mod sync;

pub use client::{DEFAULT_URL, URL_ENV, ZoteroClient, ZoteroError};
pub use model::{
    Attachment, Collection, Group, Item, NON_SOURCE_TYPES, file_url_to_path, parse_attachment, parse_collection,
    parse_group, parse_item, year_from_date,
};
