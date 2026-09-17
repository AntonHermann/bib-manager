# Decision log (brainstorming)

All points decided or confirmed by the user, 2026-09-14 to 2026-09-16.

| # | Question | Decision | Rationale / context |
|---|---|---|---|
| 1 | Writing tool | **Typst** (paper + Touying slides) | citations in plain text, chapters recognizable as headings |
| 2 | Role of Zotero | **between "replace" and "hybrid"**: Zotero remains the entry point (connector) and mobile companion (tablet sync); the tool is the workspace | fully replacing Zotero too costly, Zotero's data model too limiting |
| 3 | Writing back to Zotero | **read-only for now**, Zotero IDs stored, writing back addable later | typed notes don't need to reach the tablet |
| 4 | Field and size | **computer science**, some medicine; **a few hundred** entries | OpenAlex/Semantic Scholar cover this well; everything feasible locally |
| 5 | Editor | **Zed** | language server + CLI + own UI for visual parts |
| 6 | MCP server | **not firmly planned**; CLI first, MCP later as a wrapper | small advantage over CLI, costs context |
| 7 | LLM execution | **selectable per task**, as much local as possible | iGPU laptop, see `09` |
| 8 | First usable version | **core + Typst integration** (option a) | direct benefit while writing; delivers sentence extraction for later verification |
| 9 | Scope of the first version | core (read Zotero, text layer, excerpts with anchors, CLI) + language server (hover, references, wording verification, insertion); **not GROBID**, but the layer model prepared for it | following the Semantic Reader research |
| 10 | Data storage | **central database** (option b) | consequences: backup, export command, project marker in the repo |
| 11 | Reference project | EHR seminar **only after submission (2026-09-23)**; until then, its own test project with open-access papers | graded paper, AI rules |
| 12 | Responsible AI | **design principle from the start**, see `11` | user's wish |
| 13 | Programming language | **Rust** (approach A: one program `bib`, CLI + `bib lsp`, Zed extension launches it alongside tinymist; no daemon in v1, SQLite WAL) | user prefers Rust; no v1 building block needs Python |
| 14 | License | **MIT OR Apache-2.0** | broad impact for science matters more than copyleft |
| 15 | PDF extraction | **`pdf_oxide` + a custom repair step**, optional `mutool` as an external process, **benchmark as the first planning step** | measurements in `04` |

## Proposed breakdown into subprojects

1. **Core:** data model with robust anchors, Zotero import, text extraction, CLI.
2. **Typst integration:** parser (citations, sentences, chapters), language server.
3. **Search:** hybrid search over full texts, results shown as excerpts.
4. **Reading and notes:** PDF reader with anchors, three note levels, typed notes/links, groups (own UI).
5. **Graph and enrichment:** OpenAlex/Semantic Scholar, citation graph, co-authors, keywords, retraction check, gaps.
6. **Quote verification:** sentence ↔ source passage, status, report (needs 1, 2, 3).
7. **Later:** extraction matrix, canvas, MCP, writing back to Zotero.

First version = 1 + 2.

## State of the design

The design was worked out in a parallel session and exists as a spec: [`2026-09-16-library-core-typst-design.md`](../superpowers/specs/2026-09-16-library-core-typst-design.md).
These research files come from a fork of the brainstorming session and reflect the state before the spec.
