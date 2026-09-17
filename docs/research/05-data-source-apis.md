# Data sources and APIs

Research 2026-09-14.

## Semantic Scholar Academic Graph API

**Own test** using the EHR seminar's primary source
(Jonnagaddala & Wong 2025, *Privacy preserving strategies for electronic health records in the era of large language models*):

```
GET /graph/v1/paper/search/match?query=<title>&fields=title,year,citationCount
→ paperId 3aadb2b130320424dc5b11711715760c619fdb8d, year 2025, citationCount 79

GET /graph/v1/paper/3aadb2b1.../citations?fields=title,contexts,intents,isInfluential&limit=100
```

| Field | Result |
|---|---|
| citing papers | 79 |
| with `contexts` (the sentence in which the citation occurs) | **41** |
| with `intents` (Background/Method/Result) | **0** |
| with `isInfluential` | 3 |

- Without an API key, the first attempt returned `429 Too Many Requests`; it worked after a short pause.
- The citation sentences are exactly what was manually reviewed for the forward search in the seminar → automatable.
- According to Semantic Scholar, intents exist only for papers with full-text access. A sample from CASRAI (5 × 100 citations) found intents in 0–23% of cases, contexts in 4–84%. **Don't treat as a complete annotation layer.**
- `isInfluential` is deliberately rare (an ML model based on the number and context of citations).
- Static SPECTER2 embeddings are available via the paper endpoints.

Sources: [Semantic Scholar: Citation Intent](https://www.semanticscholar.org/faq/citation-intent) · [CASRAI: measured rate limits and data density](https://casrai.org/guides/semantic-scholar-api) · [Semantic Scholar Open Data Platform](https://arxiv.org/pdf/2301.10140) · [API key form](https://www.semanticscholar.org/product/api#api-key-form)

## OpenAlex

- Fully open index, snapshot from 2024-06-30: 256,997,006 works (1400–2024), including datasets, peer-review reports, books.
- Pricing model since February 2026: free API keys with a daily quota; **single lookups by ID or DOI remain free**.
- Complete snapshots can be downloaded.
- Known gaps: coverage of individual countries (e.g. China) and metadata quality across languages.

Assessment: for the user's library (computer science, some medicine, a few hundred entries with DOI/arXiv), OpenAlex and Semantic Scholar together cover the citation graph well. Not verified against the actual library.

Sources: [CASRAI: OpenAlex API](https://casrai.org/guides/openalex-api) · [IntuitionLabs: OpenAlex vs Semantic Scholar vs PubMed](https://intuitionlabs.ai/articles/openalex-semantic-scholar-pubmed-comparison) · [IntuitionLabs: Research Paper APIs 2026](https://intuitionlabs.ai/articles/research-paper-apis-scientific-literature) · [OpenAlex (Wikipedia)](https://en.wikipedia.org/wiki/OpenAlex) · [Linguistic coverage](https://arxiv.org/pdf/2409.10633) · [China coverage](https://arxiv.org/pdf/2507.19302)

## S2ORC

- 81.1 million English-language papers with metadata, abstracts, resolved references; **structured full text for 8.1 million open-access papers**.
- Full text annotated with inline mentions of citations, figures, tables, linked to the paper objects.
- Accessible via the Semantic Scholar API (Datasets) since February 2023.

Sources: [S2ORC (ACL 2020)](https://aclanthology.org/2020.acl-main.447/) · [allenai/s2orc](https://github.com/allenai/s2orc) · [Hugging Face](https://huggingface.co/datasets/allenai/s2orc)

## Embeddings: SPECTER2

- Citation-trained document embeddings (built on SciBERT), 9 tasks, 23 fields.
- Adapters for classification, regression, proximity, and ad-hoc search.
- Operates on **title + abstract**, i.e. whole-paper level → good for "similar papers," less so for passages.
- Hybrid search: SPECTER2 + BM25, merged with reciprocal rank fusion.
- For passage-level search, a general passage-embedding model is better; both can be combined.

Sources: [AI2 Blog: SPECTER2](https://allenai.org/blog/specter2-adapting-scientific-document-embeddings-to-multiple-fields-and-task-formats-c95686c06567) · [allenai/specter](https://github.com/allenai/specter) · [ISLE (hybrid search)](https://arxiv.org/pdf/2512.12760)

## PaperQA2 (FutureHouse / Edison Scientific)

- Agentic RAG over local PDF collections.
- Three search strategies: full text with **tantivy** (Rust), semantic search with metadata-aware embeddings, LLM reranking + "retrieval-augmented contextual summarization."
- Enriches chunks with citation counts (Semantic Scholar), venue, authors, and **retraction status** (Crossref, Unpaywall).
- Default: OpenAI embeddings + NumPy vector store, swappable.

Sources: [paper-qa](https://github.com/future-house/paper-qa) · [Edison Docs](https://docs.edisonscientific.com/paperqa) · [PaperQA paper](https://arxiv.org/pdf/2312.07559) · [Starlog analysis](https://starlog.is/articles/data-knowledge/future-house-paper-qa/)

## GROBID

- Active (`grobidOrg/grobid`, push 2026-09-13, Apache-2.0, 5.1k stars). Moved from `kermitt2/grobid`.
- Structures full text: paragraphs, headings, **reference callouts** (`ref`), figures, tables, reference list broken down into fields.
- `teiCoordinates` parameter: coordinates for selected structures; `segmentSentences`: sentences (`s`) with boxes.
- Coordinates in the PDF coordinate system (origin top-left).
- Uses CRF (a Wapiti fork) by default, runs well on CPU; DeLFT deep-learning models optional, less scalable without a GPU.
- Known issue: partially incomplete coordinates for sentence elements (#811).
- Already used at scale (CORE: 34 million documents).

Use in the project: citation marker "[18]" ↔ reference entry (basis for CiteSee coloring), sentence boundaries, fallback for reference lists without a DOI. **Not in the first version.**

Sources: [GROBID: Coordinates in PDF](https://grobid.readthedocs.io/en/latest/Coordinates-in-PDF/) · [GROBID REST API](https://grobid.readthedocs.io/en/latest/Grobid-service/) · [How GROBID works](https://grobid.readthedocs.io/en/latest/Principles/) · [Issue #811](https://github.com/kermitt2/grobid/issues/811) · [CORE + GROBID](https://blog.core.ac.uk/2023/07/17/core-grobid-structured-text-from-34-million-scientific-documents-and-counting/)

## Crossref / Retraction Watch

See `01-tool-landscape.md`, section "Retracted papers," and `06-zotero.md`.
