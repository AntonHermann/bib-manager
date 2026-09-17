# Tool landscape: what already exists

Research 2026-09-14. Related points from `IDEA.md` are noted at the end of each section.

## Citation graphs and literature discovery

| Tool | Distinguishing feature |
|---|---|
| **Connected Papers** | A single seed paper, similarity graph without axes. No collections from multiple papers, no co-author view. |
| **Litmaps** | Freely chosen axes (e.g. year × citations), multiple seed papers per map, so whole research fields can be mapped. |
| **ResearchRabbit** | Collections from multiple papers, co-authors overlaid on the citation graph. |
| **Inciteful** | Free, no sign-up. "Literature Connector" finds the **shortest citation path between two papers**. |

The tools complement each other and are often combined in practice.

→ Relation: *Visualize Dependencies* (co-authors, citation graph). New ideas: freely chosen axes, shortest path between two sources.

Sources: [Effortless Academic: Litmaps vs ResearchRabbit vs Connected Papers](https://effortlessacademic.com/litmaps-vs-researchrabbit-vs-connected-papers-the-best-literature-review-tool-in-2025/) · [HKUST: comparison of mapping tools](https://libguides.hkust.edu.hk/citation-chaining/citation-mapping-tools-comparison) · [Ponder: ResearchRabbit alternatives](https://ponder.ing/blog/research-rabbit-alternatives)

## scite: Smart Citations

- Every citing statement is classified by deep learning: **supporting / contradicting / mentioning**.
- The sentence in which the citation occurs is shown together with the classification.
- scite claims 1.4 billion classified citation statements from over 38 million papers.
- Motivation: a citation that contradicts counts the same as one that supports in classic indices.

→ Relation: edges in the citation graph get a type. The classes also fit quote verification.

Sources: [scite (Quantitative Science Studies)](https://direct.mit.edu/qss/article/2/3/882/102990/scite-A-smart-citation-index-that-displays-the) · [scite Features](https://scite.ai/features)

## Extraction matrices: Elicit and ORKG

- **Elicit:** rows are papers, columns are freely defined questions (method, dataset, result, …). The LLM fills the cells, **each with a supporting citation**. Column templates are suggested or built by hand.
- **ORKG (Open Research Knowledge Graph):** papers are described as structured "contributions" (research problem, materials, methods, results). Comparison tables emerge semi-automatically from these.
- Practical tip from a guide: start with six core columns (citation, goal, method, results, limitations, relevance); add new columns only once a topic appears in three or more sources.

→ Relation: missing from `IDEA.md`, but a standard tool for literature reviews.

Sources: [Elicit Systematic Review](https://elicit.com/solutions/systematic-review) · [Aaron Tay: Research Matrix Tools](https://aarontay.medium.com/three-tools-that-can-help-you-create-a-literature-review-research-matrix-of-papers-scholarcy-a05af8ae5339) · [ORKG System Walkthrough](https://arxiv.org/pdf/2206.01439) · [PaperSynapse: Literature Review Tables](https://papersynapse.com/blog/how-to-structure-literature-review-data-tables)

## Canvas and annotation tools

- **LiquidText:** infinite canvas, several documents side by side, excerpts stay linked to their spot in the PDF.
- **MarginNote:** excerpts turn into mind maps and flashcards.
- **Heptabase:** cards on whiteboards, split into "Research Canvas" (understanding) and "Creative Canvas" (building from it).
- Common combination in academia: Zotero plus Obsidian/Heptabase for synthesis.

→ Relation: *Integrated note-taking*, a free canvas as a later view.

Sources: [Paperlike: LiquidText vs MarginNote](https://paperlike.com/blogs/paperlikers-insights/liquidtext-vs-marginnote) · [Storyflow: Heptabase alternatives](https://storyflow.so/blog/best-heptabase-alternatives-2026) · [Myflexnote: note-taking for researchers](https://myflexnote.com/blog/best-note-taking-apps-for-researchers)

## Discourse Graphs (Joel Chan)

- Notes have a type: **question, claim, evidence**.
- Links have a type: **supports, contradicts, answers**.
- Contradicting claims may coexist, because each is connected to its own evidence.
- This structures literature searches and makes them reusable.

→ Relation: the three note levels from `IDEA.md` map onto this. The sentences in one's own paper are claims; quote verification checks the "evidence supports claim" edge.

Sources: [Protocol Labs: Discourse Graphs and the Future of Science](https://research.protocol.ai/blog/2023/discourse-graphs-and-the-future-of-science/) · [Scaling Synthesis: decentralized discourse graph](https://scalingsynthesis.com/q-what-is-a-decentralized-discourse-graph/)

## Retracted papers (Retraction Watch)

- The Retraction Watch database has belonged to **Crossref** since 2023. Over 63,000 documented retractions.
- A paper can be flagged as retracted only in Retraction Watch, only in Crossref, or in both.
- **Zotero** has warned about retracted entries in the library and when citing via the word-processor plugin since 2019.

→ Relation: missing from `IDEA.md`, low effort, high value.

Sources: [Zotero Blog: Retracted item notifications](https://www.zotero.org/blog/retracted-item-notifications/) · [TU Hamburg on Retraction Watch](https://www.tub.tuhh.de/en/2026/02/23/retraction-watch-retracted-articles/) · [Zotero Forum: Retraction Watch vs Crossref](https://forums.zotero.org/discussion/130655/retracted-articles-solely-identified-by-the-retraction-watch-database-or-also-by-crossref-api)

## Own feature ideas from the brainstorming

- Citation suggestions while writing, drawn from one's own library, explicitly including contradicting sources.
- "Cited but never read" warning (no highlight, no note on the source).
- Gaps in the library: papers that many of one's own sources cite but which are missing.
- Comparison between paper and presentation: what one cites that the other doesn't; verification status is shared.
- Search hits as excerpts in multibuffer style instead of a list of papers.
- Metadata check (venue/volume/issue complete? preprint instead of peer-reviewed version?), derived from the formal check in the EHR seminar.
