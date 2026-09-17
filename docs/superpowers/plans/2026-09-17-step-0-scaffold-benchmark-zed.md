# Step 0: Scaffold, Extraction Benchmark, and Zed Test — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Set up a Rust workspace with normalization and swappable PDF backends, measure the backends against a curated corpus (step 0a), and check whether Zed cleanly merges a second Typst language server alongside tinymist (step 0b).

**Architecture:** A Cargo workspace. `bib-core` contains only the normalization for now (comparison form with mapping back to the raw text). `bib-extract` defines the backend interface (text mandatory, spans optional, a shared coordinate system) with four backends: `pdf_oxide`, `pdf-extract`, `mutool` as an external process, and `pdfium` behind a feature. `tools/extract-bench` loads the corpus, measures each backend against hand-curated assertions, and writes a Markdown report. `spikes/` contains throwaway code for the Zed test.

**Tech Stack:** Rust (Edition 2024, rustc 1.95), `unicode-normalization`, `pdf_oxide` 0.3.78, `pdf-extract` 0.12.0, `pdfium-render` 0.9.4, `thiserror`, `anyhow`, `clap`, `serde`, `toml`, `sha2`, `tempfile`, `tower-lsp-server` 0.23, `tokio`, `zed_extension_api` 0.7.0; external programs `curl`, `mutool` (optional).

**Spec:** `docs/superpowers/specs/2026-09-16-library-core-typst-design.md` (sections 7, 9, 14, 15, 17, 18)

**Follow-up plans** (to be written using the results of this plan): Plan 2 = spec steps 1–3 (data model, Zotero sync, extraction with cascade), Plan 3 = steps 4–6 (anchors, Typst parser, import), Plan 4 = steps 7–8 (language server, Zed extension, export).

## Global Constraints

- Project license: `MIT OR Apache-2.0`. **No AGPL dependency in the workspace** (no `mupdf-rs`, no PyMuPDF). `mutool` only as an optional external process.
- Rust: Edition 2024, `rust-version = "1.95"`.
- Normalization (comparison form), in this order and compatible with `check_quotes.py`: NFKC; remove soft hyphens (U+00AD); remove `-\s*\n\s*` (ASCII hyphen only); `‐‑‒–—` → `-`; `‘’‚‛` → `'`; `“”„‟` → `"`; collapse whitespace to a single space; trim leading and trailing whitespace; lowercase.
- Coordinates in `bib-extract`: points, origin top-left, y grows downward, relative to the MediaBox.
- Backend interface: text is mandatory, geometry optional (spec §7).
- Zed: the language is named exactly `Typst`; extensions are built for `wasm32-wasip2`; `zed_extension_api` 0.7.0 requires edition 2024.
- The seminar repo `~/Documents/seminar_ehr_ss26` is **not** read or used through 2026-09-23 inclusive.
- Changes to the Zed installation or `~/.config/zed/settings.json` only with the user's explicit consent.
- Documentation and comments in German, identifiers in English.
- Every commit ends with:
  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE
  ```

## File Structure

```
Cargo.toml                         Workspace
LICENSE-MIT, LICENSE-APACHE
.gitignore
crates/bib-core/
  Cargo.toml
  src/lib.rs                       Module list
  src/normalize.rs                 Comparison form + mapping back (Task 1)
crates/bib-extract/
  Cargo.toml
  src/lib.rs                       Types, backend trait, panic guard (Task 3)
  src/join.rs                      Spans → text (Task 3)
  src/backends/mod.rs
  src/backends/oxide.rs            pdf_oxide (Task 3)
  src/backends/pdf_extract.rs      pdf-extract, text only (Task 4)
  src/backends/mutool.rs           mutool as a process (Task 5)
  src/backends/stext.rs            parser for mutool's stext XML (Task 5)
  src/backends/pdfium.rs           pdfium, feature `pdfium` (Task 6)
  tests/common/mod.rs              Fixture paths
  tests/oxide.rs, tests/pdf_extract.rs, tests/coordinates.rs, tests/pdfium.rs
tools/extract-bench/
  Cargo.toml
  src/main.rs                      CLI: fetch, run
  src/corpus.rs                    corpus.toml + corpus.lock (Task 2)
  src/metrics.rs                   Scoring a text (Task 8)
  src/report.rs                    Markdown report (Task 8)
bench/
  corpus.toml                      Corpus + expected assertions (Task 2, 7)
  corpus.lock                      SHA-256 per document (Task 2, 7)
  README.md                        Curation rules, decision rule (Task 7, 10)
  results/                         Reports (Task 10)
  cache/                           Downloads, not version-controlled
spikes/lsp-coexist/                Throwaway language server (Task 9)
spikes/zed-coexist-ext/            Throwaway Zed extension, separate workspace (Task 9)
spikes/typst-sample/               Test document for Zed (Task 9)
docs/research/14-zed-two-language-servers.md   Log of the Zed test (Task 9)
```

---

### Task 1: Workspace Scaffold and Normalization

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `LICENSE-MIT`, `LICENSE-APACHE`
- Create: `crates/bib-core/Cargo.toml`, `crates/bib-core/src/lib.rs`, `crates/bib-core/src/normalize.rs`

**Interfaces:**
- Produces:
  - `bib_core::normalize::NORMALIZATION_VERSION: u32`
  - `bib_core::normalize::normalize(input: &str) -> Normalized`
  - `bib_core::normalize::Normalized { pub text: String }` with `fn source_range(&self, out: std::ops::Range<usize>) -> Option<std::ops::Range<usize>>` (byte range in the raw text for a byte range in `text`)

- [ ] **Step 1: Create the workspace**

`Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = [
    "crates/bib-core",
]
exclude = ["spikes/zed-coexist-ext", "docs/research/probes"]

[workspace.package]
version = "0.0.0"
edition = "2024"
rust-version = "1.95"
license = "MIT OR Apache-2.0"
publish = false

[workspace.dependencies]
unicode-normalization = "0.1.25"
```

`.gitignore`:

```gitignore
/target
bench/cache/
spikes/zed-coexist-ext/target
spikes/zed-coexist-ext/extension.wasm
```

`crates/bib-core/Cargo.toml`:

```toml
[package]
name = "bib-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
unicode-normalization.workspace = true
```

`crates/bib-core/src/lib.rs`:

```rust
//! Core of the Bibliography Manager.

pub mod normalize;
```

License files:

```bash
curl -sSfL https://www.apache.org/licenses/LICENSE-2.0.txt -o LICENSE-APACHE
cat > LICENSE-MIT <<'EOF'
MIT License

Copyright (c) 2026 Anton Oehler

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
```

- [ ] **Step 2: Write tests for normalization**

`crates/bib-core/src/normalize.rs` (tests and empty signatures only for now, so it compiles but fails):

```rust
//! Comparison form for wording search (spec §7, "normalization").
//!
//! Search always happens in the comparison form; [`Normalized::source_range`]
//! maps every match back to the raw text that anchors and boxes attach to.

use std::ops::Range;

/// Incremented whenever the result of [`normalize`] changes for any input.
pub const NORMALIZATION_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    pub text: String,
    /// For each byte of `text`: the byte range in the raw text it came from.
    spans: Vec<(usize, usize)>,
}

impl Normalized {
    pub fn source_range(&self, out: Range<usize>) -> Option<Range<usize>> {
        let _ = out;
        todo!()
    }
}

pub fn normalize(input: &str) -> Normalized {
    let _ = input;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epsilon_variants_become_greek_epsilon() {
        assert_eq!(normalize("ϵ-Differential 𝜖 𝜀").text, "ε-differential ε ε");
    }

    #[test]
    fn ligatures_are_expanded_and_map_to_source() {
        let n = normalize("ﬁnd");
        assert_eq!(n.text, "find");
        assert_eq!(n.source_range(0..2), Some(0..3));
        assert_eq!(n.source_range(2..3), Some(3..4));
    }

    #[test]
    fn line_end_hyphenation_is_joined() {
        assert_eq!(normalize("differ-\n   ential privacy").text, "differential privacy");
    }

    #[test]
    fn hyphen_without_newline_is_kept() {
        assert_eq!(normalize("state-of-the-art").text, "state-of-the-art");
    }

    #[test]
    fn en_dash_at_line_end_is_not_joined() {
        assert_eq!(normalize("1–\n2").text, "1- 2");
    }

    #[test]
    fn soft_hyphen_is_removed() {
        assert_eq!(normalize("pri\u{00AD}vacy").text, "privacy");
    }

    #[test]
    fn dashes_and_quotes_are_unified() {
        assert_eq!(normalize("“a” ‘b’ c—d").text, "\"a\" 'b' c-d");
    }

    #[test]
    fn whitespace_is_collapsed_and_trimmed() {
        assert_eq!(normalize("  a \t\n b  ").text, "a b");
    }

    #[test]
    fn combining_marks_are_composed() {
        assert_eq!(normalize("e\u{301}").text, "é");
    }

    #[test]
    fn source_range_covers_joined_hyphenation() {
        let raw = "differ-\n  ential";
        let n = normalize(raw);
        assert_eq!(n.text, "differential");
        assert_eq!(&raw[n.source_range(0..n.text.len()).unwrap()], raw);
    }

    #[test]
    fn empty_or_out_of_bounds_range_is_none() {
        let n = normalize("abc");
        assert_eq!(n.source_range(1..1), None);
        assert_eq!(n.source_range(2..9), None);
    }

    #[test]
    fn normalization_is_idempotent() {
        for s in [
            "ϵ-Diﬀerential\u{00AD} “Privacy”\n– tail-\n end",
            "  É\u{301}x  ",
            "",
        ] {
            let once = normalize(s).text;
            assert_eq!(normalize(&once).text, once, "input: {s:?}");
        }
    }
}
```

- [ ] **Step 3: Run tests, confirm failure**

Run: `cargo test -p bib-core`
Expected: FAIL, all tests abort with `not yet implemented`.

- [ ] **Step 4: Implement normalization**

In `crates/bib-core/src/normalize.rs`, replace the import and the two `todo!()` bodies:

```rust
use std::ops::Range;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
```

```rust
impl Normalized {
    pub fn source_range(&self, out: Range<usize>) -> Option<Range<usize>> {
        if out.start >= out.end || out.end > self.text.len() {
            return None;
        }
        Some(self.spans[out.start].0..self.spans[out.end - 1].1)
    }
}

/// A character of the intermediate stage: character, source range, may be joined at line end.
struct Unit {
    ch: char,
    start: usize,
    end: usize,
    joinable_hyphen: bool,
}

pub fn normalize(input: &str) -> Normalized {
    let units = map_characters(input);
    collapse(&units)
}

