# Design: Library Core and Typst Integration (Subproject 1 + 2)

**Date:** 2026-09-16
**Status:** Steps 0–2 implemented; remaining steps designed but not implemented. Acceptance of architectural decisions is recorded in the [ADR register](../../decisions/README.md).
**Predecessor:** `IDEA.md`

---

## 1. Goal and Context

The project described in `IDEA.md` is too large for a single spec. This spec describes the **first usable version**: library core plus Typst integration. It is meant to replace the author's home-grown scripts with a coherent tool.

### The existing workflow

Determined from `~/Documents/seminar_ehr_ss26` (EHR privacy seminar, Typst paper plus Touying slides):

| Today | What it does |
|---|---|
| `notes/quote_verification.json` | 143 excerpts: key, chapter label, wording, some marked `validated` |
| `notes/check_quotes.py` | verifies wording against `pdftotext` full texts, with NFKC normalization, key→file mapping maintained by hand |
| `notes/check_slide_quotes.py` | verifies slides against the quote collection |
| `notes/quote_inserter.sh` | Zed task using fzf, inserts `#quote(attribution: <key>)[…]` |
| `just pres-all-used-citations` | lists the keys used in a document |
| `literatur/text/**.txt` | full texts, versioned, searched with grep |
| `literatur/{A_kern,B_belege,C_rest}` | triage tiers as folders |
| `notes/Quellenauswahl.md` | manual citation research (backward/forward, co-author check) |
| `literatur/zotero_korrekturen.md` | formal check of the metadata |

Known weaknesses of this workflow that the tool should fix: tables falling apart, ε disappearing in math fonts, a manually maintained key→file mapping, excerpts without anchors, triage only as folder structure.

### Environment

- Typst 0.15, written in **Zed** (alongside tinymist)
- **Zotero 9.0.1** (Snap) with native citation keys, migrated from Better BibTeX; tablet sync via Zotero
- Library: a few hundred entries, focused on computer science, some medicine
- Laptop: Ryzen 7 PRO 5850U, integrated graphics, 30 GiB RAM, no dedicated GPU

---

## 2. Decisions

This table is a summary, not a second decision register. Linked ADRs are authoritative
for rationale and acceptance status; this spec describes intended behavior, not proof
that every feature is implemented. Rows without an ADR retain their design context
until separately reviewed or recorded.

| Decision | Summary / design context | Record |
|---|---|---|
| **Zotero stays the source of truth**, the tool only reads | Preserve the connector and tablet workflow; retain Zotero IDs for possible later write-back. | [ADR-0001](../../decisions/0001-keep-zotero-as-source-of-truth.md) |
| **All data centrally in SQLite** | Share excerpts across projects; export irreplaceable data when needed (§5). | [ADR-0002](../../decisions/0002-use-central-sqlite-storage.md) |
| **Configuration, by contrast, lives in the repo** (`bib.toml`) | Hand-written, versioned, diffable; policy history remains with the project. | [ADR-0002](../../decisions/0002-use-central-sqlite-storage.md) |
| **Rust, one program, one workspace** | `bib` provides CLI commands and `bib lsp`, launched by a Zed extension alongside tinymist (§4, §9). | [ADR-0007](../../decisions/0007-use-one-rust-program-with-a-companion-language-server.md) |
| **License `MIT OR Apache-2.0`** | An explicit preference: impact in academia takes priority over copyleft. This rules out `mupdf-rs` (AGPL) as a dependency. | Project constraint |
| **Extraction behind an interface, default `pdf_oxide`** | Step 0a selected `pdf_oxide`, with a planned `pdf-extract` fallback on suspicious pages (§7). | [ADR-0003](../../decisions/0003-select-pdf-extraction-backends.md) |
| **No LLM in this version** | Everything needed is deterministic. Still, the groundwork for later LLM features is included (§12). | First-version scope; see [ADR-0006](../../decisions/0006-enforce-responsible-ai-invariants.md) |
| **No daemon** | The language server and CLI access SQLite directly in WAL mode. | [ADR-0002](../../decisions/0002-use-central-sqlite-storage.md) |
| **Anchors separate from content; shared node references** | Wording, context, and position support re-resolution as text layers change; features share `node_type` + `node_id` references (§5). | [ADR-0005](../../decisions/0005-separate-anchors-from-content.md) |
| **Responsible-AI invariants across subprojects** | Separate model output from document writing; preserve provenance, mandatory call logging, and enforced project policy (§12). | [ADR-0006](../../decisions/0006-enforce-responsible-ai-invariants.md) |

