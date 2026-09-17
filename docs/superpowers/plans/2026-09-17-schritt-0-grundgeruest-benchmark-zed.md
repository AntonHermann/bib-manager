# Schritt 0: Grundgerüst, Extraktions-Benchmark und Zed-Test — Implementierungsplan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rust-Workspace mit Normalisierung und austauschbaren PDF-Backends aufsetzen, die Backends an einem kuratierten Korpus messen (Schritt 0a) und prüfen, ob Zed einen zweiten Typst-Language-Server neben tinymist sauber zusammenführt (Schritt 0b).

**Architecture:** Ein Cargo-Workspace. `bib-core` enthält vorerst nur die Normalisierung (Vergleichsform mit Rückweg zum Rohtext). `bib-extract` definiert die Backend-Schnittstelle (Text Pflicht, Spans optional, ein gemeinsames Koordinatensystem) mit vier Backends: `pdf_oxide`, `pdf-extract`, `mutool` als externer Prozess und `pdfium` hinter einem Feature. `tools/extract-bench` lädt den Korpus, misst jedes Backend gegen handkuratierte Aussagen und schreibt einen Markdown-Bericht. `spikes/` enthält Wegwerf-Code für den Zed-Test.

**Tech Stack:** Rust (Edition 2024, rustc 1.95), `unicode-normalization`, `pdf_oxide` 0.3.78, `pdf-extract` 0.12.0, `pdfium-render` 0.9.4, `thiserror`, `anyhow`, `clap`, `serde`, `toml`, `sha2`, `tempfile`, `tower-lsp-server` 0.23, `tokio`, `zed_extension_api` 0.7.0; externe Programme `curl`, `mutool` (optional).

**Spec:** `docs/superpowers/specs/2026-09-16-bibliothek-kern-typst-design.md` (Abschnitte 7, 9, 14, 15, 17, 18)

**Folgepläne** (werden mit den Ergebnissen dieses Plans geschrieben): Plan 2 = Spec-Schritte 1–3 (Datenmodell, Zotero-Abgleich, Extraktion mit Kaskade), Plan 3 = Schritte 4–6 (Anker, Typst-Parser, Import), Plan 4 = Schritte 7–8 (Language Server, Zed-Extension, Export).

## Global Constraints

- Lizenz des Projekts: `MIT OR Apache-2.0`. **Keine AGPL-Abhängigkeit im Workspace** (kein `mupdf-rs`, kein PyMuPDF). `mutool` nur als optionaler externer Prozess.
- Rust: Edition 2024, `rust-version = "1.95"`.
- Normalisierung (Vergleichsform), in dieser Reihenfolge und kompatibel zu `check_quotes.py`: NFKC; weiche Trennstriche (U+00AD) entfernen; `-\s*\n\s*` (nur ASCII-Bindestrich) entfernen; `‐‑‒–—` → `-`; `‘’‚‛` → `'`; `“”„‟` → `"`; Leerraum zu einem Leerzeichen zusammenfassen; vorn und hinten trimmen; Kleinschreibung.
- Koordinaten in `bib-extract`: Punkte, Ursprung oben links, y wächst nach unten, relativ zur MediaBox.
- Backend-Schnittstelle: Text ist Pflicht, Geometrie optional (Spec §7).
- Zed: Die Sprache heißt exakt `Typst`; Extensions werden für `wasm32-wasip2` gebaut; `zed_extension_api` 0.7.0 verlangt Edition 2024.
- Das Seminar-Repo `~/Documents/seminar_ehr_ss26` wird bis einschließlich 23.09.2026 **nicht** gelesen oder benutzt.
- Änderungen an der Zed-Installation oder `~/.config/zed/settings.json` nur nach ausdrücklicher Zustimmung des Nutzers.
- Doku und Kommentare auf Deutsch, Bezeichner auf Englisch.
- Jeder Commit endet mit:
  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE
  ```

## Dateistruktur

```
Cargo.toml                         Workspace
LICENSE-MIT, LICENSE-APACHE
.gitignore
crates/bib-core/
  Cargo.toml
  src/lib.rs                       Modulliste
  src/normalize.rs                 Vergleichsform + Rückweg (Task 1)
crates/bib-extract/
  Cargo.toml
  src/lib.rs                       Typen, Backend-Trait, Panik-Schutz (Task 3)
  src/join.rs                      Spans → Text (Task 3)
  src/backends/mod.rs
  src/backends/oxide.rs            pdf_oxide (Task 3)
  src/backends/pdf_extract.rs      pdf-extract, nur Text (Task 4)
  src/backends/mutool.rs           mutool als Prozess (Task 5)
  src/backends/stext.rs            Parser für mutools stext-XML (Task 5)
  src/backends/pdfium.rs           pdfium, Feature `pdfium` (Task 6)
  tests/common/mod.rs              Fixture-Pfade
  tests/oxide.rs, tests/pdf_extract.rs, tests/coordinates.rs, tests/pdfium.rs
tools/extract-bench/
  Cargo.toml
  src/main.rs                      CLI: fetch, run
  src/corpus.rs                    corpus.toml + corpus.lock (Task 2)
  src/metrics.rs                   Bewertung eines Textes (Task 8)
  src/report.rs                    Markdown-Bericht (Task 8)
bench/
  corpus.toml                      Korpus + erwartete Aussagen (Task 2, 7)
  corpus.lock                      SHA-256 je Dokument (Task 2, 7)
  README.md                        Kurationsregeln, Entscheidungsregel (Task 7, 10)
  results/                         Berichte (Task 10)
  cache/                           Downloads, nicht versioniert
spikes/lsp-coexist/                Wegwerf-Language-Server (Task 9)
spikes/zed-coexist-ext/            Wegwerf-Zed-Extension, eigener Workspace (Task 9)
spikes/typst-sample/               Testdokument für Zed (Task 9)
docs/research/14-zed-zwei-language-server.md   Protokoll des Zed-Tests (Task 9)
```

---

### Task 1: Workspace-Grundgerüst und Normalisierung

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `LICENSE-MIT`, `LICENSE-APACHE`
- Create: `crates/bib-core/Cargo.toml`, `crates/bib-core/src/lib.rs`, `crates/bib-core/src/normalize.rs`

**Interfaces:**
- Produces:
  - `bib_core::normalize::NORMALIZATION_VERSION: u32`
  - `bib_core::normalize::normalize(input: &str) -> Normalized`
  - `bib_core::normalize::Normalized { pub text: String }` mit `fn source_range(&self, out: std::ops::Range<usize>) -> Option<std::ops::Range<usize>>` (Byte-Bereich im Rohtext zu einem Byte-Bereich in `text`)

- [ ] **Step 1: Workspace anlegen**

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
//! Kern des Bibliography Managers.

pub mod normalize;
```

Lizenzdateien:

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

- [ ] **Step 2: Tests für die Normalisierung schreiben**

`crates/bib-core/src/normalize.rs` (zunächst nur Tests und leere Signaturen, damit es kompiliert, aber fehlschlägt):

