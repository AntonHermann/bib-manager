# Offene Fragen und ungeprüfte Annahmen

Stand 2026-09-16.

## Ungeprüft, aber im Design vorausgesetzt

- **Zed mit zwei Typst-Language-Servern:** Hover-Zusammenführung und References-Fan-out stammen aus Doku/Issues, nicht aus einem praktischen Test mit tinymist. Früh im Plan prüfen.
- **`pdf_oxide`-Lücke bei Type1-Encodings:** Ursache nur vermutet; ob ein Reparaturschritt außerhalb der Bibliothek möglich ist (Zugriff auf Schrift-Encoding und Zeichencodes über die öffentliche API?) ist nicht geprüft.
- **Leserichtung/Textqualität:** Übereinstimmung zwischen Extraktoren nur 64–89 %; wer richtig liegt, ist offen → Benchmark mit Referenztext.
- **Abadi-2016-Download** hatte laut `file` nur 2 Seiten; Ergebnisse für dieses PDF mit Vorsicht lesen.
- **Abdeckung von OpenAlex/Semantic Scholar** für die eigene Bibliothek nicht gemessen.
- **LLM-Geschwindigkeit** auf dem Laptop nur geschätzt.
- **Zotero lokale API**: Annotationen lesen, Citation-Key-Feld und Anhangspfade in Zotero 9.0.1 (Snap) nicht praktisch abgefragt.
- **Normalisierung von `ϵ`/`ε`:** NFKC-Verhalten für U+03F5 und mathematische Varianten vor der Implementierung genau prüfen.

## Offene Designfragen

- Projekt-Markierung im Repo: Format und Name der Datei mit der Projekt-ID.
- Backup- und Exportformat der zentralen Datenbank.
- Wie werden Belegstellen ohne PDF (Quelle nur in Zotero, kein Anhang) behandelt?
- Umgang mit Quellen, die nicht in Zotero sind (im Seminar gab es solche).
- Seitenangaben im Zitat (`supplement`) zur Eingrenzung der Wortlaut-Suche nutzen?
- Textschicht versionieren: was passiert mit Ankern, wenn ein neues Backend oder eine neue Normalisierung kommt (Neuauflösung + Prüfliste ist vorgeschlagen)?
- Verhältnis Zed-Extension ↔ Programm `bib`: Auslieferung/Download des Binaries.
- Mehrdeutige Treffer eines Zitats in einer Quelle (mehrfach vorkommender Wortlaut).

## Ideen für später (nicht entschieden)

- Upstream-Beitrag zu `pdf_oxide` für Type1-Glyphennamen.
- Vision-Modell per Cloud (olmOCR) nur für problematische Seiten.
- CiteSee-Färbung und CiteRead-Randnotizen in einer eigenen Leseoberfläche.
- Discourse-Graph-Modell für Notizen.
- Metadaten-Prüfung nach Formalcheck-Vorbild.
