//! Corpus description (`bench/corpus.toml`) and checksums (`bench/corpus.lock`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Corpus {
    #[serde(rename = "doc")]
    pub docs: Vec<Doc>,
}

#[derive(Debug, Deserialize)]
pub struct Doc {
    pub id: String,
    pub url: String,
    pub category: String,
    #[serde(default)]
    pub expect: Expect,
}

/// Hand-curated assertions about a document's text.
#[derive(Debug, Default, Deserialize)]
pub struct Expect {
    /// Sentences that must appear contiguously in the normalized text.
    #[serde(default)]
    pub sentences: Vec<String>,
    /// Sentence pairs: the first must come before the second (reading order).
    #[serde(default)]
    pub before: Vec<(String, String)>,
    /// Characters that must occur at least once (e.g. "ε").
    #[serde(default)]
    pub chars: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LockStatus {
    New,
    Match,
    Mismatch { expected: String },
}

pub fn parse_corpus(s: &str) -> anyhow::Result<Corpus> {
    Ok(toml::from_str(s)?)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

pub fn check_lock(lock: &BTreeMap<String, String>, id: &str, sha: &str) -> LockStatus {
    match lock.get(id) {
        None => LockStatus::New,
        Some(expected) if expected == sha => LockStatus::Match,
        Some(expected) => LockStatus::Mismatch {
            expected: expected.clone(),
        },
    }
}

pub fn bench_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bench")
}

pub fn cache_path(id: &str) -> PathBuf {
    bench_dir().join("cache").join(format!("{id}.pdf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docs_with_and_without_expectations() {
        let corpus = parse_corpus(
            r#"
            [[doc]]
            id = "a"
            url = "https://example.org/a.pdf"
            category = "test"
            [doc.expect]
            sentences = ["One two three."]
            before = [["One", "three"]]
            chars = ["ε"]

            [[doc]]
            id = "b"
            url = "https://example.org/b.pdf"
            category = "test"
            "#,
        )
        .unwrap();
        assert_eq!(corpus.docs.len(), 2);
        assert_eq!(corpus.docs[0].expect.sentences, vec!["One two three."]);
        assert_eq!(
            corpus.docs[0].expect.before,
            vec![("One".to_string(), "three".to_string())]
        );
        assert_eq!(corpus.docs[0].expect.chars, vec!["ε"]);
        assert!(corpus.docs[1].expect.sentences.is_empty());
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn lock_statuses() {
        let lock = BTreeMap::from([("a".to_string(), "111".to_string())]);
        assert_eq!(check_lock(&lock, "a", "111"), LockStatus::Match);
        assert_eq!(
            check_lock(&lock, "a", "222"),
            LockStatus::Mismatch {
                expected: "111".to_string()
            }
        );
        assert_eq!(check_lock(&lock, "b", "333"), LockStatus::New);
    }
}