---

## 3. Scope

**Included:** Zotero sync (read-only), text extraction with normalization, anchors and excerpts, text-layer corrections, Typst analysis, quote verification, language server, CLI, import of the existing quote collection.

**Not included** (each its own subproject): GROBID and sentence structure, citation markers in the PDF, embeddings and semantic search, citation graph and metadata enrichment, notes and discourse graph, PDF interface, writing back to Zotero, annotations from the tablet, any LLM feature, MCP server, detection of figures and tables.

---

## 4. Architecture

```
bib-core      data model, SQLite, Zotero, normalization, anchors
bib-extract   interface + backends (pdf_oxide, pdf-extract, optional mutool)
bib-typst     Typst parser, project and document analysis
bib-lsp       language server
bib-cli       program `bib`
zed-bib       Zed extension, starts `bib lsp` alongside tinymist
```

```
Zotero (local API) ──► sources, citation keys, PDF paths, tags
PDF ──► extraction ──► text layer (spans, boxes, pages) ──► + corrections ──► canonical text
Excerpt ──► anchor in the canonical text ──► quote verification
*.typ ──► parser ──► citations, chapters, #quote ──► usage in the project
```

---

## 5. Data Model

A single SQLite database at a fixed location (`~/.local/share/bib/bib.db`), WAL mode, versioned migrations, foreign keys enabled.

**Implementation scope:** the table overview below describes the target model, not
the schema already present. Migration 1 creates the library/source/attachment,
tag/collection, and sync bookkeeping tables needed by steps 1–2, plus `review_queue`
and the empty `llm_call` table required by §12. Anchors, excerpts, text layers,
projects, documents, citations, usages, and the remaining target tables arrive
with the features that use them, through later migrations.

| Table | Contents |
|---|---|
| `source` | source from Zotero: library + item key (required), citation key, metadata, tags, status (active/retired) |
| `attachment` | PDF: path, checksum, page count |
| `text_layer` | text layer of an attachment: backend per page, backend version, normalization version, canonical text (zstd), quality flags |
| `page_geometry` | per page: size, rotation, coordinate system, spans as a compact block with an offset index |
| `text_patch` | manual correction: target as an anchor, replacement text, rationale, provenance, status (active/obsolete) |
| `anchor` | **its own table.** Kind *text range* (wording, 32 characters of context on each side, position) or *page range* (page, rectangle). State *anchored*, *ambiguous*, or *unanchored* |
| `excerpt` | verbatim excerpt at an anchor, with verification status and provenance |
| `project` | path, snapshot of `bib.toml`, status |
| `document` | `.typ` file of a project, kind paper or slides |
| `chapter` | heading tree of a document |
| `citation` | citation in the document: position, key, form, optionally an excerpt, optionally "dynamic" |
| `usage` | use of an excerpt in a project: chapter or label, role |
| `llm_call` | usage log, created empty in this version |
| `review_queue` | everything a human needs to look at |

### Principles

**Anchors separate from content.** Excerpts, corrections, and later notes or figures all point at the same anchor type. Nodes are referenced uniformly as `node_type` plus `node_id`; this convention applies everywhere from now on.

**Anchors follow the W3C model.** Wording plus context plus position. Resolution proceeds exact → normalized → fuzzy (at most 5% deviation), with context resolving ambiguity. If the text layer changes, all anchors are re-resolved; discrepancies go to the review queue.

**Anchors without a text layer.** If a source doesn't (yet) have a PDF, an excerpt can still exist: its anchor is *unanchored*, and it counts as not verifiable. As soon as a PDF is available, it is anchored automatically.

**Ambiguous matches.** For *verifying* a wording, one match is enough. When *creating* an excerpt, if there are multiple matches, context decides first, then a page reference from the document (section 8); otherwise the anchor stays *ambiguous* and lands in the review queue with all candidates.

