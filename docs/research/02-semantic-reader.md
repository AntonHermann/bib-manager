# Semantic Reader Project (Allen AI and partners)

Research 2026-09-14. Collaboration between NLP and HCI researchers (AI2, UC Berkeley,
University of Washington, and others). The result is the Semantic Reader application on
semanticscholar.org plus a series of research prototypes.

Overview: [Semantic Reader Project (arXiv)](https://ar5iv.labs.arxiv.org/html/2303.14334) · [CACM article](https://dl.acm.org/doi/10.1145/3659096) · [Open Research Platform](https://openreader.semanticscholar.org/)

## Concepts from the prototypes

| Prototype | Idea | Relevance for us |
|---|---|---|
| **CiteSee** | Citations in the PDF colored by personal context: already read, saved, self-cited. Paper cards explain how the cited work relates to one's own reading history. Built as a Chrome extension on top of ScholarPhi (~5,000 new lines, ~17,000 lines of TypeScript total). | Coloring by A/B/C tier: cited, unread, missing |
| **CiteRead** | Shows in the margin of the source what later citing papers say **about this exact spot**. Three contributions: selecting important citers, localizing the comment in the source, margin-note interaction. Study with 12 researchers: better understanding and recall than with a plain list of citers. | Opposite direction of "where do I cite this?" |
| **ScholarPhi** | Definitions of terms and symbols shown right at the point of use; prefers the definition shortly before first use; formula explanations with symbol definitions in the margin. | ε and friends in DP formulas |
| **Scim** | While skimming, goal, novelty, method, and result are highlighted in color. | Supporting sources one doesn't read in full |
| **Relatedly** | Reading and cross-referencing related-work paragraphs that overlap across papers. | Overview of a field |
| **Threddy** | While reading, collect passages (mainly from other papers' related work) and organize them into one's own hierarchical "threads." | Outline emerges while reading |
| **Synergi** | Like Threddy, plus LLM-generated hierarchical summaries; AI as scaffolding, judgment stays with the human. | Grouping by chapter |
| **PaperWeaver** | Recommends new papers based on one's own collection. | Gaps in the library |
| **Papeos** | Links paper passages to talk videos. | probably not |
| **ReaderQuizzer** | Just-in-time comprehension questions while reading. | probably not |

Sources: [CiteSee](https://arxiv.org/pdf/2302.07302) · [CiteRead](https://dl.acm.org/doi/fullHtml/10.1145/3490099.3511162) · [Threddy](https://arxiv.org/html/2208.03455) · [Synergi](https://ar5iv.labs.arxiv.org/html/2308.07517) · [Papeos](https://arxiv.org/pdf/2308.15224) · [ReaderQuizzer](https://arxiv.org/pdf/2308.07988) · [Semantic Reader product page](https://www.semanticscholar.org/product/semantic-reader)

## State of the open-source code

Queried via the GitHub API on 2026-09-14.

| Repo | Purpose | Last push | Stars | License | Assessment |
|---|---|---|---|---|---|
| `allenai/papermage` | PDF → document made of layers (symbols, tokens, lines, sentences, paragraphs, sections, bibliography, equations, tables, …; 27 layers), parser + rasterizer + predictors | 2024-11-08 | 803 | Apache-2.0 | README: *"research prototype for EMNLP 2023 … unlikely to be addressing issues / maintaining this on a regular cadence"*, successor announced under AI2's Dolma project. **The parser is pdfplumber** → inherits its ε problem (see `04`). |
| `allenai/pdf-component-library` ("PaperCraft") | React components for PDF readers with overlays, citation cards, thumbnails, notes; built on React-PDF | 2024-02-15 | 93 | Repo has no license file; `ui/library/package.json`: Apache-2.0, package `@allenai/pdf-components` 0.0.1 | dormant |
| `allenai/scholarphi` | interactive PDF reader, basis of CiteSee | 2023-07-19 | 428 | Apache-2.0 | dormant |
| `allenai/s2orc-doc2json` | PDF/LaTeX → JSON (S2ORC format) | 2024-04-11 | 476 | Apache-2.0 | dormant |
| `allenai/olmocr` | PDF → Markdown with a 7B vision-language model; formulas, tables, reading order, strips headers/footers | 2026-03-25 | 19,468 | Apache-2.0 | **active**, but locally requires an NVIDIA GPU ≥ 12 GB VRAM (tested on RTX 4090, L40S, A100, H100), ~30 GB of space. Hosted at Cirrascale, DeepInfra, Parasail for roughly $0.07–0.20 per million tokens. |

The Open Research Platform page still lists PaperMage and PaperCraft as "active," which doesn't match the repos.

Sources: [papermage](https://github.com/allenai/papermage) · [pdf-component-library](https://github.com/allenai/pdf-component-library) · [scholarphi](https://github.com/allenai/scholarphi) · [olmocr](https://github.com/allenai/olmocr) · [olmOCR paper](https://arxiv.org/html/2502.18443)

## Conclusions

1. **Adopt the concepts, not the code.** Especially valuable: PaperMage's layer model (one canonical text, each layer being a set of regions plus boxes), CiteSee coloring, CiteRead margin notes.
2. The layer model unifies note anchors, wording verification, and later citation-marker coloring.
3. CiteSee coloring needs the link "[18]" ↔ 18th bibliography entry. GROBID provides that (see `05`).
