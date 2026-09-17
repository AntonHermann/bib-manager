# Datenquellen und APIs

Recherche 2026-09-14.

## Semantic Scholar Academic Graph API

**Eigener Test** mit der Hauptquelle des EHR-Seminars
(Jonnagaddala & Wong 2025, *Privacy preserving strategies for electronic health records in the era of large language models*):

```
GET /graph/v1/paper/search/match?query=<Titel>&fields=title,year,citationCount
→ paperId 3aadb2b130320424dc5b11711715760c619fdb8d, year 2025, citationCount 79

GET /graph/v1/paper/3aadb2b1.../citations?fields=title,contexts,intents,isInfluential&limit=100
```

| Feld | Ergebnis |
|---|---|
| zitierende Papers | 79 |
| davon mit `contexts` (Satz, in dem zitiert wird) | **41** |
| davon mit `intents` (Background/Method/Result) | **0** |
| davon `isInfluential` | 3 |

- Ohne API-Key kam beim ersten Versuch `429 Too Many Requests`; nach kurzer Pause ging es.
- Die Zitat-Sätze sind genau das, was im Seminar für die Forward-Recherche von Hand durchgesehen wurde → automatisierbar.
- Intents gibt es laut Semantic Scholar nur für Papers mit Volltextzugang. Eine Stichprobe von CASRAI (5 × 100 Zitationen) fand Intents bei 0–23 %, Contexts bei 4–84 %. **Nicht als vollständige Annotationsschicht behandeln.**
- `isInfluential` ist absichtlich selten (ML-Modell über Anzahl und Kontext der Zitationen).
- Statische SPECTER2-Embeddings sind über die Paper-Endpunkte abrufbar.