```rust
//! Vergleichsform für die Wortlaut-Suche (Spec §7, „Normalisierung").
//!
//! Gesucht wird immer in der Vergleichsform; über [`Normalized::source_range`]
//! führt jeder Treffer zurück zum Rohtext, an dem Anker und Boxen hängen.

use std::ops::Range;

/// Wird erhöht, sobald sich das Ergebnis von [`normalize`] für irgendeine Eingabe ändert.
pub const NORMALIZATION_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    pub text: String,
    /// Für jedes Byte von `text`: Byte-Bereich im Rohtext, aus dem es stammt.
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
            assert_eq!(normalize(&once).text, once, "Eingabe: {s:?}");
        }
    }
}
```

- [ ] **Step 3: Tests laufen lassen, Fehlschlag prüfen**

Run: `cargo test -p bib-core`
Expected: FAIL, alle Tests brechen mit `not yet implemented` ab.

- [ ] **Step 4: Normalisierung implementieren**

In `crates/bib-core/src/normalize.rs` den Import und die beiden `todo!()`-Körper ersetzen:

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

/// Ein Zeichen der Zwischenstufe: Zeichen, Quellbereich, darf an Zeilenende verbunden werden.
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

/// NFKC je Segment (Basiszeichen + kombinierende Zeichen), Ersetzungen, Kleinschreibung.
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

/// Trennung am Zeilenende verbinden, Leerraum zusammenfassen, trimmen.
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

- [ ] **Step 5: Tests laufen lassen**

Run: `cargo test -p bib-core`
Expected: PASS, 12 Tests.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml .gitignore LICENSE-MIT LICENSE-APACHE crates/bib-core
git commit -m "Add workspace and comparison normalization with source mapping

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 2: Benchmark-Werkzeug mit Korpus-Download

**Files:**
- Modify: `Cargo.toml` (Member, Abhängigkeiten)
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
  - Datei `bench/cache/<id>.pdf` für jedes Korpus-Dokument

- [ ] **Step 1: Crate anlegen**

In `Cargo.toml` `members` ergänzen und Abhängigkeiten hinzufügen:

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

- [ ] **Step 2: Tests für Korpus und Lock-Datei schreiben**

`tools/extract-bench/src/corpus.rs`:

```rust
//! Korpus-Beschreibung (`bench/corpus.toml`) und Prüfsummen (`bench/corpus.lock`).

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

/// Handkuratierte Aussagen über den Text eines Dokuments.
#[derive(Debug, Default, Deserialize)]
pub struct Expect {
    /// Sätze, die zusammenhängend im normalisierten Text stehen müssen.
    #[serde(default)]
    pub sentences: Vec<String>,
    /// Satzpaare: der erste muss vor dem zweiten stehen (Leserichtung).
    #[serde(default)]
    pub before: Vec<(String, String)>,
    /// Zeichen, die mindestens einmal vorkommen müssen (z. B. "ε").
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
            sentences = ["Eins zwei drei."]
            before = [["Eins", "drei"]]
            chars = ["ε"]

            [[doc]]
            id = "b"
            url = "https://example.org/b.pdf"
            category = "test"
            "#,
        )
        .unwrap();
        assert_eq!(corpus.docs.len(), 2);
        assert_eq!(corpus.docs[0].expect.sentences, vec!["Eins zwei drei."]);
        assert_eq!(corpus.docs[0].expect.before, vec![("Eins".to_string(), "drei".to_string())]);
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

`tools/extract-bench/src/main.rs` (vorläufig):

```rust
mod corpus;

fn main() {}
```

- [ ] **Step 3: Tests laufen lassen, Fehlschlag prüfen**

Run: `cargo test -p extract-bench`
Expected: FAIL mit `not yet implemented`.

- [ ] **Step 4: Korpus-Funktionen implementieren**

In `corpus.rs` die drei `todo!()`-Funktionen ersetzen:

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
Expected: PASS, 3 Tests.

- [ ] **Step 5: Befehl `fetch` implementieren**

`tools/extract-bench/src/main.rs`:

```rust
//! Extraktions-Benchmark (Spec §15, Schritt 0a).

mod corpus;

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

use corpus::{LockStatus, bench_dir, cache_path, check_lock, parse_corpus, sha256_hex};

#[derive(Parser)]
#[command(about = "Misst PDF-Backends an einem kuratierten Korpus")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Lädt alle Korpus-Dokumente nach bench/cache/ und prüft ihre SHA-256-Summen.
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
            println!("lade {} …", doc.id);
            let status = Command::new("curl")
                .args(["-sSfL", "--retry", "3", "-o"])
                .arg(&path)
                .arg(&doc.url)
                .status()
                .context("curl nicht ausführbar")?;
            if !status.success() {
                bail!("Download von {} fehlgeschlagen ({})", doc.id, doc.url);
            }
        }
        let sha = sha256_hex(&std::fs::read(&path)?);
        match check_lock(&lock, &doc.id, &sha) {
            LockStatus::Match => println!("ok    {}", doc.id),
            LockStatus::New => {
                println!("neu   {} {sha}", doc.id);
                lock.insert(doc.id.clone(), sha);
            }
            LockStatus::Mismatch { expected } => bail!(
                "Prüfsumme von {} weicht ab: erwartet {expected}, gefunden {sha}. \
                 Datei löschen und neu laden, oder corpus.lock bewusst anpassen.",
                doc.id
            ),
        }
    }
    std::fs::write(&lock_path, toml::to_string(&lock)?)?;
    Ok(())
}
```

- [ ] **Step 6: Anfangskorpus anlegen und laden**

`bench/corpus.toml`:

```toml
# Korpus für den Extraktions-Benchmark. Kurationsregeln: bench/README.md.

[[doc]]
id = "dwork2006"
url = "https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/dwork.pdf"
category = "altes LaTeX, Type1-Mathe-Schrift ohne Unicode-Zuordnung"
[doc.expect]
sentences = ["This is captured by differential privacy."]
chars = ["ε"]

[[doc]]
id = "abadi2016"
url = "https://arxiv.org/pdf/1607.00133v2"
category = "modernes zweispaltiges ACM-Paper mit Formeln"
```

Run: `cargo run -p extract-bench -- fetch`
Expected: zwei Zeilen `neu   <id> <sha>`; `bench/corpus.lock` enthält beide Summen. Ein zweiter Lauf gibt zweimal `ok` aus.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock tools/extract-bench bench/corpus.toml bench/corpus.lock
git commit -m "Add extraction benchmark tool with pinned corpus download

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 3: Backend-Schnittstelle und `pdf_oxide`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/lib.rs`, `crates/bib-extract/src/join.rs`, `crates/bib-extract/src/backends/mod.rs`, `crates/bib-extract/src/backends/oxide.rs`
- Test: `crates/bib-extract/tests/common/mod.rs`, `crates/bib-extract/tests/oxide.rs`

