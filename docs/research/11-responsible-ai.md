# Verantwortungsvolle KI-Nutzung als Designprinzip

Ergebnis aus dem Brainstorming (2026-09-14), vom Nutzer als Aufteilung bestätigt.
Ausgangspunkt waren die KI-Regeln des EHR-Seminars (FU Berlin, „Guidelines on AI usage,
presentation & report"), die Prinzipien gelten aber allgemein.

## A. Das Tool schreibt keinen Text für den Nutzer

1. **Grundregel im Datenmodell:** LLM-Funktionen recherchieren, ordnen und prüfen nur. In Dokumente gelangt ausschließlich Wortlaut aus Quellen, nie Modell-Wortlaut. Technisch erzwungen: Es gibt keinen Pfad von LLM-Ausgabe in eine `.typ`-Datei.
2. **Ähnlichkeitswarnung:** Der Language Server vergleicht eigene Sätze mit protokollierten LLM-Ausgaben und warnt („ähnelt Modell-Ausgabe vom …: kennzeichnen oder umformulieren"). Setzt Kennzeichnungspflichten direkt im Editor um.

## B. Nachvollziehbarkeit statt Vertrauen

3. **Herkunft an jedem Datenobjekt:** *vom Nutzer*, *deterministisch geprüft*, *importiert, Herkunft unbekannt*, *LLM-Einschätzung* (Modell, Version, Datum), *vom Nutzer bestätigt*. LLM-Urteile werden nie automatisch „wahr", zeigen immer die Belegstelle und warten auf Bestätigung.
4. **Keine erfundenen Quellen:** Vorschläge nur aus der eigenen Bibliothek, immer mit verankerter Textstelle.
5. **Reproduzierbarkeit:** Prompt, Modell, Parameter und Hash der Eingabe werden zu jeder LLM-Einschätzung gespeichert.

## C. Dokumentationspflicht automatisieren

6. **Vollständiges Nutzungsprotokoll** jedes LLM-Aufrufs: Tool, Version, Datum, URL, Prompt, Ergebnis, Art der Nutzung (Recherche, Prüfung, Struktur …).
7. **Anhang-Generator:** Typst-Tabelle deterministisch aus dem Protokoll, nicht von einem LLM formuliert → „geschönte" Angaben strukturell ausgeschlossen.
8. **Claude-Code-Sessions erfassen (Idee):** erkennen, welche Sessions in `~/.claude/projects/` ein Projekt berührt haben, und zählen.

## D. Regeln pro Projekt

9. **Richtlinie pro Projekt:** erlaubte LLM-Funktionen, Cloud ja/nein, Protokollpflicht; vom Tool durchgesetzt.
10. **Datenabfluss sichtbar:** Bei Cloud-Aufrufen festhalten, welcher eigene Text das Gerät verlassen hat.

## E. Lesen nicht wegautomatisieren

11. **„Zitiert, aber nie gelesen"-Hinweis;** Skimming-Hilfen (Scim-Stil) als Lesehilfe, nicht als Ersatz.

## Umfang in der ersten Version

Die erste Version enthält kein LLM. Trotzdem von Anfang an:

- **Herkunftsfelder im Datenmodell (3).**
- **Tabelle für das Nutzungsprotokoll (6)**, zunächst leer; jede spätere LLM-Funktion muss sie nutzen.

Alles andere als verbindliche Designprinzipien in der Spec, umgesetzt mit den LLM-Features.