Quellen: [Semantic Scholar: Citation Intent](https://www.semanticscholar.org/faq/citation-intent) · [CASRAI: gemessene Rate Limits und Datendichte](https://casrai.org/guides/semantic-scholar-api) · [Semantic Scholar Open Data Platform](https://arxiv.org/pdf/2301.10140) · [API-Key-Formular](https://www.semanticscholar.org/product/api#api-key-form)

## OpenAlex

- Vollständig offener Index, Snapshot vom 30.06.2024: 256.997.006 Werke (1400–2024), inkl. Datensätze, Peer-Review-Berichte, Bücher.
- Seit Februar 2026 Preismodell: kostenlose API-Keys mit Tageskontingent; **Einzelabrufe per ID oder DOI bleiben kostenlos**.
- Komplette Snapshots herunterladbar.
- Bekannte Lücken: Abdeckung einzelner Länder (z. B. China) und sprachliche Metadatenqualität.

Einschätzung: Für die Bibliothek des Nutzers (Informatik, etwas Medizin, einige Hundert Einträge mit DOI/arXiv) decken OpenAlex und Semantic Scholar den Zitationsgraphen gut ab. Nicht an der eigenen Bibliothek verifiziert.

Quellen: [CASRAI: OpenAlex API](https://casrai.org/guides/openalex-api) · [IntuitionLabs: OpenAlex vs Semantic Scholar vs PubMed](https://intuitionlabs.ai/articles/openalex-semantic-scholar-pubmed-comparison) · [IntuitionLabs: Research Paper APIs 2026](https://intuitionlabs.ai/articles/research-paper-apis-scientific-literature) · [OpenAlex (Wikipedia)](https://en.wikipedia.org/wiki/OpenAlex) · [Linguistische Abdeckung](https://arxiv.org/pdf/2409.10633) · [Abdeckung China](https://arxiv.org/pdf/2507.19302)

## S2ORC

- 81,1 Mio. englischsprachige Papers mit Metadaten, Abstracts, aufgelösten Referenzen; **strukturierter Volltext für 8,1 Mio. Open-Access-Papers**.
- Volltext annotiert mit Inline-Erwähnungen von Zitaten, Abbildungen, Tabellen, verknüpft mit den Paper-Objekten.
- Seit Februar 2023 über die Semantic Scholar API (Datasets) zugänglich.

Quellen: [S2ORC (ACL 2020)](https://aclanthology.org/2020.acl-main.447/) · [allenai/s2orc](https://github.com/allenai/s2orc) · [Hugging Face](https://huggingface.co/datasets/allenai/s2orc)

## Embeddings: SPECTER2

- Zitationsbasiert trainierte Dokument-Embeddings (auf SciBERT), 9 Aufgaben, 23 Fachgebiete.
- Adapter für Klassifikation, Regression, Nähe (proximity) und Ad-hoc-Suche.
- Arbeitet auf **Titel + Abstract**, also Ebene ganzer Papers → gut für „ähnliche Papers", weniger für Textstellen.
- Hybride Suche: SPECTER2 + BM25, zusammengeführt mit Reciprocal Rank Fusion.
- Für Textstellen-Suche besser ein allgemeines Passage-Embedding-Modell; beides kombinierbar.

Quellen: [AI2 Blog: SPECTER2](https://allenai.org/blog/specter2-adapting-scientific-document-embeddings-to-multiple-fields-and-task-formats-c95686c06567) · [allenai/specter](https://github.com/allenai/specter) · [ISLE (hybride Suche)](https://arxiv.org/pdf/2512.12760)

## PaperQA2 (FutureHouse / Edison Scientific)

- Agentisches RAG über lokale PDF-Sammlungen.
- Drei Suchstrategien: Volltext mit **tantivy** (Rust), semantische Suche mit metadaten-bewussten Embeddings, LLM-Reranking + „Retrieval-augmented Contextual Summarization".
- Reichert Chunks mit Zitationszahlen (Semantic Scholar), Venue, Autoren und **Rückzugsstatus** (Crossref, Unpaywall) an.
- Standard: OpenAI-Embeddings + NumPy-Vektorspeicher, austauschbar.

Quellen: [paper-qa](https://github.com/future-house/paper-qa) · [Edison Docs](https://docs.edisonscientific.com/paperqa) · [PaperQA-Paper](https://arxiv.org/pdf/2312.07559) · [Starlog-Analyse](https://starlog.is/articles/data-knowledge/future-house-paper-qa/)

## GROBID

- Aktiv (`grobidOrg/grobid`, Push 2026-09-13, Apache-2.0, 5,1k Sterne). Umgezogen von `kermitt2/grobid`.
- Strukturiert Volltext: Absätze, Überschriften, **Referenz-Callouts** (`ref`), Abbildungen, Tabellen, Referenzliste aufgeschlüsselt.
- Parameter `teiCoordinates`: Koordinaten für gewählte Strukturen; `segmentSentences`: Sätze (`s`) mit Boxen.
- Koordinaten im PDF-Koordinatensystem (Ursprung oben links).
- Standardmäßig CRF (Wapiti-Fork), läuft gut auf CPU; optional DeLFT-Deep-Learning-Modelle, ohne GPU weniger skalierbar.
- Bekanntes Issue: teilweise unvollständige Koordinaten für Satzelemente (#811).
- Bereits im großen Maßstab genutzt (CORE: 34 Mio. Dokumente).

Einsatz im Projekt: Zitatmarker „[18]" ↔ Referenzeintrag (Grundlage für CiteSee-Färbung), Satzgrenzen, Rückfall für Referenzlisten ohne DOI. **Nicht in der ersten Version.**

Quellen: [GROBID: Koordinaten im PDF](https://grobid.readthedocs.io/en/latest/Coordinates-in-PDF/) · [GROBID REST API](https://grobid.readthedocs.io/en/latest/Grobid-service/) · [How GROBID works](https://grobid.readthedocs.io/en/latest/Principles/) · [Issue #811](https://github.com/kermitt2/grobid/issues/811) · [CORE + GROBID](https://blog.core.ac.uk/2023/07/17/core-grobid-structured-text-from-34-million-scientific-documents-and-counting/)

## Crossref / Retraction Watch

Siehe `01-tool-landscape.md`, Abschnitt „Zurückgezogene Papers", und `06-zotero.md`.