**Interfaces:**
- Consumes: `bib_core::normalize::normalize` (nur in Tests); Fixture `bench/cache/dwork2006.pdf` aus Task 2
- Produces:
  - `bib_extract::Rect { left: f32, top: f32, right: f32, bottom: f32 }`
  - `bib_extract::Span { text: String, bbox: Rect, font: String }`
  - `bib_extract::PageSize { width: f32, height: f32, rotation: Option<u16> }`
  - `bib_extract::PageContent { Spans(Vec<Span>), Plain(String) }`
  - `bib_extract::Page { index: usize, size: Option<PageSize>, content: PageContent }` mit `fn text(&self) -> String`, `fn has_geometry(&self) -> bool`
  - `bib_extract::Extraction { backend: &'static str, backend_version: String, pages: Vec<Page> }` mit `fn text(&self) -> String`
  - `bib_extract::ExtractError { Open(String), Page { index: usize, message: String }, Panicked(String), Unavailable(String) }`
  - `trait bib_extract::Backend { fn name(&self) -> &'static str; fn version(&self) -> String; fn extract(&self, path: &Path) -> Result<Extraction, ExtractError>; }`
  - `bib_extract::join_spans(spans: &[Span]) -> String`
  - `bib_extract::backends::oxide::PdfOxide` (Unit-Struct, implementiert `Backend`)
  - Testhilfe `common::fixture(id: &str) -> PathBuf`

- [ ] **Step 1: Crate anlegen**

`Cargo.toml`: `"crates/bib-extract"` zu `members`; Abhängigkeiten ergänzen:

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

- [ ] **Step 2: Tests für das Zusammenfügen von Spans schreiben**

`crates/bib-extract/src/lib.rs`:

```rust
//! PDF-Textextraktion hinter einer austauschbaren Schnittstelle (Spec §7).

pub mod backends;
mod join;

use std::path::Path;

pub use join::join_spans;

/// Rechteck in Punkten, **Ursprung oben links, y wächst nach unten**, relativ zur MediaBox.
/// Jedes Backend rechnet in dieses System um.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Zusammenhängendes Textstück mit Box und Schrift, in Leserichtung.
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
    /// `None`, wenn das Backend die Drehung nicht kennt.
    pub rotation: Option<u16>,
}

/// Text ist Pflicht, Geometrie optional (Spec §7).
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
    /// Text aller Seiten, durch Zeilenumbrüche getrennt.
    pub fn text(&self) -> String {
        self.pages.iter().map(Page::text).collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("PDF konnte nicht geöffnet werden: {0}")]
    Open(String),
    #[error("Seite {index}: {message}")]
    Page { index: usize, message: String },
    #[error("Backend ist abgestürzt: {0}")]
    Panicked(String),
    #[error("Werkzeug nicht verfügbar: {0}")]
    Unavailable(String),
}

pub trait Backend {
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError>;
}

/// Fängt Paniken eines Backends ab (`pdf-extract` bricht bei kaputten Dateien hart ab).
pub(crate) fn guard<T>(f: impl FnOnce() -> Result<T, ExtractError>) -> Result<T, ExtractError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unbekannte Ursache".to_string());
            Err(ExtractError::Panicked(message))
        }
    }
}
```

`crates/bib-extract/src/join.rs`:

```rust
//! Spans in Leserichtung zu Text zusammenfügen.

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

`crates/bib-extract/src/backends/oxide.rs` (vorläufig leer bis Step 6):

```rust
//! Backend auf Basis von `pdf_oxide` (MIT/Apache-2.0, reines Rust).
```

- [ ] **Step 3: Tests laufen lassen, Fehlschlag prüfen**

Run: `cargo test -p bib-extract --lib`
Expected: FAIL mit `not yet implemented` in vier Tests.

- [ ] **Step 4: `join_spans` implementieren**

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
Expected: PASS, 5 Tests.

- [ ] **Step 5: Integrationstest für `pdf_oxide` schreiben**

`crates/bib-extract/tests/common/mod.rs`:

```rust
use std::path::PathBuf;

/// Pfad eines Korpus-Dokuments; bricht mit Anleitung ab, wenn es fehlt.
pub fn fixture(id: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/cache")
        .join(format!("{id}.pdf"));
    assert!(
        path.exists(),
        "Fixture {id} fehlt: zuerst `cargo run -p extract-bench -- fetch` ausführen"
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
        assert!(page.has_geometry(), "Seite {} ohne Geometrie", page.index);
    }
}

#[test]
fn first_page_size_is_plausible() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    let size = extraction.pages[0].size.expect("Seitengröße");
    assert!((500.0..700.0).contains(&size.width), "Breite {}", size.width);
    assert!((700.0..900.0).contains(&size.height), "Höhe {}", size.height);
}

#[test]
fn title_is_near_the_top_in_top_left_coordinates() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    let page = &extraction.pages[0];
    let PageContent::Spans(spans) = &page.content else { panic!("keine Spans") };
    let title_top = spans
        .iter()
        .filter(|s| s.text.contains("Differential"))
        .map(|s| s.bbox.top)
        .fold(f32::INFINITY, f32::min);
    let height = page.size.unwrap().height;
    assert!(title_top < 0.25 * height, "Titel bei top={title_top}, Seitenhöhe {height}");
}

#[test]
fn text_contains_the_title_after_normalization() {
    let extraction = PdfOxide.extract(&common::fixture("dwork2006")).unwrap();
    assert!(normalize(&extraction.text()).text.contains("differential privacy"));
}
```

Run: `cargo test -p bib-extract --test oxide`
Expected: FAIL beim Kompilieren, `PdfOxide` existiert nicht.

- [ ] **Step 6: `pdf_oxide`-Backend implementieren**

`crates/bib-extract/src/backends/oxide.rs`:

```rust
//! Backend auf Basis von `pdf_oxide` (MIT/Apache-2.0, reines Rust).
//!
//! `pdf_oxide` liefert Boxen in PDF-Koordinaten (Ursprung unten links, `y` ist die
//! Unterkante). Umrechnung: top = ury - (y + height), bottom = ury - y.

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

    // Ohne eigene MediaBox (z. B. geerbt) keine verlässliche Umrechnung: nur Text, ehrlich ohne Geometrie.
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

Die Behelfsboxen im Zweig ohne MediaBox dienen nur `join_spans` (gleiche relative Lage, gespiegelte y-Achse), sie werden nicht gespeichert.

- [ ] **Step 7: Tests laufen lassen**

Run: `cargo test -p bib-extract`
Expected: PASS (5 Unit-Tests, 4 Integrationstests). Schlägt `title_is_near_the_top_in_top_left_coordinates` fehl, ist die Annahme über `pdf_oxide`s `y` falsch: den Wert von `s.bbox.y` für den Titel ausgeben (`dbg!`) und die Umrechnung sowie den Modulkommentar entsprechend korrigieren, nicht den Test.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/bib-extract
git commit -m "Add extraction backend interface and pdf_oxide backend

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 4: `pdf-extract`-Backend (nur Text)