/// NFKC per segment (base character + combining marks), replacements, lowercasing.
fn map_characters(input: &str) -> Vec<Unit> {
    let mut units = Vec::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        let mut end = start + c.len_utf8();
        while let Some(&(i, next)) = chars.peek() {
            if !is_combining_mark(next) {
                break;
            }
            end = i + next.len_utf8();
            chars.next();
        }
        for n in input[start..end].nfkc() {
            let push = |units: &mut Vec<Unit>, ch: char, joinable_hyphen: bool| {
                units.push(Unit { ch, start, end, joinable_hyphen })
            };
            match n {
                '\u{00AD}' => {}
                '-' => push(&mut units, '-', true),
                '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' => {
                    push(&mut units, '-', false)
                }
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => push(&mut units, '\'', false),
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => push(&mut units, '"', false),
                other => {
                    for lower in other.to_lowercase() {
                        push(&mut units, lower, false);
                    }
                }
            }
        }
    }
    units
}

/// Join line-end hyphenation, collapse whitespace, trim.
fn collapse(units: &[Unit]) -> Normalized {
    let mut text = String::with_capacity(units.len());
    let mut spans = Vec::with_capacity(units.len());
    let mut pending_space: Option<(usize, usize)> = None;
    let mut i = 0;
    while i < units.len() {
        let unit = &units[i];
        if unit.joinable_hyphen {
            let mut j = i + 1;
            let mut saw_newline = false;
            while j < units.len() && units[j].ch.is_whitespace() {
                saw_newline |= units[j].ch == '\n';
                j += 1;
            }
            if saw_newline {
                i = j;
                continue;
            }
        }
        if unit.ch.is_whitespace() {
            pending_space.get_or_insert((unit.start, unit.end));
            i += 1;
            continue;
        }
        if let Some((start, end)) = pending_space.take() {
            if !text.is_empty() {
                push_char(&mut text, &mut spans, ' ', start, end);
            }
        }
        push_char(&mut text, &mut spans, unit.ch, unit.start, unit.end);
        i += 1;
    }
    Normalized { text, spans }
}

fn push_char(text: &mut String, spans: &mut Vec<(usize, usize)>, ch: char, start: usize, end: usize) {
    text.push(ch);
    spans.extend(std::iter::repeat_n((start, end), ch.len_utf8()));
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p bib-core`
Expected: PASS, 12 tests.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml .gitignore LICENSE-MIT LICENSE-APACHE crates/bib-core
git commit -m "Add workspace and comparison normalization with source mapping

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 2: Benchmark Tool with Corpus Download

**Files:**
- Modify: `Cargo.toml` (members, dependencies)
- Create: `tools/extract-bench/Cargo.toml`, `tools/extract-bench/src/main.rs`, `tools/extract-bench/src/corpus.rs`
- Create: `bench/corpus.toml`, `bench/corpus.lock`

**Interfaces:**
- Produces:
  - `corpus::Corpus { docs: Vec<Doc> }`, `corpus::Doc { id: String, url: String, category: String, expect: Expect }`, `corpus::Expect { sentences: Vec<String>, before: Vec<(String, String)>, chars: Vec<String> }`
  - `corpus::parse_corpus(s: &str) -> anyhow::Result<Corpus>`
  - `corpus::sha256_hex(bytes: &[u8]) -> String`
  - `corpus::LockStatus { New, Match, Mismatch { expected: String } }`, `corpus::check_lock(lock: &BTreeMap<String, String>, id: &str, sha: &str) -> LockStatus`
  - `corpus::bench_dir() -> PathBuf`, `corpus::cache_path(id: &str) -> PathBuf`
  - CLI: `cargo run -p extract-bench -- fetch`
  - File `bench/cache/<id>.pdf` for each corpus document

- [ ] **Step 1: Create the crate**

In `Cargo.toml`, add to `members` and add dependencies:

```toml
members = [
    "crates/bib-core",
    "tools/extract-bench",
]
```

```toml
[workspace.dependencies]
unicode-normalization = "0.1.25"
anyhow = "1.0"
clap = { version = "4.6", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
toml = "1.1"
sha2 = "0.11"
```

`tools/extract-bench/Cargo.toml`:

```toml
[package]
name = "extract-bench"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
anyhow.workspace = true
clap.workspace = true
serde.workspace = true
toml.workspace = true
sha2.workspace = true
bib-core = { path = "../../crates/bib-core" }
```

- [ ] **Step 2: Write tests for the corpus and lock file**

`tools/extract-bench/src/corpus.rs`:

```rust
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

pub fn parse_corpus(_s: &str) -> anyhow::Result<Corpus> {
    todo!()
}

pub fn sha256_hex(_bytes: &[u8]) -> String {
    todo!()
}

pub fn check_lock(_lock: &BTreeMap<String, String>, _id: &str, _sha: &str) -> LockStatus {
    todo!()
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
        assert_eq!(corpus.docs[0].expect.before, vec![("One".to_string(), "three".to_string())]);
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
            LockStatus::Mismatch { expected: "111".to_string() }
        );
        assert_eq!(check_lock(&lock, "b", "333"), LockStatus::New);
    }
}
```

`tools/extract-bench/src/main.rs` (temporary):

```rust
mod corpus;

fn main() {}
```

- [ ] **Step 3: Run tests, confirm failure**

Run: `cargo test -p extract-bench`
Expected: FAIL with `not yet implemented`.

- [ ] **Step 4: Implement corpus functions**

In `corpus.rs`, replace the three `todo!()` functions:

```rust
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
        Some(expected) => LockStatus::Mismatch { expected: expected.clone() },
    }
}
```

Run: `cargo test -p extract-bench`
Expected: PASS, 3 tests.

- [ ] **Step 5: Implement the `fetch` command**

`tools/extract-bench/src/main.rs`:

```rust
//! Extraction benchmark (spec §15, step 0a).

mod corpus;

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

use corpus::{LockStatus, bench_dir, cache_path, check_lock, parse_corpus, sha256_hex};

#[derive(Parser)]
#[command(about = "Measures PDF backends against a curated corpus")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Downloads all corpus documents into bench/cache/ and checks their SHA-256 sums.
    Fetch,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Cmd::Fetch => fetch(),
    }
}

fn load_corpus() -> anyhow::Result<corpus::Corpus> {
    let path = bench_dir().join("corpus.toml");
    parse_corpus(&std::fs::read_to_string(&path).with_context(|| format!("{}", path.display()))?)
}

fn fetch() -> anyhow::Result<()> {
    let corpus = load_corpus()?;
    let lock_path = bench_dir().join("corpus.lock");
    let mut lock: BTreeMap<String, String> = match std::fs::read_to_string(&lock_path) {
        Ok(s) => toml::from_str(&s)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
        Err(e) => return Err(e.into()),
    };
    std::fs::create_dir_all(bench_dir().join("cache"))?;
    for doc in &corpus.docs {
        let path = cache_path(&doc.id);
        if !path.exists() {
            println!("loading {} …", doc.id);
            let status = Command::new("curl")
                .args(["-sSfL", "--retry", "3", "-o"])
                .arg(&path)
                .arg(&doc.url)
                .status()
                .context("curl not executable")?;
            if !status.success() {
                bail!("download of {} failed ({})", doc.id, doc.url);
            }
        }
        let sha = sha256_hex(&std::fs::read(&path)?);
        match check_lock(&lock, &doc.id, &sha) {
            LockStatus::Match => println!("ok    {}", doc.id),
            LockStatus::New => {
                println!("new   {} {sha}", doc.id);
                lock.insert(doc.id.clone(), sha);
            }
            LockStatus::Mismatch { expected } => bail!(
                "checksum for {} does not match: expected {expected}, found {sha}. \
                 Delete the file and re-download it, or edit corpus.lock deliberately.",
                doc.id
            ),
        }
    }
    std::fs::write(&lock_path, toml::to_string(&lock)?)?;
    Ok(())
}
```

- [ ] **Step 6: Create and load the initial corpus**

`bench/corpus.toml`:

```toml
# Corpus for the extraction benchmark. Curation rules: bench/README.md.

[[doc]]
id = "dwork2006"
url = "https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/dwork.pdf"
category = "old LaTeX, Type1 math font without Unicode mapping"
[doc.expect]
sentences = ["This is captured by differential privacy."]
chars = ["ε"]

[[doc]]
id = "abadi2016"
url = "https://arxiv.org/pdf/1607.00133v2"
category = "modern two-column ACM paper with formulas"
```

Run: `cargo run -p extract-bench -- fetch`
Expected: two lines `new   <id> <sha>`; `bench/corpus.lock` contains both sums. A second run prints `ok` twice.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock tools/extract-bench bench/corpus.toml bench/corpus.lock
git commit -m "Add extraction benchmark tool with pinned corpus download

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 3: Backend Interface and `pdf_oxide`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/lib.rs`, `crates/bib-extract/src/join.rs`, `crates/bib-extract/src/backends/mod.rs`, `crates/bib-extract/src/backends/oxide.rs`
- Test: `crates/bib-extract/tests/common/mod.rs`, `crates/bib-extract/tests/oxide.rs`

**Interfaces:**
- Consumes: `bib_core::normalize::normalize` (tests only); fixture `bench/cache/dwork2006.pdf` from Task 2
- Produces:
  - `bib_extract::Rect { left: f32, top: f32, right: f32, bottom: f32 }`
  - `bib_extract::Span { text: String, bbox: Rect, font: String }`
  - `bib_extract::PageSize { width: f32, height: f32, rotation: Option<u16> }`
  - `bib_extract::PageContent { Spans(Vec<Span>), Plain(String) }`
  - `bib_extract::Page { index: usize, size: Option<PageSize>, content: PageContent }` with `fn text(&self) -> String`, `fn has_geometry(&self) -> bool`
  - `bib_extract::Extraction { backend: &'static str, backend_version: String, pages: Vec<Page> }` with `fn text(&self) -> String`
  - `bib_extract::ExtractError { Open(String), Page { index: usize, message: String }, Panicked(String), Unavailable(String) }`
  - `trait bib_extract::Backend { fn name(&self) -> &'static str; fn version(&self) -> String; fn extract(&self, path: &Path) -> Result<Extraction, ExtractError>; }`
  - `bib_extract::join_spans(spans: &[Span]) -> String`
  - `bib_extract::backends::oxide::PdfOxide` (unit struct, implements `Backend`)
  - Test helper `common::fixture(id: &str) -> PathBuf`

- [ ] **Step 1: Create the crate**

`Cargo.toml`: add `"crates/bib-extract"` to `members`; add dependencies:

```toml
thiserror = "2.0"
pdf_oxide = "=0.3.78"
```

`crates/bib-extract/Cargo.toml`:

```toml
[package]
name = "bib-extract"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
thiserror.workspace = true
pdf_oxide.workspace = true

[dev-dependencies]
bib-core = { path = "../bib-core" }
```