**Excerpt and usage are separate.** The excerpt belongs to the source and is usable across projects; the chapter label belongs to the usage within the project.

**Provenance on every entry:** *by you*, *deterministically verified*, *imported, provenance unknown*, *LLM assessment* (referencing `llm_call`), *confirmed by you*.

**Data that cannot be regenerated** are excerpts, anchors, usages, corrections, review-queue decisions, and later notes. Everything else can be rebuilt from Zotero and the PDFs.

**Backup and export:**

- **Backup** is a complete, consistent copy of the database via `VACUUM INTO`: automatic before every migration and on command (`bib backup`), retaining recent snapshots.
- **Export** covers only the data that cannot be regenerated, as **JSON Lines per table** with stable IDs. That is diffable, readable, and independent of the database schema; `bib import` can restore from it. Project-specific exports (for instance in the format of `quote_verification.json`) are additionally configurable via `[export]` in `bib.toml`.

### Preview: notes (subproject 4)

So this fits later without rework, the shape is fixed here:

```
note ──< note_target ──► anchor | source | chapter | project | tag | note
link (from, to, type: supports · contradicts · answers · refines)
```

An excerpt is a verbatim, verifiable passage; a note is in the author's own words, typeable and linkable. Zotero highlights will later become `anchor` plus `excerpt` with a Zotero ID, comments become `note` at the same anchor. This version only creates `anchor` and the node convention.

---

## 6. Zotero Sync

- **Local API** at `127.0.0.1:23119/api/users/0/…`, offline, no rate limit, **read-only**. Requirement: Zotero is running and the "allow other applications" option is enabled.
- **Full sync instead of incremental.** The local API returns `version = 0` and `Last-Modified-Version: 0` for every entry, and there is no `/deleted` endpoint (measured, section 17). The sync therefore fetches all entries page by page and compares against the database: new, changed (via `dateModified` and a checksum of the fields), gone. With a few hundred entries, this is cheap.
- **Endpoints:** sources from `/items/top` (skipping notes, standalone attachments and annotations), PDFs from `/items?itemType=attachment` joined via `parentItem`, collections from `/collections`, groups from `/users/0/groups`; at most 100 items per page, `Total-Results` gives the total (`/items` alone returns every item type: 603 rows against 168 top-level items).
- **Fallback:** a read-only copy of `zotero.sqlite` when Zotero isn't running. Without either, the tool works from the last known state, whose age is shown everywhere. Deferred (plan 2): until then, the tool works from the last synced state.
- **Imported:** metadata, native citation key (field `citationKey`), tags, collections, attachments. The file path is not in the attachment data itself but in the `enclosure` link as a `file://` URL (alternatively `/items/<key>/file/view/url`). Attachments with `contentType = application/pdf` are considered, regardless of `linkMode`.
- **Deleted, merged, or renamed entries** — recognized by being missing from the full sync or by their key having changed — are retired, not deleted, and land in the review queue because excerpts depend on them.
- **Annotations** are readable via the local API (`annotationText`, `annotationComment`, `annotationPosition`, `annotationPageLabel`, …), but are only imported starting in subproject 4.

**Zotero is the sole source of sources.** Every `source` originates from Zotero. Sources that exist only in a `.bib` file are not created; the tool prompts the user to add them in Zotero (one click with the connector). Third-party `.bib` files can, if needed, be imported into a personal Zotero library.

**Multiple libraries.** Shared literature goes through Zotero group libraries; the local API serves them (`/api/users/0/groups` lists them, `/api/groups/<id>/items` returns entries, attachments, and annotations — measured, section 17). It follows that:

- A source is identified by the pair **library + item key**, not by the item key alone. The sync runs across all libraries.
- The local user library is stored as `kind = "user"`, `zotero_id = 0`, matching the local API's `/users/0` address. Here `0` is the local-user convention, not a discovered Zotero account ID. A group is stored as `kind = "group"` with its actual group ID.
- **Citation keys are not unique across libraries.** In the measured collection, 17 keys occur in more than one library, usually the same paper in both the personal and a group library. Which libraries a project uses, and in what order, is therefore set in `bib.toml` (`[zotero] libraries`). A key is resolved in that order.
- **Collision within a project's libraries:** if the DOI matches, or the title and year match, it counts as the same work, and the first library wins silently. Otherwise, review queue.
- Entries **without a citation key** (which occurs in group libraries) are imported but not citable; `bib doctor` lists them.

