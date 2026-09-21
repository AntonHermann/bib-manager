# Bibliography Manager

A bibliography tool for writing papers in [Typst](https://typst.app/): it keeps a local mirror of
your Zotero libraries, resolves citation keys across them, and checks that what you cite is actually
there. Longer term it is meant to verify quotes against the source PDFs and connect notes to the
exact place in a paper they came from — see [IDEA.md](IDEA.md) for the full picture and
[docs/superpowers/specs/](docs/superpowers/specs/) for the design.

**Status: early.** Steps 0–2 of the design are implemented — PDF text extraction, the database, and
the Zotero sync with its CLI. Quote verification, full-text search, the Typst integration and the
review workflow are designed but not built. What exists works and is tested; there is simply not
much of it yet. See [What is not built yet](#what-is-not-built-yet).

## Requirements

- Rust 1.95 or newer (edition 2024)
- [Zotero](https://www.zotero.org/) 7 or newer, running, with **Settings → Advanced → Allow other
  applications on this computer to communicate with Zotero** enabled. The local API arrived in
  Zotero 7; this has been exercised against 9.0.1.

Zotero is only ever read, over its local API on `127.0.0.1:23119`, using HTTP GET. The tool never
writes to your library.

## Build

```bash
cargo build --release -p bib-cli
```

The binary lands at `target/release/bib`. Put it on your `PATH` or call it by path.

## Quick start

```bash
cd ~/my-paper
bib init          # creates the database and a bib.toml here
bib sync          # reads every Zotero library into the database
bib doctor        # reports what it found
```

`bib init` prints where the database went and what it created:

```
database /home/you/.local/share/bib/bib.db (schema version 1)
created /home/you/my-paper/bib.toml
```

`bib sync` reads your personal library and every group library, and reports what changed:

```
user "My Library": 161 new, 0 changed, 0 retired, 0 reactivated, 119 PDF attachments
group:1234567 "Reading group": 77 new, 0 changed, 0 retired, 0 reactivated, 68 PDF attachments
```

Running it again with nothing changed in Zotero reports zeroes across the board. The sync is always
a full read — Zotero's local API reports `version = 0` for everything and has no `/deleted`
endpoint, so there is nothing to be incremental against. With a few hundred entries it takes well
under a second.

A source that disappears from Zotero is **retired, not deleted**, and lands in the review queue,
because notes and excerpts will depend on it. If it reappears, it is reactivated.

## Commands

| Command | What it does |
|---|---|
| `bib init` | Create or migrate the database, and write a `bib.toml` into the current directory. `--no-project` sets up only the database. |
| `bib sync` | Read all Zotero libraries and update the database. Read-only towards Zotero. |
| `bib doctor` | Check the database, the sync state and the current project. |
| `bib backup` | Write a consistent copy of the database into the backup directory. |

Global `--json` makes any command print exactly one JSON document instead of text, so it can be
piped into `jq`. Failures print `{"error": "..."}`.

**Exit codes:** `0` success · `1` `bib doctor` found warnings or errors · `2` failure.

Note that `bib doctor` exits 1 for perfectly ordinary states — a retired source, a PDF whose file
has moved. That is the point of the command; do not treat it as a build failure.

## Configuration

### Environment

| Variable | Default | Meaning |
|---|---|---|
| `BIB_DATA_DIR` | `$XDG_DATA_HOME/bib`, else `~/.local/share/bib` | Where `bib.db` and `backups/` live |
| `BIB_ZOTERO_URL` | `http://127.0.0.1:23119` | Zotero's local API |

Pointing `BIB_DATA_DIR` at a temporary directory is the safe way to try things out without touching
your real database.

### `bib.toml`

`bib init` writes one into the current directory. It marks the project root — `bib doctor` walks up
from wherever you run it to find the nearest one.

```toml
id = "995e7f18-0432-410b-9182-4d169d556da9"
name = "my-paper"

# Typst entry points; further files follow from #include.
# [[documents]]
# path = "paper.typ"
# kind = "paper"

[zotero]
# Resolution order for citation keys: "user" and/or "group:<id>".
libraries = ["user"]

# [bibliography]
# path = "references.bib"
# managed_by = "zotero"

[ai]
allowed = []
cloud = false
logging = "required"
```

The `[zotero] libraries` list is the part worth understanding. **Citation keys are not unique across
Zotero libraries** — the same paper often sits in both your personal library and a group library. A
key is resolved in the order you list here, and the first library wins. Two sources count as the
same work if their DOIs match, or if their normalized titles and years match. Otherwise the key is a
genuine conflict and goes to the review queue rather than being silently resolved.

Group ids come from `bib sync`'s output (`group:1234567`).

The commented-out `[[documents]]` and `[bibliography]` sections are read but not yet acted on beyond
the `.bib` comparison below; they are there so the format is stable.

## What `bib doctor` reports

| Code | Severity | Meaning |
|---|---|---|
| `database` | info | Database path and schema version |
| `zotero` | info | Local API reachable |
| `zotero_unreachable` | warning | Zotero not answering — the tool falls back to the last synced state |
| `never_synced` | warning | Nothing synced yet |
| `library` | info / warning | Per library: source count and sync age; a warning if the **most recent** sync attempt failed |
| `missing_citation_key` | warning | Sources Zotero has no `citationKey` for — they cannot be cited |
| `missing_pdf_file` | warning | A PDF attachment whose file no longer exists on disk |
| `pdf_without_file` | info | A PDF attachment with no local copy in Zotero at all |
| `retired_sources` | info | Sources that vanished from Zotero |
| `review_open` | warning | Open review-queue entries |
| `project` | info | Project name and root |
| `library_unknown` | warning | `bib.toml` names a library that is not in the database |
| `key_conflict` | warning | One citation key, several genuinely different works |
| `bib_unreadable` | error | The `.bib` file named in `bib.toml` could not be parsed |
| `bib_only` | warning | A key in your `.bib` that Zotero does not have |
| `metadata_drift` | warning | A field differs between your `.bib` and Zotero |

The `.bib` file is **read, never written**, and only in memory.

## What is not built yet

Designed, specified, not implemented:

- Quote verification against the source PDFs
- Full-text search across the library (`bib index`, `bib text`, `bib search`)
- The Typst integration — parsing your `.typ` sources to know which keys you actually cite, which is
  also what makes the third `.bib` case ("key only in Zotero, so the citation will not compile")
  possible
- `bib review` — the command that drains the review queue. Until it exists, a single retired source
  means `bib doctor` exits 1 indefinitely.
- Notes, the citation graph, everything else in [IDEA.md](IDEA.md)

Known gaps in what *is* built are written down in [docs/deferred-work.md](docs/deferred-work.md).

## Repository layout

```
crates/bib-core/     database, Zotero client and sync, citation-key resolution, doctor checks
crates/bib-cli/      the `bib` binary
crates/bib-extract/  PDF text extraction with several backends
tools/extract-bench/ benchmark harness for the extraction backends
docs/research/       measurements and findings from the design phase
docs/superpowers/    the spec and the implementation plans
spikes/              throwaway experiments (Zed extension, Typst sample)
bench/               PDF corpus for the extraction benchmark
```

## Development

```bash
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

All three must pass. Tests never touch your real database or your real Zotero: they use an in-memory
or temporary database and an in-process fake of the Zotero API (`bib_core::zotero::fake`, behind the
`test-support` feature).

The extraction benchmark needs its PDF corpus and a pdfium build; see [bench/README.md](bench/README.md).

## License

MIT or Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).