- [ ] **Step 2: Write tests for joining spans**

`crates/bib-extract/src/lib.rs`:

```rust
//! PDF text extraction behind a swappable interface (spec §7).

pub mod backends;
mod join;

use std::path::Path;

pub use join::join_spans;

/// Rectangle in points, **origin top-left, y grows downward**, relative to the MediaBox.
/// Every backend converts into this system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Contiguous piece of text with a box and font, in reading order.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub bbox: Rect,
    pub font: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
    /// `None` if the backend doesn't know the rotation.
    pub rotation: Option<u16>,
}

/// Text is mandatory, geometry optional (spec §7).
#[derive(Debug, Clone, PartialEq)]
pub enum PageContent {
    Spans(Vec<Span>),
    Plain(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub index: usize,
    pub size: Option<PageSize>,
    pub content: PageContent,
}

impl Page {
    pub fn text(&self) -> String {
        match &self.content {
            PageContent::Spans(spans) => join_spans(spans),
            PageContent::Plain(text) => text.clone(),
        }
    }

    pub fn has_geometry(&self) -> bool {
        matches!(self.content, PageContent::Spans(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extraction {
    pub backend: &'static str,
    pub backend_version: String,
    pub pages: Vec<Page>,
}

impl Extraction {
    /// Text of all pages, separated by newlines.
    pub fn text(&self) -> String {
        self.pages.iter().map(Page::text).collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("could not open PDF: {0}")]
    Open(String),
    #[error("page {index}: {message}")]
    Page { index: usize, message: String },
    #[error("backend panicked: {0}")]
    Panicked(String),
    #[error("tool not available: {0}")]
    Unavailable(String),
}

pub trait Backend {
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError>;
}

/// Catches panics from a backend (`pdf-extract` aborts hard on corrupt files).
pub(crate) fn guard<T>(f: impl FnOnce() -> Result<T, ExtractError>) -> Result<T, ExtractError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown cause".to_string());
            Err(ExtractError::Panicked(message))
        }
    }
}
```

`crates/bib-extract/src/join.rs`:

```rust
//! Join spans into text in reading order.

use crate::Span;

pub fn join_spans(_spans: &[Span]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    fn span(text: &str, left: f32, top: f32, right: f32, bottom: f32) -> Span {
        Span { text: text.into(), bbox: Rect { left, top, right, bottom }, font: "F".into() }
    }

    #[test]
    fn adjacent_spans_on_one_line_are_glued() {
        let spans = [span("differ", 0.0, 100.0, 30.0, 110.0), span("ential", 30.5, 100.0, 60.0, 110.0)];
        assert_eq!(join_spans(&spans), "differential");
    }

    #[test]
    fn spans_with_a_gap_on_one_line_get_a_space() {
        let spans = [span("privacy", 0.0, 100.0, 30.0, 110.0), span("loss", 34.0, 100.0, 60.0, 110.0)];
        assert_eq!(join_spans(&spans), "privacy loss");
    }

    #[test]
    fn next_line_gets_a_newline() {
        let spans = [span("end of line", 0.0, 100.0, 60.0, 110.0), span("next", 0.0, 112.0, 20.0, 122.0)];
        assert_eq!(join_spans(&spans), "end of line\nnext");
    }

    #[test]
    fn jump_back_on_same_height_is_a_newline() {
        let spans = [span("right column", 300.0, 100.0, 400.0, 110.0), span("left", 0.0, 100.0, 20.0, 110.0)];
        assert_eq!(join_spans(&spans), "right column\nleft");
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(join_spans(&[]), "");
    }
}
```

`crates/bib-extract/src/backends/mod.rs`:

```rust
pub mod oxide;
```

`crates/bib-extract/src/backends/oxide.rs` (empty for now, until Step 6):

```rust
//! Backend based on `pdf_oxide` (MIT/Apache-2.0, pure Rust).
```

- [ ] **Step 3: Run tests, confirm failure**

Run: `cargo test -p bib-extract --lib`
Expected: FAIL with `not yet implemented` in four tests.

- [ ] **Step 4: Implement `join_spans`**

```rust
pub fn join_spans(spans: &[Span]) -> String {
    let mut out = String::new();
    for (i, span) in spans.iter().enumerate() {
        if i > 0 {
            out.push_str(separator(&spans[i - 1], span));
        }
        out.push_str(&span.text);
    }
    out
}

fn separator(prev: &Span, next: &Span) -> &'static str {
    let prev_height = prev.bbox.bottom - prev.bbox.top;
    let next_height = next.bbox.bottom - next.bbox.top;
    let overlap = prev.bbox.bottom.min(next.bbox.bottom) - prev.bbox.top.max(next.bbox.top);
    let same_line = overlap > 0.5 * prev_height.min(next_height);
    let gap = next.bbox.left - prev.bbox.right;
    if !same_line || gap < -prev_height.max(next_height) {
        "\n"
    } else if gap < 0.15 * prev_height.max(next_height) {
        ""
    } else {
        " "
    }
}
```

Run: `cargo test -p bib-extract --lib`
Expected: PASS, 5 tests.

- [ ] **Step 5: Write an integration test for `pdf_oxide`**

`crates/bib-extract/tests/common/mod.rs`:

```rust
use std::path::PathBuf;

/// Path of a corpus document; aborts with instructions if it is missing.
pub fn fixture(id: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/cache")
        .join(format!("{id}.pdf"));
    assert!(
        path.exists(),
        "fixture {id} missing: run `cargo run -p extract-bench -- fetch` first"
    );
    path
}
```

`crates/bib-extract/tests/oxide.rs`:

```rust
mod common;

use bib_core::normalize::normalize;
use bib_extract::backends::oxide::PdfOxide;
use bib_extract::{Backend, PageContent};

#[test]
fn extracts_every_page_with_geometry() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    assert_eq!(extraction.backend, "pdf_oxide");
    assert!(!extraction.pages.is_empty());
    for page in &extraction.pages {
        assert!(page.has_geometry(), "page {} without geometry", page.index);
    }
}

#[test]
fn first_page_size_is_plausible() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    let size = extraction.pages[0].size.expect("page size");
    assert!((500.0..700.0).contains(&size.width), "width {}", size.width);
    assert!((700.0..900.0).contains(&size.height), "height {}", size.height);
}

#[test]
fn title_is_near_the_top_in_top_left_coordinates() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    let page = &extraction.pages[0];
    let PageContent::Spans(spans) = &page.content else { panic!("no spans") };
    let title_top = spans
        .iter()
        .filter(|s| s.text.contains("Differential"))
        .map(|s| s.bbox.top)
        .fold(f32::INFINITY, f32::min);
    let height = page.size.unwrap().height;
    assert!(title_top < 0.25 * height, "title at top={title_top}, page height {height}");
}

#[test]
fn text_contains_the_title_after_normalization() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    assert!(normalize(&extraction.text()).text.contains("differential privacy"));
}
```

Run: `cargo test -p bib-extract --test oxide`
Expected: FAIL to compile, `PdfOxide` does not exist.

- [ ] **Step 6: Implement the `pdf_oxide` backend**

`crates/bib-extract/src/backends/oxide.rs`:

```rust
//! Backend based on `pdf_oxide` (MIT/Apache-2.0, pure Rust).
//!
//! `pdf_oxide` returns boxes in PDF coordinates (origin bottom-left, `y` is the
//! bottom edge). Conversion: top = ury - (y + height), bottom = ury - y.

use std::path::Path;

use crate::{Backend, ExtractError, Extraction, Page, PageContent, PageSize, Rect, Span, guard, join_spans};

pub struct PdfOxide;

impl Backend for PdfOxide {
    fn name(&self) -> &'static str {
        "pdf_oxide"
    }

    fn version(&self) -> String {
        pdf_oxide::VERSION.to_string()
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        guard(|| {
            let doc = pdf_oxide::PdfDocument::open(path).map_err(|e| ExtractError::Open(e.to_string()))?;
            let count = doc.page_count().map_err(|e| ExtractError::Open(e.to_string()))?;
            let mut pages = Vec::with_capacity(count);
            for index in 0..count {
                pages.push(extract_page(&doc, index)?);
            }
            Ok(Extraction { backend: self.name(), backend_version: self.version(), pages })
        })
    }
}

fn extract_page(doc: &pdf_oxide::PdfDocument, index: usize) -> Result<Page, ExtractError> {
    let page_error = |e: pdf_oxide::Error| ExtractError::Page { index, message: e.to_string() };
    let raw_spans = doc.extract_spans(index).map_err(page_error)?;
    let raw_spans: Vec<_> = raw_spans.into_iter().filter(|s| !s.text.trim().is_empty()).collect();

    // Without its own MediaBox (e.g. inherited) there's no reliable conversion: text only, honestly without geometry.
    let Ok((llx, lly, urx, ury)) = doc.get_page_media_box(index) else {
        let spans: Vec<Span> = raw_spans
            .into_iter()
            .map(|s| Span {
                text: s.text,
                bbox: Rect { left: s.bbox.x, top: -s.bbox.y - s.bbox.height, right: s.bbox.x + s.bbox.width, bottom: -s.bbox.y },
                font: s.font_name,
            })
            .collect();
        return Ok(Page { index, size: None, content: PageContent::Plain(join_spans(&spans)) });
    };

    let rotation = doc.get_page_rotation(index).map_err(page_error)?.rem_euclid(360) as u16;
    let spans = raw_spans
        .into_iter()
        .map(|s| Span {
            bbox: Rect {
                left: s.bbox.x - llx,
                top: ury - (s.bbox.y + s.bbox.height),
                right: s.bbox.x + s.bbox.width - llx,
                bottom: ury - s.bbox.y,
            },
            text: s.text,
            font: s.font_name,
        })
        .collect();
    Ok(Page {
        index,
        size: Some(PageSize { width: urx - llx, height: ury - lly, rotation: Some(rotation) }),
        content: PageContent::Spans(spans),
    })
}
```

The makeshift boxes in the branch without a MediaBox only serve `join_spans` (same relative position, mirrored y-axis); they are not stored.

- [ ] **Step 7: Run tests**

Run: `cargo test -p bib-extract`
Expected: PASS (5 unit tests, 4 integration tests). If `title_is_near_the_top_in_top_left_coordinates` fails, the assumption about `pdf_oxide`'s `y` is wrong: print the value of `s.bbox.y` for the title (`dbg!`) and correct the conversion and the module comment accordingly, not the test.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/bib-extract
git commit -m "Add extraction backend interface and pdf_oxide backend

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 4: `pdf-extract` Backend (Text Only)

