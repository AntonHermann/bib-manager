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
            assert_eq!(
                text.parse::<LibraryRef>(),
                Err(ParseLibraryRefError(text.to_string())),
                "{text:?}"
            );
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
