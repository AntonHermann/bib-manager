# Zitatprüfung: Wie groß ist das Problem, was gibt es?

Recherche 2026-09-14.

## Häufigkeit von Zitierfehlern (Medizin)

**Jergas & Baethge 2015** (PeerJ), systematischer Review und Meta-Analyse über 28 Studien:

| Fehlerart | Rate | 95-%-KI |
|---|---|---|
| grob (major) | 11,9 % | 8,4–16,6 |
| gering (minor) | 11,5 % | 8,3–15,7 |
| **gesamt** | **25,4 %** | 19,5–32,4 |

- Große Heterogenität, aber selbst die niedrigste Gesamtschätzung lag bei 6,7 %.
- Indirekte Zitate (Zitat aus zweiter Hand) machen weniger als ein Sechstel der Probleme aus.

**Update 2025** (Research Integrity and Peer Review):

- 16,9 % der Zitate fehlerhaft (KI 14,1–20,0 %), davon etwa die Hälfte grob: 8,0 % (KI 6,4–10,0 %).
- Meta-Regression: **keine Verbesserung über die Jahre.**

Das ist das stärkste Argument für eine eingebaute Zitat-Prüfung.

Quellen: [Jergas & Baethge 2015 (PeerJ)](https://peerj.com/articles/1364/) · [PMC-Fassung](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4627914/) · [Update 2025 (Springer)](https://link.springer.com/article/10.1186/s41073-025-00173-z) · [PMC-Fassung Update](https://pmc.ncbi.nlm.nih.gov/articles/PMC12285159/) · [Methodik-Kritik und Neuberechnung (PubMed)](https://pubmed.ncbi.nlm.nih.gov/28910404/)

## Prüfsysteme

Zwei verschiedene Fragen werden oft vermischt:

1. **Existiert die Quelle, und stimmen die Angaben?** (Problem v. a. bei LLM-generierten Texten)
2. **Stützt die Quelle die Aussage?** (das, was uns interessiert)

| System | Frage | Ansatz | Ergebnis |
|---|---|---|---|
| **SemanticCite** (Haan 2025) | 2 | Volltextprüfung, vier Klassen: *supported / partially supported / unsupported / uncertain*; kleine feinjustierte Modelle | 84 % gewichtete Genauigkeit |
| **CiteGuard** (Choi et al. 2026) | 2 (Zuordnung) | Retrieval-gestützte Validierung der Zitationszuordnung | 68 % auf CiteME, Menschen 70 % |
| **CiteCheck** | 1 | drei Schweregrade: exakt / kleinere Metadatenfehler / erfunden; Suche + LLM-Bewertung + automatische Korrektur | – |
| **CiteAudit** (Yuan et al. 2026) | 1 | Multi-Agent-Verifikation, Benchmark | – |
| Abbonato (2026) | 1 | Abgleich gegen bibliografische Metadaten | – |

- LLMs erzeugen je nach Studie bis zu 78–90 % erfundene Zitate.
- Die vier Klassen von SemanticCite passen gut zu „Score + Status-Übersicht" aus `IDEA.md`.
- Kleine Modelle reichen für die Einordnung → lokal realistisch (siehe `09`).

Quellen: [CiteCheck](https://arxiv.org/html/2605.27700v1) · [CiteAudit](https://arxiv.org/html/2602.23452v3) · [CiteGuard](https://arxiv.org/html/2510.17853v4) · [Cited but Not Verified](https://arxiv.org/html/2605.06635v1) · [Reference Hallucinations in Deep Research Agents](https://arxiv.org/pdf/2604.03173) · [INRA.AI Blog](https://www.inra.ai/blog/citation-accuracy)

## Folgerungen für das Design

- Stufe 0 (erste Version, ohne LLM): **Wortlaut-Prüfung** wörtlicher Zitate gegen die Textschicht. Deterministisch.
- Stufe 1: passende Stelle in der Quelle finden (Suche/Embeddings), grob einordnen, lokal.
- Stufe 2: gründlicher Bericht per LLM, lokal im Hintergrund oder per Cloud.
- Jedes LLM-Urteil zeigt die Belegstelle und wird nie automatisch „wahr" (siehe `11`).
- Seitenangaben im Zitat (`supplement`) grenzen die Suche stark ein.
- Empirische Aussagen sind verlässlicher prüfbar als sinngemäße Wiedergaben von Argumenten.