**Files:**
- Modify: `Cargo.toml`, `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/backends/mod.rs`
- Create: `crates/bib-extract/src/backends/pdf_extract.rs`
- Test: `crates/bib-extract/tests/pdf_extract.rs`

**Interfaces:**
- Consumes: `Backend`, `Extraction`, `Page`, `PageContent`, `ExtractError`, `guard` from Task 3
- Produces: `bib_extract::backends::pdf_extract::PdfExtract` (unit struct, `Backend`), name `"pdf-extract"`, pages always `PageContent::Plain`, `size: None`

- [ ] **Step 1: Add dependencies**

`Cargo.toml` → `[workspace.dependencies]`:

```toml
pdf-extract = "=0.12.0"
tempfile = "3.27"
```

`crates/bib-extract/Cargo.toml`:

```toml
[dependencies]
thiserror.workspace = true
pdf_oxide.workspace = true
pdf-extract.workspace = true

[dev-dependencies]
bib-core = { path = "../bib-core" }
tempfile.workspace = true
```

- [ ] **Step 2: Write the test**

`crates/bib-extract/tests/pdf_extract.rs`:

```rust
mod common;

use std::io::Write;

use bib_core::normalize::normalize;
use bib_extract::Backend;
use bib_extract::backends::pdf_extract::PdfExtract;

#[test]
fn pages_are_text_only() {
    let extraction = PdfExtract.extract(&common::fixture("dwork2006")).unwrap();
    assert_eq!(extraction.backend, "pdf-extract");
    assert!(!extraction.pages.is_empty());
    assert!(extraction.pages.iter().all(|p| !p.has_geometry() && p.size.is_none()));
}

#[test]
fn recovers_epsilon_from_type1_math_font() {
    // Measured during brainstorming: 28 ε in Dwork 2006 (pdf_oxide: 0).
    let extraction = PdfExtract.extract(&common::fixture("dwork2006")).unwrap();
    let epsilons = normalize(&extraction.text()).text.matches('ε').count();
    assert!(epsilons >= 25, "only {epsilons} ε found");
}

#[test]
fn broken_pdf_is_an_error_not_a_panic() {
    let mut file = tempfile::Builder::new().suffix(".pdf").tempfile().unwrap();
    file.write_all(b"%PDF-1.4\nnot a real PDF\n").unwrap();
    assert!(PdfExtract.extract(file.path()).is_err());
}
```

Run: `cargo test -p bib-extract --test pdf_extract`
Expected: FAIL to compile, module `pdf_extract` is missing.

- [ ] **Step 3: Implement the backend**

`crates/bib-extract/src/backends/mod.rs`:

```rust
pub mod oxide;
pub mod pdf_extract;
```

`crates/bib-extract/src/backends/pdf_extract.rs`:

```rust
//! Backend based on `pdf-extract` (MIT). Returns text only, no geometry,
//! but reads glyph names from embedded Type1 fonts (ε in old LaTeX).

use std::path::Path;

use crate::{Backend, ExtractError, Extraction, Page, PageContent, guard};

/// Must match the exact version in `Cargo.toml` (`=0.12.0`).
const VERSION: &str = "0.12.0";

pub struct PdfExtract;

impl Backend for PdfExtract {
    fn name(&self) -> &'static str {
        "pdf-extract"
    }

    fn version(&self) -> String {
        VERSION.to_string()
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        guard(|| {
            let texts = ::pdf_extract::extract_text_by_pages(path).map_err(|e| ExtractError::Open(e.to_string()))?;
            let pages = texts
                .into_iter()
                .enumerate()
                .map(|(index, text)| Page { index, size: None, content: PageContent::Plain(text) })
                .collect();
            Ok(Extraction { backend: self.name(), backend_version: self.version(), pages })
        })
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p bib-extract --test pdf_extract`
Expected: PASS, 3 tests. `pdf-extract` may print warnings, and for `broken_pdf…` a panic message on stderr; what matters is the test result.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/bib-extract
git commit -m "Add text-only pdf-extract backend with panic isolation

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 5: `mutool` Backend and Coordinate Reconciliation

**Files:**
- Modify: `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/backends/mod.rs`
- Create: `crates/bib-extract/src/backends/stext.rs`, `crates/bib-extract/src/backends/mutool.rs`
- Test: `crates/bib-extract/tests/coordinates.rs`

**Interfaces:**
- Consumes: types from Task 3, `PdfOxide`
- Produces:
  - `bib_extract::backends::stext::parse_stext(xml: &str) -> Vec<Page>` (one span per `<font>` element within a line; `rotation: None`)
  - `bib_extract::backends::mutool::Mutool { program: PathBuf }` with `Default` (`"mutool"`) and `fn is_available(&self) -> bool`; name `"mutool"`

- [ ] **Step 1: Write tests for the stext parser**

`crates/bib-extract/src/backends/mod.rs`:

```rust
pub mod mutool;
pub mod oxide;
pub mod pdf_extract;
pub mod stext;
```

`crates/bib-extract/src/backends/stext.rs`:

```rust
//! Parser for mutool's structured text format (`mutool draw -F stext`).
//! Coordinates there are already anchored top-left.

use crate::Page;

pub fn parse_stext(_xml: &str) -> Vec<Page> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PageContent, PageSize, Rect};

    const SAMPLE: &str = r##"<?xml version="1.0"?>
<document name="x.pdf">
<page id="page1" width="612" height="792">
<block bbox="238 115 375 127">
<line bbox="238 115 375 127" wmode="0" dir="1 0">
<font name="CMBX12" size="14.346">
<char quad="238 116 251 116 238 126 251 126" x="238" y="125" bidi="0" color="#000000" alpha="#ff" flags="0" c="D"/>
<char quad="251 116 256 116 251 126 256 126" x="251" y="125" bidi="0" color="#000000" alpha="#ff" flags="0" c="&amp;"/>
<char quad="256 116 259 116 256 126 259 126" x="256" y="125" bidi="0" color="#000000" alpha="#ff" flags="0" c=">"/>
</font>
<font name="CMMI10" size="10">
<char quad="260 117 266 117 260 127 266 127" x="260" y="126" bidi="0" color="#000000" alpha="#ff" flags="0" c="&#x3f5;"/>
</font>
</line>
</block>
</page>
<page id="page2" width="612" height="792">
</page>
</document>"##;

    #[test]
    fn parses_pages_spans_and_boxes() {
        let pages = parse_stext(SAMPLE);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].size, Some(PageSize { width: 612.0, height: 792.0, rotation: None }));
        let PageContent::Spans(spans) = &pages[0].content else { panic!("no spans") };
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "D&>");
        assert_eq!(spans[0].font, "CMBX12");
        assert_eq!(spans[0].bbox, Rect { left: 238.0, top: 116.0, right: 259.0, bottom: 126.0 });
        assert_eq!(spans[1].text, "ϵ");
        assert_eq!(pages[1].index, 1);
        assert_eq!(pages[1].content, PageContent::Spans(vec![]));
    }
}
```

Run: `cargo test -p bib-extract --lib stext`
Expected: FAIL with `not yet implemented`.

- [ ] **Step 2: Implement the parser**

Replace `parse_stext` and add helper functions:

```rust
use crate::{Page, PageContent, PageSize, Rect, Span};

pub fn parse_stext(xml: &str) -> Vec<Page> {
    let mut pages = Vec::new();
    let mut current: Option<(PageSize, Vec<Span>)> = None;
    let mut span: Option<Span> = None;
    for tag in tags(xml) {
        if tag.starts_with("page ") {
            let width = attr(tag, "width").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let height = attr(tag, "height").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            current = Some((PageSize { width, height, rotation: None }, Vec::new()));
        } else if tag == "/page" {
            if let Some((size, spans)) = current.take() {
                pages.push(Page { index: pages.len(), size: Some(size), content: PageContent::Spans(spans) });
            }
        } else if tag.starts_with("font ") {
            let font = attr(tag, "name").map(unescape).unwrap_or_default();
            span = Some(Span {
                text: String::new(),
                bbox: Rect { left: f32::INFINITY, top: f32::INFINITY, right: f32::NEG_INFINITY, bottom: f32::NEG_INFINITY },
                font,
            });
        } else if tag.starts_with("char ") {
            if let Some(s) = span.as_mut() {
                if let Some(c) = attr(tag, "c") {
                    s.text.push_str(&unescape(c));
                }
                if let Some(quad) = attr(tag, "quad") {
                    let nums: Vec<f32> = quad.split_whitespace().filter_map(|n| n.parse().ok()).collect();
                    if nums.len() == 8 {
                        for point in nums.chunks(2) {
                            s.bbox.left = s.bbox.left.min(point[0]);
                            s.bbox.right = s.bbox.right.max(point[0]);
                            s.bbox.top = s.bbox.top.min(point[1]);
                            s.bbox.bottom = s.bbox.bottom.max(point[1]);
                        }
                    }
                }
            }
        } else if tag == "/font" {
            if let (Some(s), Some((_, spans))) = (span.take(), current.as_mut()) {
                if !s.text.trim().is_empty() && s.bbox.left.is_finite() {
                    spans.push(s);
                }
            }
        }
    }
    pages
}

/// Contents of all tags without angle brackets. `<` is always escaped in XML attributes,
/// `>` is not necessarily (e.g. `c=">"`); so splitting happens at the last `>` before the next `<`.
/// Between tags, mutool's output contains only whitespace.
fn tags(xml: &str) -> impl Iterator<Item = &str> {
    xml.split('<').skip(1).filter_map(|chunk| chunk.rsplit_once('>').map(|(tag, _)| tag.trim_end_matches('/').trim_end()))
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let decoded = after.find(';').and_then(|j| {
            let entity = &after[..j];
            let ch = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => entity
                    .strip_prefix("#x")
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .or_else(|| entity.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                    .and_then(char::from_u32),
            };
            ch.map(|c| (c, j))
        });
        match decoded {
            Some((c, j)) => {
                out.push(c);
                rest = &after[j + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}
```

Replace the `use crate::Page;` import above with the line above.

Run: `cargo test -p bib-extract --lib stext`
Expected: PASS.

- [ ] **Step 3: Write the coordinate test**

`crates/bib-extract/Cargo.toml` → add `tempfile.workspace = true` to `[dependencies]` (needed by the backend; it can stay in `[dev-dependencies]` too).

`crates/bib-extract/tests/coordinates.rs`:

```rust
//! The same text passage must be at the same position in every backend with geometry (spec §7).

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::oxide::PdfOxide;
use bib_extract::{Backend, PageContent, Rect};

const TOLERANCE: f32 = 6.0;

pub fn first_box_containing(extraction: &bib_extract::Extraction, needle: &str) -> Rect {
    let PageContent::Spans(spans) = &extraction.pages[0].content else { panic!("{}: no spans", extraction.backend) };
    spans
        .iter()
        .filter(|s| s.text.contains(needle))
        .min_by(|a, b| a.bbox.top.total_cmp(&b.bbox.top))
        .unwrap_or_else(|| panic!("{}: \"{needle}\" not found", extraction.backend))
        .bbox
}

#[test]
fn pdf_oxide_and_mutool_agree_on_title_position() {
    let mutool = Mutool::default();
    if !mutool.is_available() {
        eprintln!("SKIPPED: mutool not installed");
        return;
    }
    let path = common::fixture("dwork2006");
    let reference = first_box_containing(&mutool.extract(&path).unwrap(), "Differential");
    let oxide = first_box_containing(&PdfOxide.extract(&path).unwrap(), "Differential");
    assert!((reference.top - oxide.top).abs() < TOLERANCE, "top: mutool {} vs pdf_oxide {}", reference.top, oxide.top);
    assert!((reference.left - oxide.left).abs() < TOLERANCE, "left: mutool {} vs pdf_oxide {}", reference.left, oxide.left);
}
```

Run: `cargo test -p bib-extract --test coordinates`
Expected: FAIL to compile, `Mutool` is missing.

- [ ] **Step 4: Implement the `mutool` backend**

`crates/bib-extract/src/backends/mutool.rs`:

```rust
//! `mutool` as an external process (optional). mutool is AGPL-licensed and is
//! therefore never linked, only invoked.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::backends::stext::parse_stext;
use crate::{Backend, ExtractError, Extraction};

pub struct Mutool {
    pub program: PathBuf,
}

impl Default for Mutool {
    fn default() -> Self {
        Self { program: PathBuf::from("mutool") }
    }
}

impl Mutool {
    pub fn is_available(&self) -> bool {
        Command::new(&self.program).arg("-v").output().is_ok()
    }
}

impl Backend for Mutool {
    fn name(&self) -> &'static str {
        "mutool"
    }

    fn version(&self) -> String {
        Command::new(&self.program)
            .arg("-v")
            .output()
            .ok()
            .and_then(|out| {
                let text = String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout);
                text.split_whitespace().skip_while(|w| *w != "version").nth(1).map(str::to_string)
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        let out_file = tempfile::Builder::new()
            .suffix(".xml")
            .tempfile()
            .map_err(|e| ExtractError::Unavailable(format!("temp file: {e}")))?;
        let output = Command::new(&self.program)
            .args(["draw", "-q", "-F", "stext", "-o"])
            .arg(out_file.path())
            .arg(path)
            .output()
            .map_err(|e| ExtractError::Unavailable(format!("mutool: {e}")))?;
        if !output.status.success() {
            return Err(ExtractError::Open(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        let xml = std::fs::read_to_string(out_file.path()).map_err(|e| ExtractError::Open(e.to_string()))?;
        Ok(Extraction { backend: self.name(), backend_version: self.version(), pages: parse_stext(&xml) })
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p bib-extract`
Expected: PASS. `coordinates` must not end with "SKIPPED"; `mutool` is installed on the development machine (`mutool version 1.25.1`). If the coordinate test fails, the conversion in `oxide.rs` is wrong: fix it, don't raise the tolerance.

- [ ] **Step 6: Commit**

```bash
git add crates/bib-extract Cargo.lock
git commit -m "Add mutool subprocess backend and cross-backend coordinate check

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 6: pdfium Backend Behind a Feature

**Files:**
- Modify: `Cargo.toml`, `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/backends/mod.rs`
- Create: `crates/bib-extract/src/backends/pdfium.rs`
- Test: `crates/bib-extract/tests/pdfium.rs`

**Interfaces:**
- Consumes: types from Task 3, `Mutool` and the `first_box_containing` logic from Task 5
- Produces: `bib_extract::backends::pdfium::Pdfium { lib_dir: PathBuf }` with `fn from_env() -> Option<Self>` (reads `BIB_PDFIUM_LIB_DIR`); name `"pdfium"`; only with feature `pdfium`

- [ ] **Step 1: Download the pdfium library and create the feature**

```bash
mkdir -p bench/cache/pdfium
curl -sSfL https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz | tar xz -C bench/cache/pdfium
ls bench/cache/pdfium/lib/libpdfium.so
```

`Cargo.toml` → `[workspace.dependencies]`: `pdfium-render = "=0.9.4"`

`crates/bib-extract/Cargo.toml`:

```toml
[features]
pdfium = ["dep:pdfium-render"]

[dependencies]
thiserror.workspace = true
pdf_oxide.workspace = true
pdf-extract.workspace = true
tempfile.workspace = true
pdfium-render = { workspace = true, optional = true }
```

`crates/bib-extract/src/backends/mod.rs`:

```rust
pub mod mutool;
pub mod oxide;
pub mod pdf_extract;
#[cfg(feature = "pdfium")]
pub mod pdfium;
pub mod stext;
```

- [ ] **Step 2: Write the test**

`crates/bib-extract/tests/pdfium.rs`:

```rust
#![cfg(feature = "pdfium")]

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::pdfium::Pdfium;
use bib_extract::{Backend, PageContent};

fn backend() -> Pdfium {
    Pdfium::from_env().expect("set BIB_PDFIUM_LIB_DIR, e.g. to bench/cache/pdfium/lib")
}

#[test]
fn extracts_pages_with_geometry() {
    let extraction = backend().extract(&common::fixture("abadi2016")).unwrap();
    assert!(!extraction.pages.is_empty());
    assert!(extraction.pages.iter().all(|p| p.has_geometry() && p.size.is_some()));
}

#[test]
fn pdfium_and_mutool_agree_on_title_position() {
    let mutool = Mutool::default();
    if !mutool.is_available() {
        eprintln!("SKIPPED: mutool not installed");
        return;
    }
    let path = common::fixture("dwork2006");
    let top_of = |e: &bib_extract::Extraction| {
        let PageContent::Spans(spans) = &e.pages[0].content else { panic!("no spans") };
        spans.iter().filter(|s| s.text.contains("Differential")).map(|s| s.bbox.top).fold(f32::INFINITY, f32::min)
    };
    let reference = top_of(&mutool.extract(&path).unwrap());
    let pdfium = top_of(&backend().extract(&path).unwrap());
    assert!((reference - pdfium).abs() < 6.0, "top: mutool {reference} vs pdfium {pdfium}");
}
```

Run: `BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib cargo test -p bib-extract --features pdfium --test pdfium`
Expected: FAIL to compile, module `pdfium` is missing.

- [ ] **Step 3: Implement the backend**

`crates/bib-extract/src/backends/pdfium.rs`:

```rust
//! Backend based on pdfium (Chrome's PDF engine) via `pdfium-render`.
//! Needs `libpdfium.so` at runtime; path via `BIB_PDFIUM_LIB_DIR`.

use std::path::{Path, PathBuf};

use pdfium_render::prelude::{PdfPageRenderRotation, Pdfium as PdfiumLib};

use crate::{Backend, ExtractError, Extraction, Page, PageContent, PageSize, Rect, Span, guard};

pub struct Pdfium {
    pub lib_dir: PathBuf,
}

impl Pdfium {
    pub fn from_env() -> Option<Self> {
        std::env::var_os("BIB_PDFIUM_LIB_DIR").map(|dir| Self { lib_dir: PathBuf::from(dir) })
    }
}

impl Backend for Pdfium {
    fn name(&self) -> &'static str {
        "pdfium"
    }

    fn version(&self) -> String {
        format!("pdfium-render 0.9.4, libpdfium from {}", self.lib_dir.display())
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        guard(|| {
            let lib = PdfiumLib::pdfium_platform_library_name_at_path(&self.lib_dir.to_string_lossy().into_owned());
            let bindings = PdfiumLib::bind_to_library(lib).map_err(|e| ExtractError::Unavailable(e.to_string()))?;
            let pdfium = PdfiumLib::new(bindings);
            let doc = pdfium.load_pdf_from_file(path, None).map_err(|e| ExtractError::Open(e.to_string()))?;
            let mut pages = Vec::new();
            for (index, page) in doc.pages().iter().enumerate() {
                let page_error = |e: pdfium_render::prelude::PdfiumError| ExtractError::Page { index, message: e.to_string() };
                let height = page.height().value;
                let rotation = match page.rotation().map_err(page_error)? {
                    PdfPageRenderRotation::None => 0,
                    PdfPageRenderRotation::Degrees90 => 90,
                    PdfPageRenderRotation::Degrees180 => 180,
                    PdfPageRenderRotation::Degrees270 => 270,
                };
                let text = page.text().map_err(page_error)?;
                let spans = text
                    .segments()
                    .iter()
                    .filter(|segment| !segment.text().trim().is_empty())
                    .map(|segment| {
                        let b = segment.bounds();
                        Span {
                            text: segment.text(),
                            bbox: Rect {
                                left: b.left().value,
                                top: height - b.top().value,
                                right: b.right().value,
                                bottom: height - b.bottom().value,
                            },
                            font: String::new(),
                        }
                    })
                    .collect();
                pages.push(Page {
                    index,
                    size: Some(PageSize { width: page.width().value, height, rotation: Some(rotation) }),
                    content: PageContent::Spans(spans),
                });
            }
            Ok(Extraction { backend: self.name(), backend_version: self.version(), pages })
        })
    }
}
```

- [ ] **Step 4: Run tests**

Run: `BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib cargo test -p bib-extract --features pdfium`
Expected: PASS, all tests including `pdfium`. Also `cargo test -p bib-extract` without the feature: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/bib-extract
git commit -m "Add feature-gated pdfium backend for benchmark comparison

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 7: Curate the Corpus

Pure data work, no code. The result is verified assertions that Task 8 measures against.

**Files:**
- Modify: `bench/corpus.toml`, `bench/corpus.lock`
- Create: `bench/README.md`

**Interfaces:**
- Consumes: the `Expect` format from Task 2
- Produces: seven documents with 3 sentences each, 1 order pair, and characters where applicable

- [ ] **Step 1: Record the curation rules**

`bench/README.md`:

````markdown
# Extraction Benchmark

Measures PDF backends against hand-curated assertions (olmOCR-Bench style): do known sentences
appear contiguously and in the right order in the normalized text? Are expected special
characters present? How much garbage data is produced?

```sh
cargo run -p extract-bench -- fetch
BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib \
  cargo run --release -p extract-bench --features pdfium -- run --out bench/results/<date>.md