**The `.bib` file is read, never written** (using the `biblatex` library, the same as in Typst), and only in memory — a `.bib` file parses in milliseconds, so it needs no table of its own. This produces three kinds of messages: key only in the `.bib` file ("not in Zotero, please add it there"), key only in Zotero (export is stale, citation won't compile), metadata that has drifted apart.

---

## 7. Extraction and Normalization

### Interface

A backend supplies, per page, **either** spans (text, box, font) in reading order along with page size and rotation, **or** plain text only. Text is required, geometry is optional: `pdf-extract` only returns text through its public API. Pages without geometry carry the quality flag *no geometry*; anchors there resolve via text only, and "open PDF at this location" jumps only to the page. Nothing in this version renders boxes, so that is sufficient.

The core builds the canonical text from the pages and, where available, the mapping from text range → page and box. Spans instead of individual characters keep the data volume small; characters can be reloaded per page when needed.

**One coordinate system for all backends:** points, origin top-left, y grows downward, relative to the MediaBox. Each backend converts on its own (`pdf_oxide` and pdfium return PDF coordinates with the origin at bottom-left, `mutool` already uses top-left). A test verifies that the same text location lands at the same place, to within a few points, across different backends.

### Cascade, per page

1. The default is **`pdf_oxide`** (MIT/Apache, pure Rust, rich data), confirmed by the benchmark from step 0a (decision rule 1, `bench/results/2026-09-17.md`).
2. A **suspicion detector** evaluates: control characters or `(cid:…)`, fonts without a Unicode mapping, math fonts like CMMI without a single Greek character, no text at all.
3. If flagged, **`pdf-extract`** (MIT) runs over the same page; the better result is stored along with which backend produced it. In the benchmark, `pdf-extract` delivered all expected characters in both documents where `pdf_oxide` was missing ε.
4. **`mutool`**, if installed, is the third stage and the benchmark's yardstick.

Recording per page which backend won makes it possible, later, to selectively replace individual broken pages with an OCR or vision model.

A dedicated repair step for missing Unicode mappings (glyph names from embedded Type1 fonts) would be desirable and would make sense as a contribution to `pdf_oxide`, but it is **not a requirement** of this version.

### Corrections

Manual corrections are **not another stage of the cascade** but a stored layer on top: extraction, then corrections applied in a fixed order, yields the canonical text. On re-extraction, corrections are found again via their anchor. Three cases: reapplied; marked *obsolete* if the new backend now gets the spot right; review queue if the spot can't be found.

### Normalization

Two levels with an offset mapping. Raw is what was extracted; the **comparison form** applies NFKC (turning `ϵ`, `𝜖` into `ε` and `ﬀ` into `ff`), removes soft hyphens, rejoins line-end hyphenation, unifies dashes and quotation marks, collapses whitespace, and ignores case. Searches run against the comparison form; storage and display use raw.

### Quality flags

Per page: unmappable characters, suspicious fonts, no text. They appear in the language server's diagnostics, so that "wording not found" becomes an actionable hint.

**This version does not detect tables.** They appear as text in reading order, which can be unusable for multi-column tables.

---

## 8. Typst Analysis

The parser is `typst-syntax`, the same one used by Typst and tinymist. Files are **parsed, not compiled**.

Recognized: `@key`; `#cite(<key>, form:, supplement:)`; `#quote(attribution: <key>)[…]`; `#quote(block: true, attribution: [@key])[…]`; headings and labels; `#include` and `#import`; `#bibliography("…")` including Typst's path rules; the paragraph surrounding a citation.

For the wording of a `#quote`, the plain text is collected, escapes are resolved, and markup is discarded.

**Page references** from `supplement` (such as `[S. 12]`, `[p. 12–13]`) serve only as a **hint**, never as a constraint: the named page is searched first. Printed page numbers often diverge from the PDF's page index, so matching is done against the PDF's page labels where available. If the wording is found only on a different page, verification still passes, with a note about the mismatched page reference.

**Project:** a directory with `bib.toml`. Entry points are listed there; further files follow from `#include`. In the editor, only the changed file is re-parsed; cross-file information comes from the database.

**Limit:** dynamically generated citations (in the example project, `#cite(label, form: "prose")` inside a helper function) cannot be resolved without compiling. They are marked as *dynamic*, not as errors. Optionally, `typst query <file> "cite"` compares the set of compiled citations against the parsed set and reports the difference.

---

## 9. Language Server and Zed

Tested in step 0b (Zed 1.20.2, tinymist 0.15.8, `docs/research/14-zed-two-language-servers.md`): Zed starts a second Typst language server next to tinymist without any settings change. Diagnostics, hover and completion of both servers are merged (the second server's entries appeared first). Code actions, definition and references of the second server work; whether Zed merges them with tinymist's results is untested, because tinymist returned nothing at any position tried. At `@key` citations tinymist offers no hover, definition, references or completion, so the editor features below do not compete with tinymist there (caveat: tested in a single file without a pinned main file).

**Diagnostics:** wording doesn't match (with page quality as the reason); unknown key; key missing from the `.bib` file; metadata diverges; dynamic citation (info); optionally: cited without an excerpt. Severity comes from `bib.toml`. The unknown-key diagnostic can be turned off, since tinymist reports something similar.

**Hover** on a key or attribution: metadata, count and verification status of excerpts, tags, whether a PDF exists, age of the Zotero snapshot.

**Completion:** after `@`, the keys, sorted by usage in the project; inside `#quote(attribution: <key>)[`, the excerpts of that source. The latter replaces `quote_inserter.sh`.

**Actions:** "adopt this wording as a text-layer correction" (needs no input field, the text is already in the document); "mark excerpt as confirmed by me"; "save as excerpt" with a chapter label taken from the surrounding heading; "open PDF at this location" via `zotero://open-pdf/…`.

**Navigation:** references on `@key` across paper and slides (multibuffer, replaces `pres-all-used-citations`); go-to-definition opens the source's extracted text from the cache at the cited location; outline built from the headings.

**Internals:** the server holds the database connection, watches `.bib` and `bib.toml`, triggers a sync on open, does heavy work in the background, and reports progress via `$/progress`. The Zed extension only registers `bib lsp` for Typst.

**Shipping the program:** in this version, the extension looks for `bib` on `PATH` or at a path configured in Zed's settings; it is installed via `cargo install`. If the program is missing, the extension shows a hint with the install command. Later, the extension will download a matching binary from GitHub releases, as many Zed extensions do for their language servers.

---

## 10. CLI

`bib init`, `sync`, `index`, `check`, `cites`, `search`, `text`, `excerpt`, `patch`, `review`, `export`, `import`, `backup`, `doctor`.

`--json` throughout, for scripts, git hooks, and Claude Code, plus sensible exit codes. `bib check` behaves like the existing script: silent on success, an error code on a real failure, `-v` also shows passes, output as `file:line` with the nearest candidate.

### Configuration

`~/.config/bib/config.toml` for defaults, `bib.toml` in the project takes precedence and is versioned. Credentials live in neither file. Environment: `BIB_DATA_DIR` overrides the data directory (default `$XDG_DATA_HOME/bib` or `~/.local/share/bib`), `BIB_ZOTERO_URL` the Zotero address (default `http://127.0.0.1:23119`).

```toml
id = "0193f2a1-…"
name = "EHR Privacy Seminar"

[[documents]]
path = "paper.typ"
kind = "paper"

[[documents]]
path = "presentation/slides.typ"
kind = "slides"

[zotero]
libraries = ["user", "group:6573630"]   # resolution order for citation keys

[bibliography]
path = "ehr_privacy.bib"
managed_by = "zotero"

[export]
excerpts = "notes/quote_verification.json"

[diagnostics]
unverified_quote = "error"

[ai]
allowed = ["retrieval", "verification"]
cloud = false
logging = "required"
```

The file is mandatory and takes precedence over the database. If the same ID shows up at two paths, the review queue asks whether it was moved or copied.

**Implementation scope:** steps 1–2 read `bib.toml` from disk; they do not yet
persist a `project` table or detect one project ID at multiple paths. Those parts
are scheduled with the Typst parser and tracked in
[deferred work](../../deferred-work.md#project-persistence-and-duplicate-project-ids).

---

## 11. Import and Acceptance

For each entry from `quote_verification.json`: resolve the key; search for the wording in the text layer (exact, normalized, fuzzy); build an anchor with context from that, which the JSON file doesn't have. Classification: source without a PDF → excerpt with an *unanchored* anchor, not verifiable; key not in Zotero → review queue with "add to Zotero, then run `bib import` again" (the import is idempotent, a second run doesn't create duplicates); found → *deterministically verified*; multiple matches → review queue with candidates; not found but marked `validated` → *confirmed by you, not machine-anchored*; not found and not marked `validated` → review queue. The label becomes the usage, with a reference to the chapter when a matching heading exists. Existing `#quote` locations are linked to the excerpts via their wording.

At the end, a report with counts per category. **Nothing is silently discarded.** Optionally, `bib import triage --from-dirs literatur/` imports the `A_kern`, `B_belege`, `C_rest` tiers as tags.

**Acceptance:** the seminar project as a read-only copy, **only after 2026-09-23** (an ongoing graded paper with its own AI rules). The oracle is `check_quotes.py`: `bib check` must wave through the same citations and flag the same ones. Until then, development happens against a separate test project using freely accessible papers.

---

## 12. Responsible AI Use

Binding principles for all subprojects:

1. **No model-generated text in the document.** LLM features research, organize, and verify. Only wording from sources ever goes into a `.typ` file. There is no code path that bypasses this.
2. **Provenance on every entry** (see section 5). An LLM judgment is never automatically treated as true; it shows the excerpt and waits for confirmation.
3. **No fabricated sources.** Suggestions come exclusively from the user's own library, always with an anchored location.
4. **Reproducibility:** for every LLM call, the model, version, parameters, prompt, and input checksum are stored, along with the checksum of the `[ai]` policy in effect.
5. **Automate the documentation obligation:** a complete log with tool, version, date, URL, prompt, result, and type of use; from this, an appendix table is generated deterministically, not written by hand.
6. **Policy per project** in the `[ai]` section of `bib.toml`, enforced by the tool.
7. **Make data flow visible:** for cloud calls, record what data left the device, regardless of authorship or origin. This includes prompts, file contents, extracted passages, metadata, and any other transmitted context—not just text written by the user.
8. **Don't automate away reading.** Hints like "cited but never read" are welcome; skimming aids remain reading aids.

**Implemented in this version:** provenance fields (2), the `llm_call` table (empty, but mandatory for later features), the `[ai]` section in `bib.toml`. Principle 1 holds trivially, since no LLM is included.

---

## 13. Error Handling

Principle: nothing is silently discarded, everything unclear goes to the review queue, `bib review` is the only way out.

**Review queue deduplication:** only open items sharing a `dedupe_key` are
deduplicated. Once an item is resolved, a recurrence of the same problem can
create a new item; the resolved record remains part of the history.

| Problem | Behavior |
|---|---|
| Zotero isn't running | last known state, age shown |
| PDF missing | source is citable, verification reports "no PDF" |
| Extraction fails on a page | page flagged, rest remains usable |
| Backend crashes (`pdf-extract` aborts hard on corrupt files) | run in isolation, failure recorded on the attachment |
| Error in the language server | no crash: background work, error surfaced as a diagnostic, timeouts on Zotero requests |

Database: WAL for concurrent access from the CLI and the server, versioned migrations, automatic backup before every migration.

---

## 14. Tests

- **Unit tests** for normalization (including idempotence), anchor search against deliberately altered texts, the Typst parser across all four citation forms including escapes and the dynamic case.
- **Sample PDFs**, not in the repo but downloaded via a script with fixed checksums: old LaTeX without Unicode mapping, a modern two-column paper, a scanned document, tables.
- **Backend benchmark** as a standalone tool, using arXiv LaTeX sources as ground truth; repeatable after updates.
- **Language server tests** against a sample project using a scripted client.
- **Acceptance** against the seminar project (see section 11).

Development is test-driven.

---

## 15. Implementation Order

| Step | Result | Usable |
|---|---|---|
| 0a | Backend benchmark | decides section 7 |
| 0b | Zed test: minimal language server alongside tinymist, checks hover, references, definition, diagnostics | confirms or corrects section 9 |
| 1 | Data model, migrations | `bib init`, `bib doctor` |
| 2 | Zotero sync, reading `.bib` | `bib sync` |
| 3 | Extraction, normalization | `bib index`, `bib text`, `bib search` |
| 4 | Anchors, excerpts, corrections | `bib excerpt`, `bib review`, `bib patch` |
| 5 | Typst parser | `bib cites`, `bib check` |
| 6 | Import | 143 excerpts imported |
| 7 | Language server | work in Zed |
| 8 | Zed extension, export | first version done |

Each step is usable on its own; from step 3 onward, the tool already replaces parts of daily use.

---

## 16. Future Work

**Figures, diagrams, and tables.** Largely additive: `block` (layout regions), `media` (images), derived artifacts (table cells, plot values). Already provided for: anchors with a page range, usage attached to the anchor instead of a text excerpt, page geometry with size, rotation, and coordinate system. A different serialization of tables shifts offsets — versioned text layers and anchors based on wording plus context guard against that.

**Reading interface.** A dedicated PDF reader with CiteSee-style coloring (citations marked by *in library / triage tier / self-cited / unread / missing*), CiteRead-style margin notes (what citing papers say about a passage), and a display of anchors, excerpts, and notes. Requires citation markers from GROBID and citation contexts from subproject 5.

**Third-party or shared `.bib` without Zotero.** Deliberately not modeled. If it becomes necessary (for instance, for joint papers with a frequently updated, hand-maintained `.bib`), it will come back as its own topic; until then, the way forward is a personal Zotero library or a group library.

**Public bibliography as an interactive website.** Own papers, selected notes, and links as a statically generated, searchable page with a graph view. Data-model requirements: per-note and per-link visibility (default *private*, publication only explicit), and no full texts or excerpts beyond fair-use quotation limits on export. Requires subproject 4 (notes) and 5 (graph).

**Further subprojects:** notes and discourse graph (4); hybrid search combining BM25 and embeddings, SPECTER2 at the paper level (3); citation graph, citation contexts of citing papers, co-authors, retraction check, metadata lint (5); content-level quote verification via LLM, extraction matrix (6); writing back to Zotero, MCP server, capturing a project's Claude sessions (7).

---

## 17. Research Basis

Own measurements (September 2026):

| Check | Result |
|---|---|
| ε in Dwork 2006 (old LaTeX without Unicode mapping) | pdftotext 0, pdfplumber 0, pdfium 0, `pdf_oxide` 0 (character drops out), **`mupdf-rs` 28, `pdf-extract` 28** |
| ε in Abadi 2016 (modern) | all backends 92. *Caveat:* `file` reported 2 pages for the download; at roughly 11,500 extracted words, `file`'s page count is more likely wrong. Not re-checked; the benchmark uses freshly downloaded files. |
| NFKC on ε variants | `ϵ` (U+03F5), `𝜖` (U+1D716), `𝜀` (U+1D700) all become `ε` (U+03B5), `ﬀ` becomes `ff` |
| Agreement between extractors | 64–89% on random 8-word excerpts; without a reference, no statement about correctness is possible → benchmark needed |
| Semantic Scholar on the seminar's main source | 79 citing papers, 41 with a citation sentence, 3 "influential", **0 with a citation intent** → intents are too sparse for subproject 5 |
| Zotero 9.0.1 (Snap), local API, queried read-only | reachable, 168 top-level entries; `citationKey` populated as a native field; **`version` is 0 everywhere, `Last-Modified-Version: 0`, no `/deleted` endpoint** → no incremental sync; PDF path via the `enclosure` link (`file://…/Zotero/storage/<key>/…`); 274 annotations with text, comment, position, and page label readable; 14 collections. Writing only works via the web API. |
| Group libraries via the local API | `/api/users/0/groups` returns 3 groups, `/api/groups/<id>/items` works including attachments and annotations (81 + 41 top-level entries, one group empty). **17 citation keys occur in more than one library** (16 between the personal library and a group library). One group entry without a citation key. |
| Benchmark (step 0a): 7 documents, 21 sentences, 7 order pairs, `bench/results/2026-09-17.md` | Sentences: `pdf_oxide` 20/21, `pdf-extract` 21/21, `mutool` 21/21, pdfium 18/21; reading order 7/7 everywhere. `pdf_oxide` is missing ε in 2 documents (Dwork 2006, Shokri 2017), `pdf-extract` delivers it there; `mutool` 191 × U+FFFD, pdfium 1128 control characters (in the recounted documents devlin2019 and abadi2016, mostly U+0002 as a line-end hyphenation marker, 527 of the 573 counted there). → **default `pdf_oxide`, cascade `pdf-extract`** (rule 1) |
| Zed with two language servers (step 0b), `docs/research/14-zed-two-language-servers.md` | Both servers start without settings change; diagnostics, hover and completion are merged; at `@key` tinymist provides no hover, definition, references or completion; merging of definition/references untested (tinymist had no results) |
| Zotero sync smoke test (plan 2), `docs/research/15-zotero-sync-smoke-test.md` | 4 libraries (3 groups), 161/41/77/0 active sources; 17 citation keys occur in more than one library, 0 of those are different-work conflicts; first sync 0.34 s |

External sources that shaped the design: Jergas & Baethge (quote error rate around 25%, a 2025 update showed no improvement) as the rationale for verification; W3C Web Annotation and Hypothesis for robust anchors; PaperMage for the layer model (a research prototype, unmaintained since 11/2024, uses pdfplumber and would have inherited the ε problem); CiteSee, CiteRead, Scim, ScholarPhi, Threddy, Synergi as sources of ideas for later subprojects; SemanticCite for the four verification stages; Discourse Graphs for the note model.

---

## 18. Open Questions

Current unresolved design questions and assumptions live here. Known implementation
and coverage gaps live in [deferred work](../../deferred-work.md); the
[brainstorming question list](../../research/13-open-questions.md) is historical.
Future-subproject questions below are not commitments for this version.

- Does Zed open an external link (`zotero://…`) sent by the language server via `window/showDocument`? If not, the CLI takes over.
- Does Zed merge definition, references, and code actions when both servers return results? Step 0b confirmed coexistence and merged diagnostics, hover, and completion, but tinymist returned no competing results for these other requests (§9).
- Does the local Zotero API support `sort=dateModified`? The query returned entries from June as "newest," even though entries had been added in September. Irrelevant for the full sync, but worth clarifying for a later optimization.
- A contribution to `pdf_oxide` for glyph names from embedded Type1 fonts is desirable, not planned. The cause of the loss and whether its public API exposes enough font encoding and character-code data for an external repair remain unverified; the successful fallback does not establish either.
- The suspicion detector from §7 step 2 only detects character-level signals (control characters, `(cid:…)`, no text). The one sentence `pdf_oxide` missed in the benchmark (20/21: vaswani2017, heading "Abstract" mid-paragraph instead of before it) is a reading-order error, which it cannot detect. Whether the separate font heuristic (math font like CMMI without a single Greek character) fires on the pages where `pdf_oxide` silently loses ε (dwork2006, shokri2017: 0 control characters, 0 `(cid:`, 0 U+FFFD) is unmeasured. Resolve this when implementing the cascade; the completed steps 1–2 did not settle it. See the [benchmark results](../../../bench/results/2026-09-17.md).
- For rotated pages, should spans be reported in rotated or unrotated page space? The existing corpus cannot settle this convention; the missing offset/CropBox/rotation coverage is tracked in [deferred work](../../deferred-work.md#extraction-coordinate-coverage).
- Before graph/enrichment work, measure OpenAlex and Semantic Scholar coverage of the actual library. The research sampled APIs, not library-wide coverage.
- Before selecting local LLMs for a later subproject, measure throughput on the target laptop; the [hardware research](../../research/09-hardware-local-llms.md) contains estimates, not measurements.
- Whether to use a cloud vision model (for example olmOCR) only for problematic pages remains an optional later idea, not a selected extraction stage. Any such design must obey §12 and the project's `[ai]` policy.
