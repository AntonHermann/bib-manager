# Existing workflow: the EHR seminar as a requirements source

Observed on 2026-09-14 by reading only, from `~/Documents/seminar_ehr_ss26`.
Only structure and tools are recorded here, no content of the paper itself.

**Important:** the project is a graded paper with its own AI rules (due 2026-09-23).
Until then, it will not be read or tested again from this project.
Only after that does it become the reference project for the first version's acceptance.

## Project structure

- Typst paper: `paper.typ`, `template.typ`, `chapters/*.typ` (one file per section), alongside `chapters/*.md` as excerpt collections used before writing.
- Slides: `presentation/slides.typ` (Touying), same `.bib`.
- `ehr_privacy.bib`: a Zotero export, 56 entries. **Must not be rewritten by tools.**
- `literatur/`: 65 PDFs + 65 `pdftotext` full texts (3.2 MB, version-controlled), sorted by triage stage.
- An Obsidian vault in the same folder (plugin `obsidian-pandoc-reference-list`), Pandoc CSL and Zotero JSON exports under `.pandoc/`.
- `Justfile` and `.zed/tasks.json` for commands.

## Tools the tool is meant to replace

| Today | Purpose | In the tool |
|---|---|---|
| `notes/quote_verification.json`: 143 entries `{key, label, quote}`, optional `validated: <session>` | excerpt collection with a chapter label | excerpt (global, anchored) + usage (project, label) |
| `notes/check_quotes.py` (91 lines) | verifies wording against full texts; fixed table key → text file; normalization: NFKC, soft hyphens, line-end hyphenation, unify dashes/quotation marks, whitespace, lowercase | automatic verification for all sources, paths from Zotero, hits with page number |
| `notes/check_slide_quotes.py` (210 lines), `just pres-check-quotes` | chain slide → collection → source; `PARSER:` lines for unrecognized `#quote` forms | diagnostics in the editor, a real Typst parser |
| `notes/quote_inserter.sh` (Zed task, `jq` + `fzf`, writes via `awk` at `ZED_ROW`/`ZED_COLUMN`) | insert an excerpt as `#quote(attribution: <key>)[…]` | completion / code action |
| `just pres-all-used-citations` (`rg`) | keys used in a document | "Find All References" |
| `grep` over `literatur/text/` | full-text search | search index (subproject 3) |
| folders `A_kern` / `B_belege` / `C_rest` + roles (primary source, backward, forward, reserve, discarded) | triage | status and role per source and project |
| `notes/Quellenauswahl.md`: backward references of the primary source, forward search via Semantic Scholar (citation sentences reviewed), co-author checks, self-citations | citation graph by hand | subproject 5 |
| `literatur/zotero_korrekturen.md`: formal check (venue/volume/issue complete? peer-reviewed version instead of arXiv?) | metadata quality | possible metadata check |
| `tool_usage/`, `jsonl2md.sh`, appendix table | documentation of AI use | usage log + appendix generator (see `11`) |

## Documented pitfalls (from `literatur/README.md`)

- Tables broken apart without `-layout` → wrong number in notes.
- ε missing in several DP full texts; quotations with ε taken directly from the PDF instead.
- Math-italic `𝜖` in one book; search needs `ε|𝜖`.
- Some sources not from Zotero (added via cited-by research), four Zotero entries without a PDF, one duplicate.

## Work habits that influence the design

- **Plain text in git** for full texts and the quote collection, so Claude can access them in web sessions → an export command in the CLI is needed even though the data lives centrally.
- Quotations are used **verbatim** with a verification chain; manual confirmation is recorded in the dataset (`validated`).
- Notes on sources also directly in Zotero (e.g. "earlier version of …").
- Zed as editor, tasks instead of extensions.

## Implications for the import (acceptance test)

- The 143 entries have **no context and no position**; anchors must be created by searching in the new text layer.
- This text layer is not the one against which verification was originally done: quotations with ε may now match, others may no longer.
- Entries with `validated` were never found by machine.
- → assign **provenance per entry** (deterministically verified / confirmed by the user, not machine-anchored / unresolved) and put unresolvable anchors into a review list. Without a git diff, the review list is the only safety net.