```

## Curation

Per document:

- **Three sentences:** (a) the first sentence of the abstract, (b) a sentence from the right
  column or the second half of page 2, (c) a sentence from the last paragraph before the references.
- **Rules for sentences:** at least 8 words, no formulas, no citation markers, no
  footnote markers, ends with a period. Copied verbatim from the LaTeX source, not from a
  PDF extraction (otherwise the benchmark would measure the tool used to curate it).
- **One order pair:** `before = [["<start of b>", "<start of c>"]]`, each a
  unique passage of at least 5 words.
- **Characters:** Greek letters that get rendered in the body text (`\epsilon`,
  `\varepsilon` → `ε`; `\delta` → `δ`).
- **Check against the source:** every sentence must appear in the LaTeX source:
  `tr -s '[:space:]' ' ' < <file>.tex | grep -F -c '<sentence>'` ≥ 1.
- **Check in the PDF:** search for and visually confirm every sentence in the PDF viewer (macros
  can change the rendered text).
- Documents without a LaTeX source (Dwork 2006): sentences taken only from the PDF viewer,
  checked against the page by eye.

If, after the first run, **not a single** backend finds a sentence, the assertion is
probably wrong: check it against the PDF and fix it before results are scored.
````

- [ ] **Step 2: Add the documents**

Append to `bench/corpus.toml`:

```toml
[[doc]]
id = "vaswani2017"
url = "https://arxiv.org/pdf/1706.03762v7"
category = "single-column, NeurIPS, tables and formulas"

[[doc]]
id = "devlin2019"
url = "https://arxiv.org/pdf/1810.04805v2"
category = "two-column, ACL, tables"

[[doc]]
id = "he2016"
url = "https://arxiv.org/pdf/1512.03385v1"
category = "two-column, CVPR, figures and tables"

[[doc]]
id = "shokri2017"
url = "https://arxiv.org/pdf/1610.05820v2"
category = "two-column, IEEE S&P, privacy topic"

