# Open questions and unchecked assumptions

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