**Files:**
- Modify: `Cargo.toml`, `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/backends/mod.rs`
- Create: `crates/bib-extract/src/backends/pdf_extract.rs`
- Test: `crates/bib-extract/tests/pdf_extract.rs`

**Interfaces:**
- Consumes: `Backend`, `Extraction`, `Page`, `PageContent`, `ExtractError`, `guard` aus Task 3
- Produces: `bib_extract::backends::pdf_extract::PdfExtract` (Unit-Struct, `Backend`), Name `"pdf-extract"`, Seiten immer `PageContent::Plain`, `size: None`

- [ ] **Step 1: Abhängigkeiten ergänzen**

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

- [ ] **Step 2: Test schreiben**

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
    // Gemessen im Brainstorming: 28 ε in Dwork 2006 (pdf_oxide: 0).
    let extraction = PdfExtract.extract(&common::fixture("dwork2006")).unwrap();
    let epsilons = normalize(&extraction.text()).text.matches('ε').count();
    assert!(epsilons >= 25, "nur {epsilons} ε gefunden");
}

#[test]
fn broken_pdf_is_an_error_not_a_panic() {
    let mut file = tempfile::Builder::new().suffix(".pdf").tempfile().unwrap();
    file.write_all(b"%PDF-1.4\nkein echtes PDF\n").unwrap();
    assert!(PdfExtract.extract(file.path()).is_err());
}
```

Run: `cargo test -p bib-extract --test pdf_extract`
Expected: FAIL beim Kompilieren, Modul `pdf_extract` fehlt.

- [ ] **Step 3: Backend implementieren**

`crates/bib-extract/src/backends/mod.rs`:

```rust
pub mod oxide;
pub mod pdf_extract;
```

`crates/bib-extract/src/backends/pdf_extract.rs`:

```rust
//! Backend auf Basis von `pdf-extract` (MIT). Liefert nur Text, keine Geometrie,
//! liest aber Glyphennamen aus eingebetteten Type1-Schriften (ε in altem LaTeX).

use std::path::Path;

use crate::{Backend, ExtractError, Extraction, Page, PageContent, guard};

/// Muss zur exakten Version in `Cargo.toml` (`=0.12.0`) passen.
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

- [ ] **Step 4: Tests laufen lassen**

Run: `cargo test -p bib-extract --test pdf_extract`
Expected: PASS, 3 Tests. `pdf-extract` gibt dabei eventuell Warnungen und bei `broken_pdf…` eine Panik-Meldung auf stderr aus; entscheidend ist das Testergebnis.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/bib-extract
git commit -m "Add text-only pdf-extract backend with panic isolation

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 5: `mutool`-Backend und Koordinatenabgleich

**Files:**
- Modify: `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/backends/mod.rs`
- Create: `crates/bib-extract/src/backends/stext.rs`, `crates/bib-extract/src/backends/mutool.rs`
- Test: `crates/bib-extract/tests/coordinates.rs`

**Interfaces:**
- Consumes: Typen aus Task 3, `PdfOxide`
- Produces:
  - `bib_extract::backends::stext::parse_stext(xml: &str) -> Vec<Page>` (je `<font>`-Element innerhalb einer Zeile ein Span; `rotation: None`)
  - `bib_extract::backends::mutool::Mutool { program: PathBuf }` mit `Default` (`"mutool"`) und `fn is_available(&self) -> bool`; Name `"mutool"`

- [ ] **Step 1: Tests für den stext-Parser schreiben**

`crates/bib-extract/src/backends/mod.rs`:

```rust
pub mod mutool;
pub mod oxide;
pub mod pdf_extract;
pub mod stext;
```

`crates/bib-extract/src/backends/stext.rs`:

```rust
//! Parser für mutools strukturiertes Textformat (`mutool draw -F stext`).
//! Koordinaten sind dort bereits oben links verankert.

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
        let PageContent::Spans(spans) = &pages[0].content else { panic!("keine Spans") };
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
Expected: FAIL mit `not yet implemented`.

- [ ] **Step 2: Parser implementieren**

`parse_stext` ersetzen und Hilfsfunktionen ergänzen:

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

/// Inhalte aller Tags ohne spitze Klammern. `<` ist in XML-Attributen immer maskiert,
/// `>` nicht zwingend (etwa `c=">"`); deshalb wird am letzten `>` vor dem nächsten `<` getrennt.
/// Zwischen den Tags steht in mutools Ausgabe nur Leerraum.
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

Den Import `use crate::Page;` oben durch die obige Zeile ersetzen.

Run: `cargo test -p bib-extract --lib stext`
Expected: PASS.

- [ ] **Step 3: Koordinatentest schreiben**

`crates/bib-extract/Cargo.toml` → `[dependencies]` um `tempfile.workspace = true` ergänzen (wird vom Backend gebraucht; in `[dev-dependencies]` kann es bleiben).

`crates/bib-extract/tests/coordinates.rs`:

```rust
//! Dieselbe Textstelle muss in allen Backends mit Geometrie an derselben Stelle liegen (Spec §7).

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::oxide::PdfOxide;
use bib_extract::{Backend, PageContent, Rect};

const TOLERANCE: f32 = 6.0;

pub fn first_box_containing(extraction: &bib_extract::Extraction, needle: &str) -> Rect {
    let PageContent::Spans(spans) = &extraction.pages[0].content else { panic!("{}: keine Spans", extraction.backend) };
    spans
        .iter()
        .filter(|s| s.text.contains(needle))
        .min_by(|a, b| a.bbox.top.total_cmp(&b.bbox.top))
        .unwrap_or_else(|| panic!("{}: „{needle}“ nicht gefunden", extraction.backend))
        .bbox
}

