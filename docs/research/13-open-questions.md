# Open questions and unchecked assumptions

**Status: Historical snapshot.** The list below is preserved as recorded during
brainstorming; it is not the current backlog. Current open design questions and
unchecked assumptions are in [spec §18](../superpowers/specs/2026-09-16-library-core-typst-design.md#18-open-questions);
known implementation and coverage gaps are in [deferred work](../deferred-work.md).
See the [ADR register](../decisions/README.md) for architectural decisions.

Reconciled during the documentation review:

- Project configuration, backup/export formats, PDF-less excerpts, sources outside
  Zotero, page hints, text-layer changes, binary delivery, and ambiguous matches
  have design answers in spec §§5–6 and §§8–11. Designed does not mean implemented.
- Zotero API fields and epsilon normalization have evidence in spec §17.
  Zed coexistence and extraction quality were tested in step 0; remaining merging,
  repair, and detector questions are retained in §18.
- The [benchmark](../../bench/results/2026-09-17.md) reports 14 pages for its
  freshly fetched Abadi 2016 PDF. The earlier download's page-count discrepancy
  remains a caveat on that historical measurement, not an unresolved requirement
  to use that old file.
- Library-wide API coverage and local LLM throughput remain unmeasured and are
  carried into §18 for later subprojects.
- Later ideas remain optional: Type1 repair and cloud vision are in §18; the
  reading interface, discourse graph, and metadata lint are in spec §16.

As of 2026-09-16.

## Unchecked, but assumed in the design

- **Zed with two Typst language servers:** hover merging and references fan-out come from docs/issues, not from a practical test with tinymist. Check early in the plan.
- **`pdf_oxide` gap with Type1 encodings:** cause only hypothesized; whether a repair step is possible outside the library (access to font encoding and character codes via the public API?) has not been checked.
- **Reading order/text quality:** agreement between extractors only 64–89%; who is right is open → benchmark with a reference text.
- **Abadi 2016 download** had only 2 pages according to `file`; read results for this PDF with caution.
- **Coverage of OpenAlex/Semantic Scholar** for the actual library not measured.
- **LLM speed** on the laptop only estimated.
- **Zotero local API**: reading annotations, the citation-key field, and attachment paths in Zotero 9.0.1 (Snap) not practically queried.
- **Normalization of `ϵ`/`ε`:** check NFKC behavior for U+03F5 and the math variants precisely before implementation.

## Open design questions

- Project marker in the repo: format and name of the file holding the project ID.
- Backup and export format of the central database.
- How are excerpts without a PDF (source only in Zotero, no attachment) handled?
- Handling of sources not in Zotero (there were some in the seminar).
- Use page numbers in the citation (`supplement`) to narrow the wording search?
- Versioning the text layer: what happens to anchors when a new backend or normalization arrives (re-resolution + review list is proposed)?
- Relationship between the Zed extension and the `bib` program: delivery/download of the binary.
- Ambiguous matches of a quotation within a source (wording occurring more than once).

## Ideas for later (not decided)

- Upstream contribution to `pdf_oxide` for Type1 glyph names.
- Vision model via cloud (olmOCR) only for problematic pages.
- CiteSee coloring and CiteRead margin notes in a dedicated reading UI.
- Discourse-graph model for notes.
- Metadata check modeled on the formal check.
