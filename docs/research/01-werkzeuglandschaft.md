# Werkzeuglandschaft: Was es schon gibt

Recherche 2026-09-14. Bezug zu den Punkten in `IDEA.md` jeweils am Ende.

## Zitationsgraphen und Entdecken von Literatur

| Tool | Besonderheit |
|---|---|
| **Connected Papers** | Ein Start-Paper, Ähnlichkeitsgraph ohne Achsen. Keine Sammlungen aus mehreren Papers, keine Co-Autoren-Ansicht. |
| **Litmaps** | Frei wählbare Achsen (z. B. Jahr × Zitationen), mehrere Start-Papers pro Karte, damit ganze Forschungsfelder abbildbar. |
| **ResearchRabbit** | Sammlungen aus mehreren Papers, Co-Autoren über dem Zitationsgraphen eingeblendet. |
| **Inciteful** | Kostenlos, ohne Anmeldung. „Literature Connector" findet den **kürzesten Zitationspfad zwischen zwei Papers**. |

Die Tools ergänzen sich und werden in der Praxis oft kombiniert.

→ Bezug: *Visualize Dependencies* (Co-Autoren, Zitationsgraph). Neue Ideen: frei wählbare Achsen, kürzester Pfad zwischen zwei Quellen.

Quellen: [Effortless Academic: Litmaps vs ResearchRabbit vs Connected Papers](https://effortlessacademic.com/litmaps-vs-researchrabbit-vs-connected-papers-the-best-literature-review-tool-in-2025/) · [HKUST: Vergleich der Mapping-Tools](https://libguides.hkust.edu.hk/citation-chaining/citation-mapping-tools-comparison) · [Ponder: ResearchRabbit-Alternativen](https://ponder.ing/blog/research-rabbit-alternatives)

## scite: Smart Citations

- Jede zitierende Aussage wird per Deep Learning eingeordnet: **stützt / widerspricht / erwähnt**.
- Angezeigt wird der Satz, in dem zitiert wird, zusammen mit der Einordnung.
- Laut scite 1,4 Milliarden eingeordnete Zitat-Aussagen aus über 38 Millionen Papers.
- Motivation: Eine Zitation, die widerspricht, zählt in klassischen Indizes genauso wie eine, die stützt.

→ Bezug: Kanten im Zitationsgraphen bekommen einen Typ. Die Klassen passen auch zur Zitat-Prüfung.

Quellen: [scite (Quantitative Science Studies)](https://direct.mit.edu/qss/article/2/3/882/102990/scite-A-smart-citation-index-that-displays-the) · [scite Features](https://scite.ai/features)

## Extraktions-Matrizen: Elicit und ORKG

- **Elicit:** Zeilen sind Papers, Spalten frei definierbare Fragen (Methode, Datensatz, Ergebnis …). Das LLM füllt die Zellen, **jede mit Belegzitat**. Spaltenvorlagen werden vorgeschlagen oder selbst gebaut.
- **ORKG (Open Research Knowledge Graph):** Papers werden als strukturierte „Contributions" beschrieben (Forschungsproblem, Materialien, Methoden, Ergebnisse). Daraus entstehen halbautomatisch Vergleichstabellen.
- Praxis-Tipp aus einem Guide: mit sechs Kernspalten starten (Zitat, Ziel, Methode, Ergebnisse, Limitationen, Relevanz), neue Spalten erst, wenn ein Thema in drei oder mehr Quellen vorkommt.

→ Bezug: fehlt in `IDEA.md`, ist aber ein Standardwerkzeug für Literatur-Reviews.

Quellen: [Elicit Systematic Review](https://elicit.com/solutions/systematic-review) · [Aaron Tay: Research Matrix Tools](https://aarontay.medium.com/three-tools-that-can-help-you-create-a-literature-review-research-matrix-of-papers-scholarcy-a05af8ae5339) · [ORKG System Walkthrough](https://arxiv.org/pdf/2206.01439) · [PaperSynapse: Literature Review Tables](https://papersynapse.com/blog/how-to-structure-literature-review-data-tables)

## Canvas- und Annotationswerkzeuge

- **LiquidText:** unendliche Arbeitsfläche, mehrere Dokumente nebeneinander, Ausschnitte bleiben mit der Stelle im PDF verlinkt.
- **MarginNote:** Ausschnitte werden zu Mindmaps und Karteikarten.
- **Heptabase:** Karten auf Whiteboards, getrennt in „Research Canvas" (verstehen) und „Creative Canvas" (daraus etwas bauen).
- Verbreitete Kombination im Studium: Zotero plus Obsidian/Heptabase zur Synthese.

→ Bezug: *Integrated note-taking*, freie Arbeitsfläche als spätere Ansicht.

Quellen: [Paperlike: LiquidText vs MarginNote](https://paperlike.com/blogs/paperlikers-insights/liquidtext-vs-marginnote) · [Storyflow: Heptabase-Alternativen](https://storyflow.so/blog/best-heptabase-alternatives-2026) · [Myflexnote: Note-Taking für Forschende](https://myflexnote.com/blog/best-note-taking-apps-for-researchers)

## Discourse Graphs (Joel Chan)

- Notizen haben einen Typ: **Frage, Behauptung, Beleg**.
- Verknüpfungen haben einen Typ: **stützt, widerspricht, beantwortet**.
- Widersprüchliche Behauptungen dürfen nebeneinander stehen, weil jede mit ihren Belegen verbunden ist.
- Literatursuchen werden dadurch strukturiert und wiederverwendbar.

→ Bezug: Die drei Notiz-Ebenen aus `IDEA.md` lassen sich darauf abbilden. Die Sätze im eigenen Paper sind Behauptungen, die Zitat-Prüfung prüft die Kante „Beleg stützt Behauptung".

Quellen: [Protocol Labs: Discourse Graphs and the Future of Science](https://research.protocol.ai/blog/2023/discourse-graphs-and-the-future-of-science/) · [Scaling Synthesis: decentralized discourse graph](https://scalingsynthesis.com/q-what-is-a-decentralized-discourse-graph/)

## Zurückgezogene Papers (Retraction Watch)

- Die Retraction-Watch-Datenbank gehört seit 2023 zu **Crossref**. Über 63.000 dokumentierte Rückzüge.
- Ein Paper kann nur bei Retraction Watch, nur bei Crossref oder bei beiden als zurückgezogen markiert sein.
- **Zotero** warnt seit 2019 bei zurückgezogenen Einträgen in der Bibliothek und beim Zitieren über das Textverarbeitungs-Plugin.

→ Bezug: fehlt in `IDEA.md`, geringer Aufwand, hoher Nutzen.

Quellen: [Zotero Blog: Retracted item notifications](https://www.zotero.org/blog/retracted-item-notifications/) · [TU Hamburg zu Retraction Watch](https://www.tub.tuhh.de/en/2026/02/23/retraction-watch-retracted-articles/) · [Zotero Forum: Retraction Watch vs Crossref](https://forums.zotero.org/discussion/130655/retracted-articles-solely-identified-by-the-retraction-watch-database-or-also-by-crossref-api)

## Eigene Feature-Ideen aus dem Brainstorming

- Zitat-Vorschläge beim Schreiben aus der eigenen Bibliothek, ausdrücklich auch widersprechende Quellen.
- Warnung „zitiert, aber nie gelesen" (keine Markierung, keine Notiz in der Quelle).
- Lücken in der Bibliothek: Papers, die viele eigene Quellen zitieren, aber fehlen.
- Abgleich zwischen Paper und Präsentation: Was zitiert die eine, was das andere nicht; Prüfstatus wird geteilt.
- Suchtreffer als Belegstellen im Multibuffer-Stil statt als Paper-Liste.
- Metadaten-Prüfung (Venue/Volume/Issue vollständig? Preprint statt begutachteter Fassung?), abgeleitet aus dem Formalcheck im EHR-Seminar.