[[doc]]
id = "carlini2021"
url = "https://arxiv.org/pdf/2012.07805v2"
category = "two-column, USENIX Security, long appendices"
```

Run: `cargo run -p extract-bench -- fetch`
Expected: five new entries `new …`, two `ok`.

- [ ] **Step 3: Download the LaTeX sources**

```bash
for id in 1607.00133v2 1706.03762v7 1810.04805v2 1512.03385v1 1610.05820v2 2012.07805v2; do
  mkdir -p bench/cache/src/$id
  curl -sSfL "https://arxiv.org/e-print/$id" -o bench/cache/src/$id.src
  tar xzf bench/cache/src/$id.src -C bench/cache/src/$id 2>/dev/null \
    || gunzip -c bench/cache/src/$id.src > bench/cache/src/$id/main.tex
  ls bench/cache/src/$id/*.tex
done
```

Expected: at least one `.tex` file per document.

- [ ] **Step 4: Curate the assertions**

For `abadi2016`, `vaswani2017`, `devlin2019`, `he2016`, `shokri2017`, `carlini2021`, add one `[doc.expect]` block each, following the rules in `bench/README.md`, under the respective `[[doc]]`. For `dwork2006`, add two more sentences (b) and (c) and an order pair from the PDF. Form:

```toml
[doc.expect]
sentences = [
  "<sentence a>",
  "<sentence b>",
  "<sentence c>",
]
before = [["<passage from b>", "<passage from c>"]]
chars = ["ε", "δ"]
```

The placeholders in angle brackets here describe the format; the curated sentences go in their place.

- [ ] **Step 5: Check every sentence against the source**

```bash
check() { tr -s '[:space:]' ' ' < "$1" | grep -F -c -- "$2"; }
# Example for a sentence from abadi2016:
check <(cat bench/cache/src/1607.00133v2/*.tex) "<sentence>"
```

Expected: a number ≥ 1 for every sentence of the six arXiv documents. Replace any sentence with 0.

- [ ] **Step 6: Have the corpus parsed**

Run: `cargo run -p extract-bench -- fetch`
Expected: `ok` seven times, no parse errors.

- [ ] **Step 7: Commit**

```bash
git add bench/README.md bench/corpus.toml bench/corpus.lock
git commit -m "Curate benchmark corpus with verified sentence assertions

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 8: Measurement and Report

**Files:**
- Modify: `tools/extract-bench/Cargo.toml`, `tools/extract-bench/src/main.rs`
- Create: `tools/extract-bench/src/metrics.rs`, `tools/extract-bench/src/report.rs`

**Interfaces:**
- Consumes: `bib_core::normalize::normalize`; `bib_extract::{Backend, Extraction}`; all backends; `corpus::{Expect, cache_path}`; `load_corpus()` from `main.rs` (Task 2)
- Produces:
  - `metrics::TextScores { sentences_found, sentences_total, order_ok, order_total: usize, chars_missing: Vec<String>, control_chars, cid_markers, replacement_chars: usize }`
  - `metrics::evaluate(expect: &Expect, raw: &str) -> TextScores`
  - `report::Row { doc: String, backend: String, scores: Option<TextScores>, pages: usize, pages_with_geometry: usize, millis: u128, error: Option<String> }`
  - `report::render_markdown(rows: &[Row]) -> String`
  - CLI: `extract-bench run --out <file> [--backends <list>]`

- [ ] **Step 1: Write tests for scoring**

`tools/extract-bench/Cargo.toml` → `[dependencies]`:

```toml
bib-extract = { path = "../../crates/bib-extract" }
```

and

```toml
[features]
pdfium = ["bib-extract/pdfium"]
```

`tools/extract-bench/src/metrics.rs`:

```rust
//! Scoring an extracted text against the curated assertions.

use crate::corpus::Expect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextScores {
    pub sentences_found: usize,
    pub sentences_total: usize,
    pub order_ok: usize,
    pub order_total: usize,
    pub chars_missing: Vec<String>,
    /// Control characters except \n, \r, \t and form feed.
    pub control_chars: usize,
    /// Occurrences of `(cid:`.
    pub cid_markers: usize,
    /// Occurrences of U+FFFD.
    pub replacement_chars: usize,
}

pub fn evaluate(_expect: &Expect, _raw: &str) -> TextScores {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect() -> Expect {
        Expect {
            sentences: vec!["Deeper networks are harder to train.".into(), "Not in the text at all.".into()],
            before: vec![("deeper networks".into(), "residual learning".into())],
            chars: vec!["ε".into(), "δ".into()],
        }
    }

    #[test]
    fn counts_sentences_after_normalization() {
        let raw = "Deeper net-\nworks are harder to train. We use residual learning with ϵ.";
        let scores = evaluate(&expect(), raw);
        assert_eq!((scores.sentences_found, scores.sentences_total), (1, 2));
    }

    #[test]
    fn checks_reading_order() {
        let in_order = evaluate(&expect(), "Deeper networks first, then residual learning.");
        assert_eq!((in_order.order_ok, in_order.order_total), (1, 1));
        let reversed = evaluate(&expect(), "Residual learning first, then deeper networks.");
        assert_eq!(reversed.order_ok, 0);
        let missing = evaluate(&expect(), "Only deeper networks here.");
        assert_eq!(missing.order_ok, 0);
    }

    #[test]
    fn reports_missing_characters() {
        let scores = evaluate(&expect(), "privacy with ϵ only");
        assert_eq!(scores.chars_missing, vec!["δ".to_string()]);
    }

    #[test]
    fn counts_garbage() {
        let scores = evaluate(&expect(), "a\u{f}b (cid:15) c\u{FFFD}\n\t\u{c}");
        assert_eq!(scores.control_chars, 1);
        assert_eq!(scores.cid_markers, 1);
        assert_eq!(scores.replacement_chars, 1);
    }
}
```

`tools/extract-bench/src/main.rs`: add the line `mod metrics;` under `mod corpus;`.

Run: `cargo test -p extract-bench metrics`
Expected: FAIL with `not yet implemented`.

- [ ] **Step 2: Implement scoring**

```rust
use bib_core::normalize::normalize;

pub fn evaluate(expect: &Expect, raw: &str) -> TextScores {
    let text = normalize(raw).text;
    let position = |needle: &str| text.find(&normalize(needle).text);
    let sentences_found = expect.sentences.iter().filter(|s| position(s).is_some()).count();
    let order_ok = expect
        .before
        .iter()
        .filter(|(first, second)| matches!((position(first), position(second)), (Some(a), Some(b)) if a < b))
        .count();
    let chars_missing = expect
        .chars
        .iter()
        .filter(|c| !text.contains(&normalize(c).text))
        .cloned()
        .collect();
    TextScores {
        sentences_found,
        sentences_total: expect.sentences.len(),
        order_ok,
        order_total: expect.before.len(),
        chars_missing,
        control_chars: raw.chars().filter(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t' | '\u{c}')).count(),
        cid_markers: raw.matches("(cid:").count(),
        replacement_chars: raw.matches('\u{FFFD}').count(),
    }
}
```

Run: `cargo test -p extract-bench metrics`
Expected: PASS, 4 tests.

- [ ] **Step 3: Write tests for the report**

`tools/extract-bench/src/report.rs`:

```rust
//! Markdown report: one row per document and backend, with a per-backend total below.

use crate::metrics::TextScores;

#[derive(Debug, Clone)]
pub struct Row {
    pub doc: String,
    pub backend: String,
    pub scores: Option<TextScores>,
    pub pages: usize,
    pub pages_with_geometry: usize,
    pub millis: u128,
    pub error: Option<String>,
}

pub fn render_markdown(_rows: &[Row]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores(found: usize, total: usize, order: usize) -> TextScores {
        TextScores {
            sentences_found: found,
            sentences_total: total,
            order_ok: order,
            order_total: 1,
            chars_missing: vec!["ε".into()],
            control_chars: 2,
            cid_markers: 0,
            replacement_chars: 0,
        }
    }

    fn row(doc: &str, backend: &str, scores: Option<TextScores>, error: Option<&str>) -> Row {
        Row { doc: doc.into(), backend: backend.into(), scores, pages: 6, pages_with_geometry: 6, millis: 120, error: error.map(Into::into) }
    }

    #[test]
    fn renders_detail_rows_and_backend_totals() {
        let md = render_markdown(&[
            row("a", "pdf_oxide", Some(scores(2, 3, 1)), None),
            row("b", "pdf_oxide", Some(scores(3, 3, 0)), None),
            row("a", "mutool", None, Some("not installed")),
        ]);
        assert!(md.contains("| a | pdf_oxide | 2/3 | 1/1 | ε | 2 | 0 | 0 | 6/6 | 120 |"), "{md}");
        assert!(md.contains("| a | mutool | error: not installed |"), "{md}");
        assert!(md.contains("| pdf_oxide | 5/6 | 1/2 | 2 | 4 | 0 | 0 | 0 |"), "{md}");
    }
}
```

`tools/extract-bench/src/main.rs`: add `mod report;`.

Run: `cargo test -p extract-bench report`
Expected: FAIL with `not yet implemented`.

- [ ] **Step 4: Implement the report**

```rust
pub fn render_markdown(rows: &[Row]) -> String {
    let mut md = String::new();
    md.push_str("## Details\n\n");
    md.push_str("| Document | Backend | Sentences | Order | Missing chars | Control chars | (cid:) | U+FFFD | Pages with geometry | ms |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for row in rows {
        match (&row.scores, &row.error) {
            (Some(s), _) => md.push_str(&format!(
                "| {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {}/{} | {} |\n",
                row.doc,
                row.backend,
                s.sentences_found,
                s.sentences_total,
                s.order_ok,
                s.order_total,
                if s.chars_missing.is_empty() { "–".to_string() } else { s.chars_missing.join(" ") },
                s.control_chars,
                s.cid_markers,
                s.replacement_chars,
                row.pages_with_geometry,
                row.pages,
                row.millis,
            )),
            (None, error) => md.push_str(&format!(
                "| {} | {} | error: {} |\n",
                row.doc,
                row.backend,
                error.as_deref().unwrap_or("unknown")
            )),
        }
    }

    md.push_str("\n## Totals per backend\n\n");
    md.push_str("| Backend | Sentences | Order | Docs with missing chars | Control chars | (cid:) | U+FFFD | Errors |\n");
    md.push_str("|---|---|---|---|---|---|---|---|\n");
    let mut backends: Vec<&str> = Vec::new();
    for row in rows {
        if !backends.contains(&row.backend.as_str()) {
            backends.push(&row.backend);
        }
    }
    for backend in backends {
        let of_backend: Vec<&Row> = rows.iter().filter(|r| r.backend == backend).collect();
        let scored: Vec<&TextScores> = of_backend.iter().filter_map(|r| r.scores.as_ref()).collect();
        let sum = |f: fn(&TextScores) -> usize| scored.iter().map(|s| f(s)).sum::<usize>();
        md.push_str(&format!(
            "| {} | {}/{} | {}/{} | {} | {} | {} | {} | {} |\n",
            backend,
            sum(|s| s.sentences_found),
            sum(|s| s.sentences_total),
            sum(|s| s.order_ok),
            sum(|s| s.order_total),
            scored.iter().filter(|s| !s.chars_missing.is_empty()).count(),
            sum(|s| s.control_chars),
            sum(|s| s.cid_markers),
            sum(|s| s.replacement_chars),
            of_backend.iter().filter(|r| r.error.is_some()).count(),
        ));
    }
    md
}
```

Run: `cargo test -p extract-bench report`
Expected: PASS.

- [ ] **Step 5: Implement the `run` command**

Add to `tools/extract-bench/src/main.rs`. Imports:

```rust
use std::path::PathBuf;
use std::time::Instant;

use bib_extract::Backend;
use bib_extract::backends::{mutool::Mutool, oxide::PdfOxide, pdf_extract::PdfExtract};
```

Extend `Cmd`:

```rust
    /// Extracts all corpus documents with all available backends and writes a report.
    Run {
        /// Target file for the markdown report, e.g. bench/results/2026-09-18.md
        #[arg(long)]
        out: PathBuf,
        /// Comma-separated selection: pdf_oxide,pdf-extract,mutool,pdfium
        #[arg(long, value_delimiter = ',')]
        backends: Option<Vec<String>>,
    },
```

Extend `main`:

```rust
        Cmd::Run { out, backends } => run(out, backends),
```

Functions:

```rust
fn available_backends(selection: Option<Vec<String>>) -> Vec<Box<dyn Backend>> {
    let mut all: Vec<Box<dyn Backend>> = vec![Box::new(PdfOxide), Box::new(PdfExtract)];
    let mutool = Mutool::default();
    if mutool.is_available() {
        all.push(Box::new(mutool));
    } else {
        eprintln!("mutool not installed, skipped");
    }
    #[cfg(feature = "pdfium")]
    match bib_extract::backends::pdfium::Pdfium::from_env() {
        Some(pdfium) => all.push(Box::new(pdfium)),
        None => eprintln!("BIB_PDFIUM_LIB_DIR not set, pdfium skipped"),
    }
    match selection {
        Some(names) => all.into_iter().filter(|b| names.iter().any(|n| n == b.name())).collect(),
        None => all,
    }
}

fn run(out: PathBuf, selection: Option<Vec<String>>) -> anyhow::Result<()> {
    let corpus = load_corpus()?;
    let backends = available_backends(selection);
    let mut rows = Vec::new();
    for doc in &corpus.docs {
        let path = cache_path(&doc.id);
        anyhow::ensure!(path.exists(), "{} missing, run `fetch` first", doc.id);
        for backend in &backends {
            let started = Instant::now();
            let result = backend.extract(&path);
            let millis = started.elapsed().as_millis();
            let row = match result {
                Ok(extraction) => report::Row {
                    doc: doc.id.clone(),
                    backend: backend.name().to_string(),
                    scores: Some(metrics::evaluate(&doc.expect, &extraction.text())),
                    pages: extraction.pages.len(),
                    pages_with_geometry: extraction.pages.iter().filter(|p| p.has_geometry()).count(),
                    millis,
                    error: None,
                },
                Err(e) => report::Row {
                    doc: doc.id.clone(),
                    backend: backend.name().to_string(),
                    scores: None,
                    pages: 0,
                    pages_with_geometry: 0,
                    millis,
                    error: Some(e.to_string()),
                },
            };
            println!("{:12} {:12} {} ms", row.doc, row.backend, row.millis);
            rows.push(row);
        }
    }
    let mut header = String::from("# Extraction Benchmark\n\n## Backends\n\n");
    for backend in &backends {
        header.push_str(&format!("- {} {}\n", backend.name(), backend.version()));
    }
    header.push('\n');
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, header + &report::render_markdown(&rows))?;
    println!("Report: {}", out.display());
    Ok(())
}
```

- [ ] **Step 6: Short run**

Run: `cargo run --release -p extract-bench -- run --out /dev/stdout --backends pdf_oxide,pdf-extract`
Expected: report with 14 detail rows and 2 total rows on stdout, no panic.

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add tools/extract-bench Cargo.lock
git commit -m "Add benchmark metrics, markdown report and run command

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 9: Zed Test with Two Language Servers (Step 0b)

Throwaway code. It answers one question and is not developed further afterward.

**Files:**
- Modify: `Cargo.toml`
- Create: `spikes/lsp-coexist/Cargo.toml`, `spikes/lsp-coexist/src/lib.rs`, `spikes/lsp-coexist/src/main.rs`
- Create: `spikes/zed-coexist-ext/extension.toml`, `spikes/zed-coexist-ext/Cargo.toml`, `spikes/zed-coexist-ext/src/lib.rs`
- Create: `spikes/typst-sample/main.typ`, `spikes/typst-sample/refs.bib`
- Create: `docs/research/14-zed-two-language-servers.md`

**Interfaces:**
- Produces: program `lsp-coexist` (stdio language server), dev extension `bib-spike`, log with the result per function

- [ ] **Step 1: Create the crate and write tests for key lookup**

`Cargo.toml`: add `"spikes/lsp-coexist"` to `members`; dependencies:

```toml
tower-lsp-server = "0.23"
tokio = { version = "1", features = ["io-std", "macros", "rt-multi-thread", "sync"] }
```

`spikes/lsp-coexist/Cargo.toml`:

```toml
[package]
name = "lsp-coexist"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
tower-lsp-server.workspace = true
tokio.workspace = true
```

`spikes/lsp-coexist/src/lib.rs`:

```rust
//! Throwaway code for step 0b: finds `@key` references in Typst text.
//! Columns in UTF-16 code units, as required by LSP.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHit {
    pub key: String,
    pub line: u32,
    pub start: u32,
    pub end: u32,
}

pub fn find_keys(_text: &str) -> Vec<KeyHit> {
    todo!()
}

pub fn key_at(text: &str, line: u32, character: u32) -> Option<KeyHit> {
    find_keys(text).into_iter().find(|hit| hit.line == line && hit.start <= character && character <= hit.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_keys_and_strips_trailing_punctuation() {
        let hits = find_keys("From @dwork2006.\nAnd @a:b-c_d, done.");
        assert_eq!(
            hits,
            vec![
                KeyHit { key: "dwork2006".into(), line: 0, start: 5, end: 15 },
                KeyHit { key: "a:b-c_d".into(), line: 1, start: 4, end: 12 },
            ]
        );
    }

    #[test]
    fn ignores_emails_and_escaped_at() {
        assert!(find_keys("email to a@b.de and \\@not").is_empty());
    }

    #[test]
    fn columns_are_utf16() {
        let hits = find_keys("𝜖 @x1");
        assert_eq!(hits[0].start, 3);
        assert_eq!(hits[0].end, 6);
    }

    #[test]
    fn key_at_hits_inside_and_at_edges() {
        let text = "about @dwork2006 here";
        assert_eq!(key_at(text, 0, 6).unwrap().key, "dwork2006");
        assert_eq!(key_at(text, 0, 16).unwrap().key, "dwork2006");
        assert!(key_at(text, 0, 2).is_none());
    }
}
```

`spikes/lsp-coexist/src/main.rs` (temporary):

```rust
fn main() {}
```

Run: `cargo test -p lsp-coexist`
Expected: FAIL with `not yet implemented`.

- [ ] **Step 2: Implement key lookup**

```rust
pub fn find_keys(text: &str) -> Vec<KeyHit> {
    let mut hits = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut utf16_col = 0u32;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let starts_key = c == '@'
                && !matches!(prev, Some(p) if p.is_alphanumeric() || p == '\\')
                && chars.get(i + 1).is_some_and(|n| n.is_alphabetic());
            if !starts_key {
                utf16_col += c.len_utf16() as u32;
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_alphanumeric() || matches!(chars[j], '_' | '-' | ':' | '.')) {
                j += 1;
            }
            while j > i + 1 && matches!(chars[j - 1], '.' | ':' | '-') {
                j -= 1;
            }
            let key: String = chars[i + 1..j].iter().collect();
            let width: u32 = chars[i..j].iter().map(|c| c.len_utf16() as u32).sum();
            hits.push(KeyHit { key, line: line_no as u32, start: utf16_col, end: utf16_col + width });
            utf16_col += width;
            i = j;
        }
    }
    hits
}
```

Run: `cargo test -p lsp-coexist`
Expected: PASS, 4 tests.

- [ ] **Step 3: Write the language server**

`spikes/lsp-coexist/src/main.rs`:

```rust
//! Throwaway language server for step 0b. Every response is tagged with "bib-spike",
//! so it's visible in Zed which server it came from.

use std::collections::HashMap;

use lsp_coexist::{KeyHit, find_keys, key_at};
use tokio::sync::Mutex;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct Spike {
    client: Client,
    docs: Mutex<HashMap<String, String>>,
}

fn range(hit: &KeyHit) -> Range {
    Range::new(Position::new(hit.line, hit.start), Position::new(hit.line, hit.end))
}

impl Spike {
    async fn update(&self, uri: Uri, text: String) {
        let diagnostics = find_keys(&text)
            .iter()
            .map(|hit| Diagnostic {
                range: range(hit),
                severity: Some(DiagnosticSeverity::HINT),
                source: Some("bib-spike".into()),
                message: format!("bib-spike sees @{}", hit.key),
                ..Default::default()
            })
            .collect();
        self.docs.lock().await.insert(uri.to_string(), text);
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn hit(&self, uri: &Uri, position: Position) -> Option<(String, KeyHit)> {
        let docs = self.docs.lock().await;
        let text = docs.get(&uri.to_string())?;
        key_at(text, position.line, position.character).map(|hit| (text.clone(), hit))
    }
}

impl LanguageServer for Spike {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo { name: "bib-spike".into(), version: None }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["@".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "bib-spike ready").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update(params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.update(params.text_document.uri, change.text).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let p = params.text_document_position_params;
        Ok(self.hit(&p.text_document.uri, p.position).await.map(|(_, hit)| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**bib-spike** hover for `{}`", hit.key),
            }),
            range: Some(range(&hit)),
        }))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let p = params.text_document_position_params;
        let uri = p.text_document.uri.clone();
        Ok(self.hit(&p.text_document.uri, p.position).await.map(|_| {
            GotoDefinitionResponse::Scalar(Location::new(uri, Range::new(Position::new(0, 0), Position::new(0, 0))))
        }))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let p = params.text_document_position;
        let uri = p.text_document.uri.clone();
        Ok(self.hit(&p.text_document.uri, p.position).await.map(|(text, hit)| {
            find_keys(&text)
                .iter()
                .filter(|other| other.key == hit.key)
                .map(|other| Location::new(uri.clone(), range(other)))
                .collect()
        }))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(self.hit(&params.text_document.uri, params.range.start).await.map(|(_, hit)| {
            vec![CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("bib-spike: action for {}", hit.key),
                kind: Some(CodeActionKind::QUICKFIX),
                ..Default::default()
            })]
        }))
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![CompletionItem::new_simple(
            "bibspike2026".into(),
            "bib-spike completion".into(),
        )])))
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Spike { client, docs: Mutex::new(HashMap::new()) });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket).serve(service).await;
}
```

Run: `cargo build -p lsp-coexist`
Expected: compiles without errors.

- [ ] **Step 4: Create the Zed extension and test document**

`spikes/zed-coexist-ext/extension.toml`:

```toml
id = "bib-spike"
name = "bib spike"
description = "Throwaway test: second language server for Typst alongside tinymist"
version = "0.0.1"
schema_version = 1
authors = ["Anton Oehler <antonoehler@gmx.de>"]

[language_servers.bib-spike]
name = "bib spike"
languages = ["Typst"]
```

`spikes/zed-coexist-ext/Cargo.toml`:

```toml
[package]
name = "zed-bib-spike"
version = "0.0.1"
edition = "2024"
license = "MIT OR Apache-2.0"
publish = false

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[dependencies]
zed_extension_api = "0.7.0"

# Separate workspace: built for wasm32-wasip2, not with the main workspace.
[workspace]
```

`spikes/zed-coexist-ext/src/lib.rs`:

```rust
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct BibSpike;

impl zed::Extension for BibSpike {
    fn new() -> Self {
        BibSpike
    }

    fn language_server_command(&mut self, _id: &LanguageServerId, worktree: &zed::Worktree) -> Result<zed::Command> {
        let command = worktree
            .which("lsp-coexist")
            .ok_or_else(|| "lsp-coexist not in PATH: `cargo install --path spikes/lsp-coexist`".to_string())?;
        Ok(zed::Command { command, args: vec![], env: vec![] })
    }
}

zed::register_extension!(BibSpike);
```

`spikes/typst-sample/main.typ`:

```typst
= Test document for bib-spike

Differential Privacy according to @dwork2006 and again @dwork2006.

An unknown key: @unknown2020.

#bibliography("refs.bib")
```

`spikes/typst-sample/refs.bib`:

```bibtex
@inproceedings{dwork2006,
  title = {Differential Privacy},
  author = {Dwork, Cynthia},
  booktitle = {Automata, Languages and Programming},
  year = {2006},
}
```

Run: `rustup target add wasm32-wasip2 && cargo build --manifest-path spikes/zed-coexist-ext/Cargo.toml --target wasm32-wasip2`
Expected: target installed, build successful.

- [ ] **Step 5: Commit (Code)**

```bash
git add Cargo.toml Cargo.lock spikes
git commit -m "Add throwaway language server and Zed extension for coexistence test

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

- [ ] **Step 6: Obtain the user's consent**

Tell the user what will happen before any change to their Zed installation, and wait for an explicit yes:

1. `cargo install --path spikes/lsp-coexist` (places `~/.cargo/bin/lsp-coexist`)
2. In Zed: *Install Dev Extension* → `spikes/zed-coexist-ext`
3. If needed: `"languages": { "Typst": { "language_servers": ["tinymist", "bib-spike", "..."] } }` in `~/.config/zed/settings.json`
4. After the test: remove the dev extension and run `cargo uninstall lsp-coexist`

Without consent, stop here and report Task 9 as open.

- [ ] **Step 7: Run the test in Zed**

With consent: run steps 1 and 2, then open `zed spikes/typst-sample/main.typ`. In the log (*zed: open log*), check that `tinymist` **and** `bib-spike` have started; if `bib-spike` doesn't start, run step 3 (consent already given) and restart Zed.

Then try every function on `@dwork2006` on line 3 and record **which server** is visible for each:

| Function | Trigger | Expectation if Zed merges them |
|---|---|---|
| Diagnostics | Open the file | hints "bib-spike sees @…" alongside tinymist's message for `@unknown2020` |
| Hover | Mouse over `@dwork2006` | tinymist's citation info **and** "bib-spike hover for `dwork2006`" |
| Go to Definition | F12 | targets from both servers (bibliography and line 1) |
| Find All References | Shift+F12 | one multibuffer with both occurrences, no duplicates |
| Code Actions | Ctrl+. | "bib-spike: action for dwork2006" alongside any tinymist actions |
| Completion | Type `@` | tinymist's keys **and** `bibspike2026` |

- [ ] **Step 8: Write up the log**

`docs/research/14-zed-two-language-servers.md`:

```markdown
# Zed with Two Typst Language Servers (Step 0b)

As of: <date of the test>. Zed <output of `zed --version`>, tinymist <version from the Zed log>.

Setup: `spikes/lsp-coexist` (throwaway server) via the dev extension `spikes/zed-coexist-ext`,
test document `spikes/typst-sample/main.typ`. Settings change needed: <yes/no, which>.

| Function | tinymist visible | bib-spike visible | Result |
|---|---|---|---|
| Diagnostics | | | |
| Hover | | | |
| Go to Definition | | | |
| Find All References | | | |
| Code Actions | | | |
| Completion | | | |

## Consequences for Spec §9

<Which planned editor features hold up, which don't, and what to use instead.>
```

The angle brackets and empty cells are filled in with the observations from Step 7; the document is only committed once it has real values.

- [ ] **Step 9: Clean up**

Remove the dev extension in Zed, run `cargo uninstall lsp-coexist`, revert any settings change. Confirm to the user that the original state has been restored.

- [ ] **Step 10: Commit (log)**

```bash
git add docs/research/14-zed-two-language-servers.md
git commit -m "Record Zed behaviour with two Typst language servers

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 10: Evaluate the Benchmark and Update the Spec

**Files:**
- Create: `bench/results/<date>.md`
- Modify: `bench/README.md`, `docs/superpowers/specs/2026-09-16-library-core-typst-design.md` (§7, §9, §17, §18)

**Interfaces:**
- Consumes: the report from Task 8, the corpus from Task 7, the log from Task 9
- Produces: the chosen default backend and cascade backend for Plan 2

- [ ] **Step 1: Record the decision rule**

Append to `bench/README.md`:

```markdown
## Decision Rule (Step 0a)

R(b) = sentences found / all sentences across the corpus, O(b) = correct order pairs / all pairs.
`mutool` is reference only (AGPL, not selectable as the default).

1. **`pdf_oxide` remains the default** if R(pdf_oxide) ≥ max(R(pdfium), R(mutool)) − 0.05
   **and** O(pdf_oxide) ≥ max(O(pdfium), O(mutool)) − 0.05
   **and** for every document where pdf_oxide is missing characters or has control characters/`(cid:`/U+FFFD,
   `pdf-extract` supplies all expected characters (the cascade covers it).
2. **Otherwise pdfium**, if it satisfies condition 1 with pdfium in place of pdf_oxide.
3. **Otherwise stop** and discuss the results with the user.
```

- [ ] **Step 2: Run the full benchmark**

```bash
cargo run -p extract-bench -- fetch
BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib \
  cargo run --release -p extract-bench --features pdfium -- run --out bench/results/$(date +%F).md
```

Expected: report with 7 documents × 4 backends. If there are sentences that **no** backend finds, go back to Task 7 Step 4 and fix the assertion, then run again.

- [ ] **Step 3: Apply the rule**

Read R and O per backend from the totals table, check the three conditions in order, and write the result with the numbers below the report:

```markdown
## Decision

- R: pdf_oxide …, pdfium …, mutool …, pdf-extract …
- O: pdf_oxide …, pdfium …, mutool …, pdf-extract …
- Documents with character problems in pdf_oxide: … — covered by pdf-extract: yes/no
- **Default backend:** …  **Cascade on suspicion:** …
```

If rule 3 applies: stop here, commit the report, and present it to the user.

- [ ] **Step 4: Update the spec**

In `docs/superpowers/specs/2026-09-16-library-core-typst-design.md`:

- §7 "Cascade, per page": record the default backend and cascade according to the decision, with a reference to `bench/results/<date>.md`.
- §9: change the paragraph about Zed, per `docs/research/14-zed-two-language-servers.md`, from "not tested in practice" to the result.
- §17: add the rows "Benchmark (step 0a)" and "Zed with two language servers (step 0b)" with the key figures; remove the row "From docs and issues, not tested in practice".
- §18: strike the bullet points about the benchmark result and the Zed test.

- [ ] **Step 5: Full check**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS with no warnings. Fix clippy findings, don't suppress them.

- [ ] **Step 6: Commit**

```bash
git add bench/README.md bench/results docs/superpowers/specs/2026-09-16-library-core-typst-design.md
git commit -m "Record extraction benchmark decision and update spec

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```
