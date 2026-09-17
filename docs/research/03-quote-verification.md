# Quote verification: how big is the problem, what exists?

Research 2026-09-14.

## Frequency of citation errors (medicine)

**Jergas & Baethge 2015** (PeerJ), systematic review and meta-analysis across 28 studies:

| Error type | Rate | 95% CI |
|---|---|---|
| major | 11.9% | 8.4–16.6 |
| minor | 11.5% | 8.3–15.7 |
| **total** | **25.4%** | 19.5–32.4 |

- High heterogeneity, but even the lowest overall estimate was 6.7%.
- Indirect citations (quoting at second hand) account for less than a sixth of the problems.

**2025 update** (Research Integrity and Peer Review):

- 16.9% of citations were erroneous (CI 14.1–20.0%), about half of them major: 8.0% (CI 6.4–10.0%).
- Meta-regression: **no improvement over the years.**

This is the strongest argument for built-in quote verification.

Sources: [Jergas & Baethge 2015 (PeerJ)](https://peerj.com/articles/1364/) · [PMC version](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4627914/) · [2025 update (Springer)](https://link.springer.com/article/10.1186/s41073-025-00173-z) · [PMC version of the update](https://pmc.ncbi.nlm.nih.gov/articles/PMC12285159/) · [Methodology critique and recalculation (PubMed)](https://pubmed.ncbi.nlm.nih.gov/28910404/)

## Verification systems

Two different questions are often conflated:

1. **Does the source exist, and are the details correct?** (mainly a problem with LLM-generated text)
2. **Does the source support the claim?** (what we care about)

| System | Question | Approach | Result |
|---|---|---|---|
| **SemanticCite** (Haan 2025) | 2 | Full-text verification, four classes: *supported / partially supported / unsupported / uncertain*; small fine-tuned models | 84% weighted accuracy |
| **CiteGuard** (Choi et al. 2026) | 2 (attribution) | Retrieval-assisted validation of citation attribution | 68% on CiteME, humans 70% |
| **CiteCheck** | 1 | three severity levels: exact / minor metadata errors / fabricated; search + LLM scoring + automatic correction | – |
| **CiteAudit** (Yuan et al. 2026) | 1 | multi-agent verification, benchmark | – |
| Abbonato (2026) | 1 | matching against bibliographic metadata | – |

- Depending on the study, LLMs produce up to 78–90% fabricated citations.
- SemanticCite's four classes fit well with the "score + status overview" idea from `IDEA.md`.
- Small models suffice for classification → locally feasible (see `09`).

Sources: [CiteCheck](https://arxiv.org/html/2605.27700v1) · [CiteAudit](https://arxiv.org/html/2602.23452v3) · [CiteGuard](https://arxiv.org/html/2510.17853v4) · [Cited but Not Verified](https://arxiv.org/html/2605.06635v1) · [Reference Hallucinations in Deep Research Agents](https://arxiv.org/pdf/2604.03173) · [INRA.AI Blog](https://www.inra.ai/blog/citation-accuracy)

## Design implications

- Stage 0 (first version, no LLM): **wording verification** of literal quotations against the text layer. Deterministic.
- Stage 1: find the matching spot in the source (search/embeddings), classify roughly, locally.
- Stage 2: thorough report via LLM, locally in the background or via the cloud.
- Every LLM judgment shows the excerpt and is never automatically treated as "true" (see `11`).
- Page numbers in the citation (`supplement`) narrow the search significantly.
- Empirical statements are more reliably verifiable than paraphrased arguments.