#[test]
fn pdf_oxide_and_mutool_agree_on_title_position() {
    let mutool = Mutool::default();
    if !mutool.is_available() {
        eprintln!("ÜBERSPRUNGEN: mutool nicht installiert");
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
Expected: FAIL beim Kompilieren, `Mutool` fehlt.

- [ ] **Step 4: `mutool`-Backend implementieren**

`crates/bib-extract/src/backends/mutool.rs`:

```rust
//! `mutool` als externer Prozess (optional). mutool ist AGPL-lizenziert und wird
//! deshalb nie gelinkt, nur aufgerufen.

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
            .unwrap_or_else(|| "unbekannt".to_string())
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        let out_file = tempfile::Builder::new()
            .suffix(".xml")
            .tempfile()
            .map_err(|e| ExtractError::Unavailable(format!("Temporärdatei: {e}")))?;
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

- [ ] **Step 5: Tests laufen lassen**

Run: `cargo test -p bib-extract`
Expected: PASS. `coordinates` darf nicht mit „ÜBERSPRUNGEN" enden, `mutool` ist auf dem Entwicklungsrechner installiert (`mutool version 1.25.1`). Schlägt der Koordinatentest fehl, ist die Umrechnung in `oxide.rs` falsch: korrigieren, nicht die Toleranz erhöhen.

- [ ] **Step 6: Commit**

```bash
git add crates/bib-extract Cargo.lock
git commit -m "Add mutool subprocess backend and cross-backend coordinate check

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 6: pdfium-Backend hinter Feature

**Files:**
- Modify: `Cargo.toml`, `crates/bib-extract/Cargo.toml`, `crates/bib-extract/src/backends/mod.rs`
- Create: `crates/bib-extract/src/backends/pdfium.rs`
- Test: `crates/bib-extract/tests/pdfium.rs`

**Interfaces:**
- Consumes: Typen aus Task 3, `Mutool` und `first_box_containing`-Logik aus Task 5
- Produces: `bib_extract::backends::pdfium::Pdfium { lib_dir: PathBuf }` mit `fn from_env() -> Option<Self>` (liest `BIB_PDFIUM_LIB_DIR`); Name `"pdfium"`; nur mit Feature `pdfium`

- [ ] **Step 1: pdfium-Bibliothek laden und Feature anlegen**

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

- [ ] **Step 2: Test schreiben**

`crates/bib-extract/tests/pdfium.rs`:

```rust
#![cfg(feature = "pdfium")]

mod common;

use bib_extract::backends::mutool::Mutool;
use bib_extract::backends::pdfium::Pdfium;
use bib_extract::{Backend, PageContent};

fn backend() -> Pdfium {
    Pdfium::from_env().expect("BIB_PDFIUM_LIB_DIR setzen, z. B. auf bench/cache/pdfium/lib")
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
        eprintln!("ÜBERSPRUNGEN: mutool nicht installiert");
        return;
    }
    let path = common::fixture("dwork2006");
    let top_of = |e: &bib_extract::Extraction| {
        let PageContent::Spans(spans) = &e.pages[0].content else { panic!("keine Spans") };
        spans.iter().filter(|s| s.text.contains("Differential")).map(|s| s.bbox.top).fold(f32::INFINITY, f32::min)
    };
    let reference = top_of(&mutool.extract(&path).unwrap());
    let pdfium = top_of(&backend().extract(&path).unwrap());
    assert!((reference - pdfium).abs() < 6.0, "top: mutool {reference} vs pdfium {pdfium}");
}
```

Run: `BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib cargo test -p bib-extract --features pdfium --test pdfium`
Expected: FAIL beim Kompilieren, Modul `pdfium` fehlt.

- [ ] **Step 3: Backend implementieren**

`crates/bib-extract/src/backends/pdfium.rs`:

```rust
//! Backend auf Basis von pdfium (Chromes PDF-Engine) über `pdfium-render`.
//! Braucht `libpdfium.so` zur Laufzeit; Pfad über `BIB_PDFIUM_LIB_DIR`.

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
        format!("pdfium-render 0.9.4, libpdfium aus {}", self.lib_dir.display())
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

- [ ] **Step 4: Tests laufen lassen**

Run: `BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib cargo test -p bib-extract --features pdfium`
Expected: PASS, alle Tests inklusive `pdfium`. Außerdem `cargo test -p bib-extract` ohne Feature: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/bib-extract
git commit -m "Add feature-gated pdfium backend for benchmark comparison

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 7: Korpus kuratieren

Reine Datenarbeit, kein Code. Ergebnis sind überprüfte Aussagen, gegen die Task 8 misst.

**Files:**
- Modify: `bench/corpus.toml`, `bench/corpus.lock`
- Create: `bench/README.md`

**Interfaces:**
- Consumes: Format `Expect` aus Task 2
- Produces: sieben Dokumente mit je 3 Sätzen, 1 Reihenfolge-Paar und ggf. Zeichen

- [ ] **Step 1: Kurationsregeln festhalten**

`bench/README.md`:

````markdown
# Extraktions-Benchmark

Misst PDF-Backends an handkuratierten Aussagen (Stil olmOCR-Bench): Stehen bekannte Sätze
zusammenhängend und in der richtigen Reihenfolge im normalisierten Text? Sind erwartete
Sonderzeichen vorhanden? Wie viel Datenmüll entsteht?

```sh
cargo run -p extract-bench -- fetch
BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib \
  cargo run --release -p extract-bench --features pdfium -- run --out bench/results/<datum>.md
```

## Kuration

Pro Dokument:

- **Drei Sätze:** (a) der erste Satz des Abstracts, (b) ein Satz aus der rechten Spalte
  bzw. der zweiten Hälfte von Seite 2, (c) ein Satz aus dem letzten Absatz vor den Referenzen.
- **Regeln für Sätze:** mindestens 8 Wörter, keine Formeln, keine Zitatmarker, keine
  Fußnotenzeichen, endet mit Punkt. Wörtlich aus der LaTeX-Quelle, nicht aus einer
  PDF-Extraktion kopiert (sonst misst der Benchmark das Werkzeug, mit dem kuratiert wurde).
- **Ein Reihenfolge-Paar:** `before = [["<Anfang von b>", "<Anfang von c>"]]`, je ein
  eindeutiges Stück von mindestens 5 Wörtern.
- **Zeichen:** griechische Buchstaben, die im Fließtext gerendert werden (`\epsilon`,
  `\varepsilon` → `ε`; `\delta` → `δ`).
- **Prüfung gegen die Quelle:** jeder Satz muss in der LaTeX-Quelle stehen:
  `tr -s '[:space:]' ' ' < <datei>.tex | grep -F -c '<satz>'` ≥ 1.
- **Prüfung im PDF:** jeden Satz im PDF-Viewer suchen und sichtbar finden (Makros können den
  gerenderten Text verändern).
- Dokumente ohne LaTeX-Quelle (Dwork 2006): Sätze nur aus dem PDF-Viewer, per Augenschein
  gegen die Seite geprüft.

Findet nach dem ersten Lauf **kein einziges** Backend einen Satz, ist vermutlich die Aussage
falsch: im PDF prüfen und korrigieren, bevor Ergebnisse gewertet werden.
````

- [ ] **Step 2: Dokumente eintragen**

An `bench/corpus.toml` anhängen:

```toml
[[doc]]
id = "vaswani2017"
url = "https://arxiv.org/pdf/1706.03762v7"
category = "einspaltig, NeurIPS, Tabellen und Formeln"

[[doc]]
id = "devlin2019"
url = "https://arxiv.org/pdf/1810.04805v2"
category = "zweispaltig, ACL, Tabellen"

[[doc]]
id = "he2016"
url = "https://arxiv.org/pdf/1512.03385v1"
category = "zweispaltig, CVPR, Abbildungen und Tabellen"

[[doc]]
id = "shokri2017"
url = "https://arxiv.org/pdf/1610.05820v2"
category = "zweispaltig, IEEE S&P, Privacy-Thema"

[[doc]]
id = "carlini2021"
url = "https://arxiv.org/pdf/2012.07805v2"
category = "zweispaltig, USENIX Security, lange Anhänge"
```

Run: `cargo run -p extract-bench -- fetch`
Expected: fünf neue Einträge `neu …`, zwei `ok`.

- [ ] **Step 3: LaTeX-Quellen laden**

```bash
for id in 1607.00133v2 1706.03762v7 1810.04805v2 1512.03385v1 1610.05820v2 2012.07805v2; do
  mkdir -p bench/cache/src/$id
  curl -sSfL "https://arxiv.org/e-print/$id" -o bench/cache/src/$id.src
  tar xzf bench/cache/src/$id.src -C bench/cache/src/$id 2>/dev/null \
    || gunzip -c bench/cache/src/$id.src > bench/cache/src/$id/main.tex
  ls bench/cache/src/$id/*.tex
done
```

Expected: je Dokument mindestens eine `.tex`-Datei.

- [ ] **Step 4: Aussagen kuratieren**

Für `abadi2016`, `vaswani2017`, `devlin2019`, `he2016`, `shokri2017`, `carlini2021` je einen `[doc.expect]`-Block nach den Regeln aus `bench/README.md` unter den jeweiligen `[[doc]]` eintragen. Für `dwork2006` zwei weitere Sätze (b) und (c) und ein Reihenfolge-Paar aus dem PDF ergänzen. Form:

```toml
[doc.expect]
sentences = [
  "<Satz a>",
  "<Satz b>",
  "<Satz c>",
]
before = [["<Stück aus b>", "<Stück aus c>"]]
chars = ["ε", "δ"]
```

Die Platzhalter in spitzen Klammern sind hier Formatbeschreibung; eingetragen werden die kuratierten Sätze.

- [ ] **Step 5: Jeden Satz gegen die Quelle prüfen**

```bash
check() { tr -s '[:space:]' ' ' < "$1" | grep -F -c -- "$2"; }
# Beispiel für einen Satz aus abadi2016:
check <(cat bench/cache/src/1607.00133v2/*.tex) "<Satz>"
```

Expected: für jeden Satz der sechs arXiv-Dokumente eine Zahl ≥ 1. Sätze mit 0 ersetzen.

- [ ] **Step 6: Korpus parsen lassen**

Run: `cargo run -p extract-bench -- fetch`
Expected: siebenmal `ok`, kein Parse-Fehler.

- [ ] **Step 7: Commit**

```bash
git add bench/README.md bench/corpus.toml bench/corpus.lock
git commit -m "Curate benchmark corpus with verified sentence assertions

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 8: Messung und Bericht

**Files:**
- Modify: `tools/extract-bench/Cargo.toml`, `tools/extract-bench/src/main.rs`
- Create: `tools/extract-bench/src/metrics.rs`, `tools/extract-bench/src/report.rs`

**Interfaces:**
- Consumes: `bib_core::normalize::normalize`; `bib_extract::{Backend, Extraction}`; alle Backends; `corpus::{Expect, cache_path}`; `load_corpus()` aus `main.rs` (Task 2)
- Produces:
  - `metrics::TextScores { sentences_found, sentences_total, order_ok, order_total: usize, chars_missing: Vec<String>, control_chars, cid_markers, replacement_chars: usize }`
  - `metrics::evaluate(expect: &Expect, raw: &str) -> TextScores`
  - `report::Row { doc: String, backend: String, scores: Option<TextScores>, pages: usize, pages_with_geometry: usize, millis: u128, error: Option<String> }`
  - `report::render_markdown(rows: &[Row]) -> String`
  - CLI: `extract-bench run --out <datei> [--backends <liste>]`

- [ ] **Step 1: Tests für die Bewertung schreiben**

`tools/extract-bench/Cargo.toml` → `[dependencies]`:

```toml
bib-extract = { path = "../../crates/bib-extract" }
```

und

```toml
[features]
pdfium = ["bib-extract/pdfium"]
```

`tools/extract-bench/src/metrics.rs`:

```rust
//! Bewertung eines extrahierten Textes gegen die kuratierten Aussagen.

use crate::corpus::Expect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextScores {
    pub sentences_found: usize,
    pub sentences_total: usize,
    pub order_ok: usize,
    pub order_total: usize,
    pub chars_missing: Vec<String>,
    /// Steuerzeichen außer \n, \r, \t und Seitenvorschub.
    pub control_chars: usize,
    /// Vorkommen von `(cid:`.
    pub cid_markers: usize,
    /// Vorkommen von U+FFFD.
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

`tools/extract-bench/src/main.rs`: unter `mod corpus;` die Zeile `mod metrics;` ergänzen.

Run: `cargo test -p extract-bench metrics`
Expected: FAIL mit `not yet implemented`.

- [ ] **Step 2: Bewertung implementieren**

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
Expected: PASS, 4 Tests.

- [ ] **Step 3: Tests für den Bericht schreiben**

`tools/extract-bench/src/report.rs`:

```rust
//! Markdown-Bericht: eine Zeile je Dokument und Backend, darunter eine Summe je Backend.

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
            row("a", "mutool", None, Some("nicht installiert")),
        ]);
        assert!(md.contains("| a | pdf_oxide | 2/3 | 1/1 | ε | 2 | 0 | 0 | 6/6 | 120 |"), "{md}");
        assert!(md.contains("| a | mutool | Fehler: nicht installiert |"), "{md}");
        assert!(md.contains("| pdf_oxide | 5/6 | 1/2 | 2 | 4 | 0 | 0 | 0 |"), "{md}");
    }
}
```

`tools/extract-bench/src/main.rs`: `mod report;` ergänzen.

Run: `cargo test -p extract-bench report`
Expected: FAIL mit `not yet implemented`.

- [ ] **Step 4: Bericht implementieren**

```rust
pub fn render_markdown(rows: &[Row]) -> String {
    let mut md = String::new();
    md.push_str("## Details\n\n");
    md.push_str("| Dokument | Backend | Sätze | Reihenfolge | fehlende Zeichen | Steuerzeichen | (cid:) | U+FFFD | Seiten mit Geometrie | ms |\n");
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
                "| {} | {} | Fehler: {} |\n",
                row.doc,
                row.backend,
                error.as_deref().unwrap_or("unbekannt")
            )),
        }
    }

    md.push_str("\n## Summe je Backend\n\n");
    md.push_str("| Backend | Sätze | Reihenfolge | Dokumente mit fehlenden Zeichen | Steuerzeichen | (cid:) | U+FFFD | Fehler |\n");
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

- [ ] **Step 5: Befehl `run` implementieren**

In `tools/extract-bench/src/main.rs` ergänzen. Imports:

```rust
use std::path::PathBuf;
use std::time::Instant;

use bib_extract::Backend;
use bib_extract::backends::{mutool::Mutool, oxide::PdfOxide, pdf_extract::PdfExtract};
```

`Cmd` erweitern:

```rust
    /// Extrahiert alle Korpus-Dokumente mit allen verfügbaren Backends und schreibt einen Bericht.
    Run {
        /// Zieldatei des Markdown-Berichts, z. B. bench/results/2026-09-18.md
        #[arg(long)]
        out: PathBuf,
        /// Kommagetrennte Auswahl: pdf_oxide,pdf-extract,mutool,pdfium
        #[arg(long, value_delimiter = ',')]
        backends: Option<Vec<String>>,
    },
```

`main` erweitern:

```rust
        Cmd::Run { out, backends } => run(out, backends),
```

Funktionen:

```rust
fn available_backends(selection: Option<Vec<String>>) -> Vec<Box<dyn Backend>> {
    let mut all: Vec<Box<dyn Backend>> = vec![Box::new(PdfOxide), Box::new(PdfExtract)];
    let mutool = Mutool::default();
    if mutool.is_available() {
        all.push(Box::new(mutool));
    } else {
        eprintln!("mutool nicht installiert, übersprungen");
    }
    #[cfg(feature = "pdfium")]
    match bib_extract::backends::pdfium::Pdfium::from_env() {
        Some(pdfium) => all.push(Box::new(pdfium)),
        None => eprintln!("BIB_PDFIUM_LIB_DIR nicht gesetzt, pdfium übersprungen"),
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
        anyhow::ensure!(path.exists(), "{} fehlt, zuerst `fetch` ausführen", doc.id);
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
    let mut header = String::from("# Extraktions-Benchmark\n\n## Backends\n\n");
    for backend in &backends {
        header.push_str(&format!("- {} {}\n", backend.name(), backend.version()));
    }
    header.push('\n');
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, header + &report::render_markdown(&rows))?;
    println!("Bericht: {}", out.display());
    Ok(())
}
```

- [ ] **Step 6: Kurzlauf**

Run: `cargo run --release -p extract-bench -- run --out /dev/stdout --backends pdf_oxide,pdf-extract`
Expected: Bericht mit 14 Detailzeilen und 2 Summenzeilen auf stdout, keine Panik.

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

### Task 9: Zed-Test mit zwei Language Servern (Schritt 0b)

Wegwerf-Code. Er beantwortet eine Frage und wird danach nicht weiterentwickelt.

**Files:**
- Modify: `Cargo.toml`
- Create: `spikes/lsp-coexist/Cargo.toml`, `spikes/lsp-coexist/src/lib.rs`, `spikes/lsp-coexist/src/main.rs`
- Create: `spikes/zed-coexist-ext/extension.toml`, `spikes/zed-coexist-ext/Cargo.toml`, `spikes/zed-coexist-ext/src/lib.rs`
- Create: `spikes/typst-sample/main.typ`, `spikes/typst-sample/refs.bib`
- Create: `docs/research/14-zed-zwei-language-server.md`

**Interfaces:**
- Produces: Programm `lsp-coexist` (stdio-Language-Server), Dev-Extension `bib-spike`, Protokoll mit Ergebnis je Funktion

- [ ] **Step 1: Crate anlegen und Tests für die Key-Suche schreiben**

`Cargo.toml`: `"spikes/lsp-coexist"` zu `members`; Abhängigkeiten:

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
//! Wegwerf-Code für Schritt 0b: findet `@key`-Referenzen in Typst-Text.
//! Spalten in UTF-16-Codeeinheiten, wie LSP sie verlangt.

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
        let hits = find_keys("Laut @dwork2006.\nUnd @a:b-c_d, fertig.");
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
        assert!(find_keys("mail an a@b.de und \\@nicht").is_empty());
    }

    #[test]
    fn columns_are_utf16() {
        let hits = find_keys("𝜖 @x1");
        assert_eq!(hits[0].start, 3);
        assert_eq!(hits[0].end, 6);
    }

    #[test]
    fn key_at_hits_inside_and_at_edges() {
        let text = "siehe @dwork2006 hier";
        assert_eq!(key_at(text, 0, 6).unwrap().key, "dwork2006");
        assert_eq!(key_at(text, 0, 16).unwrap().key, "dwork2006");
        assert!(key_at(text, 0, 2).is_none());
    }
}
```

`spikes/lsp-coexist/src/main.rs` (vorläufig):

```rust
fn main() {}
```

Run: `cargo test -p lsp-coexist`
Expected: FAIL mit `not yet implemented`.

- [ ] **Step 2: Key-Suche implementieren**

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
Expected: PASS, 4 Tests.

- [ ] **Step 3: Language Server schreiben**

`spikes/lsp-coexist/src/main.rs`:

```rust
//! Wegwerf-Language-Server für Schritt 0b. Jede Antwort ist mit „bib-spike" markiert,
//! damit in Zed sichtbar ist, von welchem Server sie stammt.

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
                message: format!("bib-spike sieht @{}", hit.key),
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
        self.client.log_message(MessageType::INFO, "bib-spike bereit").await;
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
                value: format!("**bib-spike** Hover für `{}`", hit.key),
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
                title: format!("bib-spike: Aktion für {}", hit.key),
                kind: Some(CodeActionKind::QUICKFIX),
                ..Default::default()
            })]
        }))
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![CompletionItem::new_simple(
            "bibspike2026".into(),
            "bib-spike Vervollständigung".into(),
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
Expected: kompiliert ohne Fehler.

- [ ] **Step 4: Zed-Extension und Testdokument anlegen**

`spikes/zed-coexist-ext/extension.toml`:

```toml
id = "bib-spike"
name = "bib spike"
description = "Wegwerf-Test: zweiter Language Server für Typst neben tinymist"
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

# Eigener Workspace: wird für wasm32-wasip2 gebaut, nicht mit dem Haupt-Workspace.
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
            .ok_or_else(|| "lsp-coexist nicht im PATH: `cargo install --path spikes/lsp-coexist`".to_string())?;
        Ok(zed::Command { command, args: vec![], env: vec![] })
    }
}

zed::register_extension!(BibSpike);
```

`spikes/typst-sample/main.typ`:

```typst
= Testdokument für bib-spike

Differential Privacy nach @dwork2006 und nochmals @dwork2006.

Ein unbekannter Key: @unbekannt2020.

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
Expected: Target installiert, Build erfolgreich.

- [ ] **Step 5: Commit (Code)**

```bash
git add Cargo.toml Cargo.lock spikes
git commit -m "Add throwaway language server and Zed extension for coexistence test

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

- [ ] **Step 6: Zustimmung des Nutzers einholen**

Dem Nutzer vor jedem Eingriff in seine Zed-Installation mitteilen, was passiert, und auf ein ausdrückliches Ja warten:

1. `cargo install --path spikes/lsp-coexist` (legt `~/.cargo/bin/lsp-coexist` ab)
2. In Zed: *Install Dev Extension* → `spikes/zed-coexist-ext`
3. Falls nötig: `"languages": { "Typst": { "language_servers": ["tinymist", "bib-spike", "..."] } }` in `~/.config/zed/settings.json`
4. Nach dem Test: Dev-Extension entfernen und `cargo uninstall lsp-coexist`

Ohne Zustimmung hier anhalten und Task 9 als offen melden.

- [ ] **Step 7: Test in Zed durchführen**

Mit Zustimmung: Schritte 1 und 2 ausführen, dann `zed spikes/typst-sample/main.typ` öffnen. Im Log (*zed: open log*) prüfen, dass `tinymist` **und** `bib-spike` gestartet sind; startet `bib-spike` nicht, Schritt 3 (mit bereits erteilter Zustimmung) ausführen und Zed neu starten.

Dann jede Funktion auf `@dwork2006` in Zeile 3 ausprobieren und festhalten, **welcher Server** jeweils sichtbar ist:

| Funktion | Auslösen | Erwartung, falls Zed zusammenführt |
|---|---|---|
| Diagnostics | Datei öffnen | Hinweise „bib-spike sieht @…" neben tinymists Meldung zu `@unbekannt2020` |
| Hover | Maus über `@dwork2006` | tinymists Literaturinfo **und** „bib-spike Hover für `dwork2006`" |
| Go to Definition | F12 | Ziele beider Server (Literaturverzeichnis und Zeile 1) |
| Find All References | Shift+F12 | ein Multibuffer mit beiden Vorkommen, ohne Duplikate |
| Code Actions | Ctrl+. | „bib-spike: Aktion für dwork2006" neben eventuellen tinymist-Aktionen |
| Completion | `@` tippen | tinymists Keys **und** `bibspike2026` |

- [ ] **Step 8: Protokoll schreiben**

`docs/research/14-zed-zwei-language-server.md`:

```markdown
# Zed mit zwei Typst-Language-Servern (Schritt 0b)

Stand: <Datum des Tests>. Zed <Ausgabe von `zed --version`>, tinymist <Version aus dem Zed-Log>.

Aufbau: `spikes/lsp-coexist` (Wegwerf-Server) über die Dev-Extension `spikes/zed-coexist-ext`,
Testdokument `spikes/typst-sample/main.typ`. Settings-Änderung nötig: <ja/nein, welche>.

| Funktion | tinymist sichtbar | bib-spike sichtbar | Ergebnis |
|---|---|---|---|
| Diagnostics | | | |
| Hover | | | |
| Go to Definition | | | |
| Find All References | | | |
| Code Actions | | | |
| Completion | | | |

## Folgen für Spec §9

<Welche geplanten Editor-Funktionen tragen, welche nicht, und was stattdessen.>
```

Die spitzen Klammern und leeren Zellen werden mit den Beobachtungen aus Step 7 gefüllt; das Dokument wird erst mit echten Werten committet.

- [ ] **Step 9: Aufräumen**

Dev-Extension in Zed entfernen, `cargo uninstall lsp-coexist`, eine eventuelle Settings-Änderung rückgängig machen. Dem Nutzer bestätigen, dass der Ausgangszustand wiederhergestellt ist.

- [ ] **Step 10: Commit (Protokoll)**

```bash
git add docs/research/14-zed-zwei-language-server.md
git commit -m "Record Zed behaviour with two Typst language servers

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```

---

### Task 10: Benchmark auswerten und Spec aktualisieren

**Files:**
- Create: `bench/results/<Datum>.md`
- Modify: `bench/README.md`, `docs/superpowers/specs/2026-09-16-bibliothek-kern-typst-design.md` (§7, §9, §17, §18)

**Interfaces:**
- Consumes: Bericht aus Task 8, Korpus aus Task 7, Protokoll aus Task 9
- Produces: festgelegtes Standard-Backend und Kaskaden-Backend für Plan 2

- [ ] **Step 1: Entscheidungsregel festhalten**

An `bench/README.md` anhängen:

```markdown
## Entscheidungsregel (Schritt 0a)

R(b) = gefundene Sätze / alle Sätze über den Korpus, O(b) = korrekte Reihenfolge-Paare / alle Paare.
`mutool` ist nur Referenz (AGPL, nicht als Standard wählbar).

1. **`pdf_oxide` bleibt Standard**, wenn R(pdf_oxide) ≥ max(R(pdfium), R(mutool)) − 0,05
   **und** O(pdf_oxide) ≥ max(O(pdfium), O(mutool)) − 0,05
   **und** bei jedem Dokument, in dem pdf_oxide Zeichen fehlen oder Steuerzeichen/`(cid:`/U+FFFD
   auftreten, `pdf-extract` alle erwarteten Zeichen liefert (die Kaskade trägt).
2. **Sonst pdfium**, wenn es Bedingung 1 mit pdfium an Stelle von pdf_oxide erfüllt.
3. **Sonst anhalten** und die Ergebnisse mit dem Nutzer besprechen.
```

- [ ] **Step 2: Vollständigen Lauf durchführen**

```bash
cargo run -p extract-bench -- fetch
BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib \
  cargo run --release -p extract-bench --features pdfium -- run --out bench/results/$(date +%F).md
```

Expected: Bericht mit 7 Dokumenten × 4 Backends. Gibt es Sätze, die **kein** Backend findet, zurück zu Task 7 Step 4 und die Aussage korrigieren, dann erneut laufen lassen.

- [ ] **Step 3: Regel anwenden**

Aus der Summentabelle R und O je Backend ablesen, die drei Bedingungen der Reihe nach prüfen und das Ergebnis mit den Zahlen unter den Bericht schreiben:

```markdown
## Entscheidung

- R: pdf_oxide …, pdfium …, mutool …, pdf-extract …
- O: pdf_oxide …, pdfium …, mutool …, pdf-extract …
- Dokumente mit Zeichenproblemen bei pdf_oxide: … — von pdf-extract abgedeckt: ja/nein
- **Standard-Backend:** …  **Kaskade bei Verdacht:** …
```

Trifft Regel 3 zu: hier anhalten, Bericht committen und dem Nutzer vorlegen.

- [ ] **Step 4: Spec aktualisieren**

In `docs/superpowers/specs/2026-09-16-bibliothek-kern-typst-design.md`:

- §7 „Kaskade, pro Seite": Standard-Backend und Kaskade gemäß Entscheidung eintragen, Verweis auf `bench/results/<Datum>.md`.
- §9: Absatz zu Zed gemäß `docs/research/14-zed-zwei-language-server.md` von „nicht praktisch getestet" auf das Ergebnis ändern.
- §17: Zeilen „Benchmark (Schritt 0a)" und „Zed mit zwei Language Servern (Schritt 0b)" mit den Kernzahlen ergänzen; die Zeile „Aus Doku und Issues, nicht praktisch getestet" entfernen.
- §18: die Punkte zum Benchmark-Ergebnis und zum Zed-Test streichen.

- [ ] **Step 5: Gesamtprüfung**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS ohne Warnungen. Clippy-Funde beheben, nicht unterdrücken.

- [ ] **Step 6: Commit**

```bash
git add bench/README.md bench/results docs/superpowers/specs/2026-09-16-bibliothek-kern-typst-design.md
git commit -m "Record extraction benchmark decision and update spec

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_016ecDAMwhZVb5J9kdrZD9PE"
```
