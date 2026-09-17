# Design: Bibliothekskern und Typst-Anbindung (Teilprojekt 1 + 2)

**Datum:** 2026-09-16
**Status:** Entwurf zur Abnahme
**Vorgänger:** `IDEA.md`

---

## 1. Ziel und Kontext

Das Projekt aus `IDEA.md` ist zu groß für eine Spec. Diese Spec beschreibt die **erste nutzbare Version**: Bibliothekskern plus Typst-Anbindung. Sie soll die selbstgebauten Skripte des Autors durch ein zusammenhängendes Werkzeug ersetzen.

### Der bestehende Workflow

Ermittelt aus `~/Documents/seminar_ehr_ss26` (EHR-Privacy-Seminar, Typst-Paper plus Touying-Folien):

| Heute | Was es leistet |
|---|---|
| `notes/quote_verification.json` | 143 Belegstellen: Key, Kapitel-Label, Wortlaut, teils `validated` |
| `notes/check_quotes.py` | prüft Wortlaut gegen `pdftotext`-Volltexte, mit NFKC-Normalisierung, Key→Datei von Hand gepflegt |
| `notes/check_slide_quotes.py` | prüft Folien gegen die Zitatsammlung |
| `notes/quote_inserter.sh` | Zed-Task mit fzf, fügt `#quote(attribution: <key>)[…]` ein |
| `just pres-all-used-citations` | listet verwendete Keys eines Dokuments |
| `literatur/text/**.txt` | Volltexte, versioniert, mit grep durchsucht |
| `literatur/{A_kern,B_belege,C_rest}` | Triage-Ebenen als Ordner |
| `notes/Quellenauswahl.md` | Zitationsrecherche von Hand (Backward/Forward, Co-Autor-Prüfung) |
| `literatur/zotero_korrekturen.md` | Formalcheck der Metadaten |

Bekannte Schwachstellen dieses Workflows, die das Tool beheben soll: zerfallende Tabellen, verschwindendes ε in Mathe-Schriften, von Hand gepflegte Key→Datei-Zuordnung, Belegstellen ohne Anker, Triage nur als Ordnerstruktur.

### Umgebung

- Typst 0.15, geschrieben in **Zed** (neben tinymist)
- **Zotero 9.0.1** (Snap) mit nativen Citation Keys, Better BibTeX migriert; Tablet-Sync über Zotero
- Bibliothek: einige Hundert Einträge, Schwerpunkt Informatik, etwas Medizin
- Laptop: Ryzen 7 PRO 5850U, integrierte Grafik, 30 GiB RAM, keine dedizierte GPU

---

## 2. Entscheidungen

| Entscheidung | Begründung |
|---|---|
| **Zotero bleibt Quelle der Wahrheit**, das Tool liest nur | Zotero Connector und Tablet-Sync sind zu wertvoll, um sie zu ersetzen. Zurückschreiben ist später nachrüstbar, die Zotero-IDs werden dafür gespeichert. |
| **Alle Daten zentral in SQLite** | Belegstellen sind projektübergreifend nutzbar; ein Export bringt sie bei Bedarf ins Repo. |
| **Konfiguration dagegen im Repo** (`bib.toml`) | Von Hand geschrieben, versioniert, diff-bar. Bei KI-Regeln zusätzlich: über die Git-Historie belegbar, welche Regeln zum Abgabezeitpunkt galten. |
| **Rust, ein Programm, ein Workspace** | Kein Baustein der ersten Version braucht Python. `typst-syntax` gibt es nur in Rust und ist der offizielle Parser. |
| **Lizenz `MIT OR Apache-2.0`** | Ausdrücklicher Wunsch: Wirkung im Wissenschaftsbetrieb geht vor Copyleft. Schließt `mupdf-rs` (AGPL) als Abhängigkeit aus. |
| **Extraktion hinter einer Schnittstelle, Standard `pdf_oxide`** | Backend-Wahl bleibt billig revidierbar; Schritt 0a der Umsetzung ist ein Benchmark. |
| **Kein LLM in dieser Version** | Alles Nötige ist deterministisch. Die Vorkehrungen für spätere LLM-Funktionen sind trotzdem enthalten (Abschnitt 12). |
| **Kein Daemon** | Language Server und CLI sprechen direkt mit SQLite (WAL). Ein Daemon lohnt erst mit geladenen Modellen. |

---

## 3. Abgrenzung

**Enthalten:** Zotero-Abgleich (lesend), Textextraktion mit Normalisierung, Anker und Belegstellen, Korrekturen an Textschichten, Typst-Analyse, Wortlaut-Prüfung, Language Server, CLI, Import der bestehenden Zitatsammlung.

**Nicht enthalten** (jeweils eigenes Teilprojekt): GROBID und Satzstruktur, Zitatmarker im PDF, Embeddings und semantische Suche, Zitationsgraph und Metadaten-Anreicherung, Notizen und Discourse Graph, PDF-Oberfläche, Schreiben nach Zotero, Annotationen vom Tablet, jegliche LLM-Funktion, MCP-Server, Erkennung von Abbildungen und Tabellen.

---

## 4. Architektur

```
bib-core      Datenmodell, SQLite, Zotero, Normalisierung, Anker
bib-extract   Schnittstelle + Backends (pdf_oxide, pdf-extract, optional mutool)
bib-typst     Typst-Parser, Projekt- und Dokumentanalyse
bib-lsp       Language Server
bib-cli       Programm `bib`
zed-bib       Zed-Extension, startet `bib lsp` neben tinymist
```

```
Zotero (lokale API) ──► Quellen, Citation Keys, PDF-Pfade, Tags
PDF ──► Extraktion ──► Textschicht (Spans, Boxen, Seiten) ──► + Korrekturen ──► kanonischer Text
Belegstelle ──► Anker im kanonischen Text ──► Wortlaut-Prüfung
*.typ ──► Parser ──► Zitate, Kapitel, #quote ──► Verwendung im Projekt
```

---

## 5. Datenmodell

Eine SQLite-Datenbank an festem Ort (`~/.local/share/bib/bib.db`), WAL-Modus, versionierte Migrationen, Fremdschlüssel aktiv.

| Tabelle | Inhalt |
|---|---|
| `source` | Quelle aus Zotero: Bibliothek + Item-Key (Pflicht), Citation Key, Metadaten, Tags, Status (aktiv/stillgelegt) |
| `attachment` | PDF: Pfad, Prüfsumme, Seitenzahl |
| `text_layer` | Textschicht eines Anhangs: Backend je Seite, Backend-Version, Normalisierungs-Version, kanonischer Text (zstd), Qualitätsmerkmale |
| `page_geometry` | je Seite: Größe, Drehung, Koordinatensystem, Spans als kompakter Block mit Offset-Index |
| `text_patch` | manuelle Korrektur: Ziel als Anker, Ersatztext, Begründung, Herkunft, Status (aktiv/überflüssig) |
| `anchor` | **eigene Tabelle.** Art *Textbereich* (Wortlaut, je 32 Zeichen Kontext, Position) oder *Seitenbereich* (Seite, Rechteck). Zustand *verankert*, *mehrdeutig* oder *unverankert* |
| `excerpt` | wörtlicher Ausschnitt an einem Anker, mit Prüfstatus und Herkunft |
| `project` | Pfad, Abzug der `bib.toml`, Status |
| `document` | `.typ`-Datei eines Projekts, Art Paper oder Folien |
| `chapter` | Überschriftenbaum eines Dokuments |
| `citation` | Zitat im Dokument: Position, Key, Form, ggf. Belegstelle, ggf. „dynamisch" |
| `usage` | Verwendung einer Belegstelle in einem Projekt: Kapitel oder Label, Rolle |
| `llm_call` | Nutzungsprotokoll, in dieser Version leer angelegt |
| `review_queue` | alles, was ein Mensch anschauen muss |

### Prinzipien

**Anker getrennt von Inhalt.** Belegstelle, Korrektur und später Notiz oder Abbildung zeigen auf denselben Ankertyp. Knoten werden einheitlich als `knoten_typ` plus `knoten_id` referenziert; diese Konvention gilt ab jetzt überall.

**Anker nach W3C-Vorbild.** Wortlaut plus Kontext plus Position. Auflösung in der Reihenfolge exakt → normalisiert → unscharf (höchstens 5 % Abweichung), Kontext löst Mehrdeutigkeit auf. Ändert sich die Textschicht, werden alle Anker neu aufgelöst; Abweichungen gehen in die Prüfliste.

**Anker ohne Textschicht.** Hat eine Quelle (noch) kein PDF, kann eine Belegstelle trotzdem existieren: Ihr Anker ist *unverankert*, sie gilt als nicht prüfbar. Sobald ein PDF da ist, wird automatisch verankert.

**Mehrdeutige Treffer.** Für die *Prüfung* eines Wortlauts genügt ein Treffer. Beim *Anlegen* einer Belegstelle entscheidet bei mehreren Treffern zuerst der Kontext, dann eine Seitenangabe aus dem Dokument (Abschnitt 8), sonst bleibt der Anker *mehrdeutig* und landet mit allen Kandidaten in der Prüfliste.

**Belegstelle und Verwendung getrennt.** Der Ausschnitt gehört zur Quelle und ist projektübergreifend nutzbar, das Kapitel-Label gehört zur Verwendung im Projekt.

**Herkunft an jedem Eintrag:** *von dir*, *deterministisch geprüft*, *importiert, Herkunft unbekannt*, *LLM-Einschätzung* (mit Verweis auf `llm_call`), *von dir bestätigt*.

**Nicht neu erzeugbare Daten** sind Belegstellen, Anker, Verwendungen, Korrekturen, Prüflisten-Entscheidungen und später Notizen. Alles andere lässt sich aus Zotero und den PDFs neu bauen.

**Sicherung und Export:**

- **Backup** ist eine vollständige, konsistente Kopie der Datenbank per `VACUUM INTO`: automatisch vor jeder Migration und auf Befehl (`bib backup`), mit Aufbewahrung der letzten Stände.
- **Export** umfasst nur die nicht neu erzeugbaren Daten, als **JSON Lines je Tabelle** mit stabilen IDs. Das ist diff-bar, lesbar und unabhängig vom Datenbankschema; `bib import` kann daraus wiederherstellen. Projektbezogene Exporte (etwa im Format von `quote_verification.json`) sind zusätzlich über `[export]` in `bib.toml` konfigurierbar.

### Vorgriff: Notizen (Teilprojekt 4)

Damit das später ohne Umbau passt, steht die Form hier fest:

```
note ──< note_target ──► anchor | source | chapter | project | tag | note
link (von, nach, typ: stützt · widerspricht · beantwortet · verfeinert)
```

Eine Belegstelle ist ein wörtlicher, prüfbarer Ausschnitt; eine Notiz sind eigene Worte, typisierbar und verknüpfbar. Zotero-Markierungen werden später zu `anchor` plus `excerpt` mit Zotero-ID, Kommentare zu `note` am selben Anker. In dieser Version entstehen nur `anchor` und die Knoten-Konvention.

---

## 6. Zotero-Abgleich

- **Lokale API** unter `127.0.0.1:23119/api/users/0/…`, offline, ohne Kontingent, **nur lesend**. Voraussetzung: Zotero läuft und die Option für andere Programme ist aktiv.
- **Vollabgleich statt inkrementell.** Die lokale API liefert für alle Einträge `version = 0` und `Last-Modified-Version: 0`, und es gibt keinen `/deleted`-Endpunkt (gemessen, Abschnitt 17). Der Abgleich holt deshalb alle Einträge seitenweise und vergleicht mit der Datenbank: neu, geändert (über `dateModified` und eine Prüfsumme der Felder), verschwunden. Bei einigen Hundert Einträgen ist das billig.
- **Rückfallebene:** schreibgeschützte Kopie von `zotero.sqlite`, wenn Zotero nicht läuft. Ohne beides wird mit dem letzten Stand gearbeitet, dessen Alter überall sichtbar ist.
- **Übernommen:** Metadaten, nativer Citation Key (Feld `citationKey`), Tags, Collections, Anhänge. Der Dateipfad steht nicht in den Anhangsdaten, sondern im Link `enclosure` als `file://`-URL (alternativ `/items/<key>/file/view/url`). Betrachtet werden Anhänge mit `contentType = application/pdf`, unabhängig vom `linkMode`.
- **Gelöschte, zusammengeführte oder umbenannte Einträge** — erkannt daran, dass sie im Vollabgleich fehlen oder ihr Key sich geändert hat — werden stillgelegt, nicht gelöscht, und landen in der Prüfliste, weil Belegstellen daran hängen.
- **Annotationen** sind über die lokale API lesbar (`annotationText`, `annotationComment`, `annotationPosition`, `annotationPageLabel`, …), werden aber erst in Teilprojekt 4 übernommen.

**Zotero ist die einzige Quelle für Quellen.** Jede `source` stammt aus Zotero. Quellen, die nur in einer `.bib` stehen, werden nicht angelegt; das Tool fordert auf, sie in Zotero anzulegen (mit dem Connector ein Klick). Fremde `.bib`-Dateien lassen sich bei Bedarf in eine eigene Zotero-Bibliothek importieren.

**Mehrere Bibliotheken.** Geteilte Literatur läuft über Zotero-Gruppenbibliotheken; die lokale API liefert sie aus (`/api/users/0/groups` listet sie, `/api/groups/<id>/items` liefert Einträge, Anhänge und Annotationen — gemessen, Abschnitt 17). Daraus folgt:

- Eine Quelle wird über das Paar **Bibliothek + Item-Key** identifiziert, nicht über den Item-Key allein. Der Abgleich läuft über alle Bibliotheken.
- **Citation Keys sind nicht bibliotheksübergreifend eindeutig.** Im gemessenen Bestand kommen 17 Keys in mehr als einer Bibliothek vor, meist dasselbe Paper in eigener und Gruppenbibliothek. Welche Bibliotheken ein Projekt benutzt und in welcher Reihenfolge, steht deshalb in `bib.toml` (`[zotero] libraries`). Ein Key wird in dieser Reihenfolge aufgelöst.
- **Kollision innerhalb der Bibliotheken eines Projekts:** Stimmen DOI oder Titel und Jahr überein, gilt es als dasselbe Werk, und die erste Bibliothek gewinnt, ohne Meldung. Sonst Prüfliste.
- Einträge **ohne Citation Key** (kommt in Gruppenbibliotheken vor) werden übernommen, sind aber nicht zitierbar; `bib doctor` listet sie.

**Die `.bib` wird gelesen, nie geschrieben** (Bibliothek `biblatex`, dieselbe wie in Typst), und zwar nur im Speicher — eine `.bib` ist in Millisekunden geparst, eine eigene Tabelle braucht es nicht. Daraus entstehen drei Meldungen: Key nur in der `.bib` („nicht in Zotero, bitte dort anlegen"), Key nur in Zotero (Export veraltet, Zitat kompiliert nicht), Metadaten auseinandergelaufen.

---

## 7. Extraktion und Normalisierung

### Schnittstelle

Ein Backend liefert je Seite **entweder** Spans (Text, Box, Schrift) in Leserichtung samt Seitengröße und Drehung **oder** nur reinen Text. Text ist Pflicht, Geometrie optional: `pdf-extract` liefert über seine öffentliche API nur Text. Seiten ohne Geometrie tragen das Qualitätsmerkmal *keine Geometrie*; Anker lösen dort über den Text auf, und „PDF an dieser Stelle öffnen" springt nur auf die Seite. In dieser Version rendert nichts Boxen, deshalb reicht das.

Der Kern baut aus den Seiten den kanonischen Text und, wo vorhanden, die Zuordnung Textbereich → Seite und Box. Spans statt einzelner Zeichen halten die Datenmenge klein; Zeichen lassen sich bei Bedarf je Seite nachladen.

**Ein Koordinatensystem für alle Backends:** Punkte, Ursprung oben links, y wächst nach unten, relativ zur MediaBox. Jedes Backend rechnet selbst um (`pdf_oxide` und pdfium liefern PDF-Koordinaten mit Ursprung unten links, `mutool` bereits oben links). Ein Test prüft, dass dieselbe Textstelle in verschiedenen Backends auf wenige Punkte genau an derselben Stelle liegt.

### Kaskade, pro Seite

1. Standard ist **`pdf_oxide`** (MIT/Apache, reines Rust, reichhaltige Daten), bestätigt durch den Benchmark aus Schritt 0a (Entscheidungsregel 1, `bench/results/2026-09-17.md`).
2. Ein **Verdachtsprüfer** bewertet: Steuerzeichen oder `(cid:…)`, Schriften ohne Unicode-Zuordnung, Mathe-Schriften wie CMMI ohne ein einziges griechisches Zeichen, gar kein Text.
3. Bei Verdacht läuft **`pdf-extract`** (MIT) über dieselbe Seite; gespeichert wird das bessere Ergebnis samt Angabe des Backends. Im Benchmark lieferte `pdf-extract` in beiden Dokumenten, in denen `pdf_oxide` ε fehlte, alle erwarteten Zeichen.
4. **`mutool`**, falls installiert, ist dritte Stufe und Vergleichsmaß im Benchmark.

Pro Seite festzuhalten, welches Backend gewonnen hat, erlaubt später, einzelne kaputte Seiten gezielt durch ein OCR- oder Vision-Modell zu ersetzen.

Ein eigener Reparaturschritt für fehlende Unicode-Zuordnungen (Glyphennamen aus eingebetteten Type1-Schriften) wäre wünschenswert und als Beitrag an `pdf_oxide` sinnvoll, ist aber **keine Voraussetzung** dieser Version.

### Korrekturen

Manuelle Korrekturen sind **keine weitere Stufe der Kaskade**, sondern eine gespeicherte Ebene darüber: Extraktion, dann Korrekturen in fester Reihenfolge, ergibt den kanonischen Text. Bei Neuextraktion werden Korrekturen über ihren Anker wiedergefunden. Drei Fälle: erneut angewendet; als *überflüssig geworden* markiert, wenn das neue Backend die Stelle richtig liefert; Prüfliste, wenn die Stelle unauffindbar ist.

### Normalisierung

Zwei Ebenen mit Offset-Zuordnung. Roh ist, was extrahiert wurde; die Vergleichsform wendet NFKC an (macht `ϵ`, `𝜖` zu `ε` und `ﬀ` zu `ff`), entfernt weiche Trennstriche, fügt Trennungen am Zeilenende zusammen, vereinheitlicht Striche und Anführungszeichen, fasst Leerraum zusammen und ignoriert Groß- und Kleinschreibung. Gesucht wird in der Vergleichsform, gespeichert und angezeigt wird roh.

### Qualitätsmerkmale

Je Seite: nicht zuordenbare Zeichen, verdächtige Schriften, kein Text. Sie erscheinen in den Meldungen des Language Servers, damit aus „Wortlaut nicht gefunden" ein verwertbarer Hinweis wird.

**Tabellen erkennt diese Version nicht.** Sie erscheinen als Text in Leserichtung, was bei mehrspaltigen Tabellen unbrauchbar sein kann.

---

## 8. Typst-Analyse

Parser ist `typst-syntax`, derselbe wie in Typst und tinymist. Es wird **geparst, nicht kompiliert**.

Erkannt werden: `@key`; `#cite(<key>, form:, supplement:)`; `#quote(attribution: <key>)[…]`; `#quote(block: true, attribution: [@key])[…]`; Überschriften und Labels; `#include` und `#import`; `#bibliography("…")` samt Typst-Pfadregeln; der umgebende Absatz eines Zitats.

Beim Wortlaut eines `#quote` wird der reine Text gesammelt, Escapes aufgelöst, Auszeichnungen verworfen.

**Seitenangaben** aus `supplement` (etwa `[S. 12]`, `[p. 12–13]`) dienen nur als **Hinweis**, nie als Einschränkung: Die genannte Seite wird zuerst durchsucht. Gedruckte Seitenzahlen weichen oft vom PDF-Seitenindex ab, deshalb wird gegen die Seitenlabels des PDFs abgeglichen, falls vorhanden. Wird der Wortlaut nur auf einer anderen Seite gefunden, gilt die Prüfung als bestanden, mit einem Hinweis auf die abweichende Seitenangabe.

**Projekt:** Verzeichnis mit `bib.toml`. Einstiegspunkte stehen dort, weitere Dateien folgen aus `#include`. Im Editor wird nur die geänderte Datei neu geparst, Dateiübergreifendes kommt aus der Datenbank.

**Grenze:** Dynamisch erzeugte Zitate (im Beispielprojekt `#cite(label, form: "prose")` in einer Hilfsfunktion) sind ohne Kompilieren nicht auflösbar. Sie werden als *dynamisch* markiert, nicht als Fehler. Optional vergleicht `typst query <datei> "cite"` die Menge der kompilierten Zitate mit der geparsten und meldet die Differenz.

---

## 9. Language Server und Zed

Tested in step 0b (Zed 1.20.2, tinymist 0.15.8, `docs/research/14-zed-zwei-language-server.md`): Zed starts a second Typst language server next to tinymist without any settings change. Diagnostics, hover and completion of both servers are merged (the second server's entries appeared first). Code actions, definition and references of the second server work; whether Zed merges them with tinymist's results is untested, because tinymist returned nothing at any position tried. At `@key` citations tinymist offers no hover, definition, references or completion, so the editor features below do not compete with tinymist there (caveat: tested in a single file without a pinned main file).

**Meldungen:** Wortlaut stimmt nicht (mit Seitenqualität als Begründung); Key unbekannt; Key fehlt in der `.bib`; Metadaten weichen ab; dynamisches Zitat (Hinweis); optional: zitiert ohne Belegstelle. Der Schweregrad kommt aus `bib.toml`. Die Meldung zu unbekannten Keys ist abschaltbar, weil tinymist Ähnliches meldet.

**Hover** auf Key oder Attribution: Metadaten, Anzahl und Prüfstatus der Belegstellen, Tags, PDF vorhanden, Alter des Zotero-Stands.

**Vervollständigung:** nach `@` die Keys, sortiert nach Verwendung im Projekt; innerhalb von `#quote(attribution: <key>)[` die Belegstellen dieser Quelle. Letzteres ersetzt `quote_inserter.sh`.

**Aktionen:** „Diesen Wortlaut als Korrektur der Textschicht übernehmen" (braucht kein Eingabefeld, der Text steht schon im Dokument); „Belegstelle als von mir bestätigt markieren"; „Als Belegstelle speichern" mit Kapitel-Label aus der umgebenden Überschrift; „PDF an dieser Stelle öffnen" über `zotero://open-pdf/…`.

**Navigation:** References auf `@key` über Paper und Folien (Multibuffer, ersetzt `pres-all-used-citations`); Definition öffnet den extrahierten Text der Quelle aus dem Cache an der zitierten Stelle; Gliederung aus den Überschriften.

**Innenleben:** Der Server hält die Datenbankverbindung, beobachtet `.bib` und `bib.toml`, stößt beim Öffnen einen Abgleich an, erledigt Schweres im Hintergrund und meldet Fortschritt über `$/progress`. Die Zed-Extension meldet nur `bib lsp` für Typst an.

**Auslieferung des Programms:** In dieser Version sucht die Extension `bib` im `PATH` oder unter einem in den Zed-Einstellungen konfigurierten Pfad; installiert wird per `cargo install`. Fehlt das Programm, zeigt die Extension einen Hinweis mit dem Installationsbefehl. Später lädt die Extension ein passendes Binary aus GitHub-Releases, wie es viele Zed-Extensions für ihre Language Server tun.

---

## 10. CLI

`bib init`, `sync`, `index`, `check`, `cites`, `search`, `text`, `excerpt`, `patch`, `review`, `export`, `import`, `backup`, `doctor`.

Durchgängig `--json` für Skripte, Git-Hooks und Claude Code, dazu sinnvolle Exit-Codes. `bib check` verhält sich wie das bestehende Skript: still bei Erfolg, Fehlercode bei echtem Fehlschlag, `-v` zeigt auch Bestandenes, Ausgabe als `datei:zeile` mit nächstliegendem Kandidaten.

### Konfiguration

`~/.config/bib/config.toml` für Voreinstellungen, `bib.toml` im Projekt hat Vorrang und wird versioniert. Zugangsdaten stehen in keiner der beiden Dateien.

```toml
id = "0193f2a1-…"
name = "EHR Privacy Seminar"

[[documents]]
path = "paper.typ"
kind = "paper"

[[documents]]
path = "presentation/slides.typ"
kind = "slides"

[zotero]
libraries = ["user", "group:6573630"]   # Auflösungsreihenfolge für Citation Keys

[bibliography]
path = "ehr_privacy.bib"
managed_by = "zotero"

[export]
excerpts = "notes/quote_verification.json"

[diagnostics]
unverified_quote = "error"

[ai]
allowed = ["retrieval", "verification"]
cloud = false
logging = "required"
```

Die Datei ist Pflicht und gewinnt gegen die Datenbank. Taucht dieselbe ID an zwei Pfaden auf, fragt die Prüfliste nach verschoben oder kopiert.

---

## 11. Import und Abnahme

Für jeden Eintrag aus `quote_verification.json`: Key auflösen; Wortlaut in der Textschicht suchen (exakt, normalisiert, unscharf); daraus Anker mit Kontext bauen, den die JSON-Datei nicht hat. Einordnung: Quelle ohne PDF → Belegstelle mit *unverankertem* Anker, nicht prüfbar; Key nicht in Zotero → Prüfliste mit „in Zotero anlegen, dann `bib import` erneut ausführen" (der Import ist idempotent, ein zweiter Lauf legt nichts doppelt an); gefunden → *deterministisch geprüft*; mehrere Treffer → Prüfliste mit Kandidaten; nicht gefunden mit `validated` → *von dir bestätigt, nicht maschinell verankert*; nicht gefunden ohne `validated` → Prüfliste. Das Label wird zur Verwendung, bei passender Überschrift mit Verweis auf das Kapitel. Vorhandene `#quote`-Stellen werden über ihren Wortlaut mit den Belegstellen verknüpft.

Am Ende ein Bericht mit Zahlen je Kategorie. **Nichts wird still verworfen.** Optional übernimmt `bib import triage --from-dirs literatur/` die Ebenen `A_kern`, `B_belege`, `C_rest` als Tags.

**Abnahme:** das Seminarprojekt als schreibgeschützte Kopie, **erst nach dem 23.09.2026** (laufende benotete Arbeit mit eigenen KI-Regeln). Oracle ist `check_quotes.py`: `bib check` muss dieselben Zitate durchwinken und dieselben bemängeln. Bis dahin wird gegen ein eigenes Testprojekt mit frei zugänglichen Papers entwickelt.

---

## 12. Verantwortungsvolle KI-Nutzung

Verbindliche Prinzipien für alle Teilprojekte:

1. **Kein Modelltext im Dokument.** LLM-Funktionen recherchieren, ordnen und prüfen. In eine `.typ`-Datei gelangt nur Wortlaut aus Quellen. Es gibt keinen Codepfad, der das umgeht.
2. **Herkunft an jedem Eintrag** (siehe Abschnitt 5). Ein LLM-Urteil wird nie automatisch wahr, sondern zeigt die Belegstelle und wartet auf Bestätigung.
3. **Keine erfundenen Quellen.** Vorschläge stammen ausschließlich aus der eigenen Bibliothek, immer mit verankerter Stelle.
4. **Reproduzierbarkeit:** Zu jedem LLM-Aufruf werden Modell, Version, Parameter, Prompt und Eingabe-Prüfsumme gespeichert, dazu die Prüfsumme der geltenden `[ai]`-Richtlinie.
5. **Dokumentationspflicht automatisieren:** vollständiges Protokoll mit Tool, Version, Datum, URL, Prompt, Ergebnis und Art der Nutzung; daraus wird eine Anhangstabelle deterministisch erzeugt, nicht formuliert.
6. **Richtlinie pro Projekt** im Abschnitt `[ai]` der `bib.toml`, vom Tool durchgesetzt.
7. **Datenabfluss sichtbar machen:** Bei Cloud-Aufrufen wird festgehalten, welcher eigene Text das Gerät verlassen hat.
8. **Lesen nicht wegautomatisieren:** Hinweise wie „zitiert, aber nie gelesen" sind erwünscht, Skimming-Hilfen bleiben Lesehilfe.

**In dieser Version umgesetzt:** Herkunftsfelder (2), die Tabelle `llm_call` (leer, aber verbindlich für spätere Funktionen), der Abschnitt `[ai]` in `bib.toml`. Prinzip 1 gilt trivial, weil kein LLM enthalten ist.

---

## 13. Fehlerbehandlung

Grundsatz: nichts still verwerfen, alles Unklare in die Prüfliste, `bib review` ist der einzige Ausgang.

| Problem | Verhalten |
|---|---|
| Zotero läuft nicht | letzter Stand, Alter sichtbar |
| PDF fehlt | Quelle zitierbar, Prüfung meldet „kein PDF" |
| Extraktion scheitert an einer Seite | Seite markiert, Rest nutzbar |
| Backend stürzt ab (`pdf-extract` bricht bei kaputten Dateien hart ab) | isoliert ausgeführt, Fehlschlag am Anhang vermerkt |
| Fehler im Language Server | kein Absturz: Hintergrundarbeit, Fehler als Meldung, Zeitlimits für Zotero-Anfragen |

Datenbank: WAL für gleichzeitigen Zugriff von CLI und Server, versionierte Migrationen, automatische Sicherung vor jeder Migration.

---

## 14. Tests

- **Einheitentests** für Normalisierung (inklusive Idempotenz), Ankersuche gegen absichtlich veränderte Texte, Typst-Parser über alle vier Zitatformen samt Escapes und dynamischem Fall.
- **Beispiel-PDFs** nicht im Repo, sondern per Skript mit festen Prüfsummen geladen: altes LaTeX ohne Unicode-Zuordnung, modernes zweispaltiges Paper, gescanntes Dokument, Tabellen.
- **Backend-Benchmark** als eigenes Werkzeug mit arXiv-LaTeX-Quellen als Referenz; wiederholbar nach Updates.
- **Language-Server-Tests** gegen ein Beispielprojekt mit Skript-Client.
- **Abnahme** gegen das Seminarprojekt (siehe Abschnitt 11).

Entwickelt wird testgetrieben.

---

## 15. Reihenfolge

| Schritt | Ergebnis | Nutzbar |
|---|---|---|
| 0a | Backend-Benchmark | entscheidet Abschnitt 7 |
| 0b | Zed-Test: Minimal-Language-Server neben tinymist, prüft Hover, References, Definition, Diagnostics | bestätigt oder korrigiert Abschnitt 9 |
| 1 | Datenmodell, Migrationen | `bib init`, `bib doctor` |
| 2 | Zotero-Abgleich, `.bib` lesen | `bib sync` |
| 3 | Extraktion, Normalisierung | `bib index`, `bib text`, `bib search` |
| 4 | Anker, Belegstellen, Korrekturen | `bib excerpt`, `bib review`, `bib patch` |
| 5 | Typst-Parser | `bib cites`, `bib check` |
| 6 | Import | 143 Belegstellen übernommen |
| 7 | Language Server | Arbeit in Zed |
| 8 | Zed-Extension, Export | erste Version fertig |

Jeder Schritt ist für sich nutzbar; ab Schritt 3 ersetzt das Tool bereits Teile des Alltags.

---

## 16. Future Work

**Abbildungen, Diagramme und Tabellen.** Weitgehend additiv: `block` (Layout-Bereiche), `media` (Bilder), abgeleitete Artefakte (Tabellenzellen, Plot-Werte). Vorgesehen ist dafür bereits: Anker mit Seitenbereich, Verwendung am Anker statt am Textausschnitt, Seitengeometrie mit Größe, Drehung und Koordinatensystem. Eine andere Serialisierung von Tabellen verschiebt Offsets — dagegen schützen versionierte Textschichten und Anker über Wortlaut plus Kontext.

**Leseoberfläche.** Eigener PDF-Leser mit CiteSee-Färbung (Zitate markiert nach *in Bibliothek / Triage-Ebene / selbst zitiert / ungelesen / fehlt*), CiteRead-Randnotizen (was zitierende Papers über eine Stelle sagen) und Anzeige von Ankern, Belegstellen und Notizen. Setzt Zitatmarker aus GROBID und Zitatkontexte aus Teilprojekt 5 voraus.

**Fremde oder geteilte `.bib` ohne Zotero.** Bewusst nicht modelliert. Wird es nötig (etwa bei gemeinsamen Papers mit häufig aktualisierter, von Hand gepflegter `.bib`), kommt es als eigenes Thema zurück; bis dahin ist der Weg eine eigene Zotero-Bibliothek oder eine Gruppenbibliothek.

**Öffentliche Bibliographie als interaktive Webseite.** Eigene Paper, ausgewählte Notizen und Verknüpfungen als statisch erzeugte, durchsuchbare Seite mit Graph-Ansicht. Voraussetzungen fürs Datenmodell: Sichtbarkeit pro Notiz und Link (Standard *privat*, Veröffentlichung nur ausdrücklich), und beim Export keine Volltexte oder Ausschnitte über das Zitatrecht hinaus. Setzt Teilprojekt 4 (Notizen) und 5 (Graph) voraus.

**Weitere Teilprojekte:** Notizen und Discourse Graph (4); hybride Suche aus BM25 und Embeddings, SPECTER2 auf Paper-Ebene (3); Zitationsgraph, Zitatkontexte zitierender Papers, Co-Autoren, Retraction-Check, Metadaten-Lint (5); Zitat-Prüfung inhaltlich per LLM, Extraktions-Matrix (6); Zurückschreiben nach Zotero, MCP-Server, Erfassung der Claude-Sessions eines Projekts (7).

---

## 17. Recherchegrundlage

Eigene Messungen (September 2026):

| Prüfung | Ergebnis |
|---|---|
| ε in Dwork 2006 (altes LaTeX ohne Unicode-Zuordnung) | pdftotext 0, pdfplumber 0, pdfium 0, `pdf_oxide` 0 (Zeichen fällt weg), **`mupdf-rs` 28, `pdf-extract` 28** |
| ε in Abadi 2016 (modern) | alle Backends 92. *Vorbehalt:* `file` meldete für den Download 2 Seiten; bei rund 11.500 extrahierten Wörtern ist eher die Seitenzählung von `file` falsch. Nicht nachgeprüft, der Benchmark nutzt frisch geladene Dateien. |
| NFKC auf ε-Varianten | `ϵ` (U+03F5), `𝜖` (U+1D716), `𝜀` (U+1D700) werden alle zu `ε` (U+03B5), `ﬀ` zu `ff` |
| Übereinstimmung der Extraktoren untereinander | 64–89 % bei zufälligen 8-Wort-Ausschnitten, ohne Referenz keine Aussage über Richtigkeit → Benchmark nötig |
| Semantic Scholar zur Hauptquelle des Seminars | 79 zitierende Papers, 41 mit Zitat-Satz, 3 „influential", **0 mit Zitationsabsicht** → Absichten sind zu dünn für Teilprojekt 5 |
| Zotero 9.0.1 (Snap), lokale API, nur lesend abgefragt | erreichbar, 168 Haupteinträge; `citationKey` als natives Feld gefüllt; **`version` überall 0, `Last-Modified-Version: 0`, kein `/deleted`-Endpunkt** → kein inkrementeller Abgleich; PDF-Pfad über Link `enclosure` (`file://…/Zotero/storage/<key>/…`); 274 Annotationen mit Text, Kommentar, Position und Seitenlabel lesbar; 14 Collections. Schreiben geht nur über die Web-API. |
| Gruppenbibliotheken über die lokale API | `/api/users/0/groups` liefert 3 Gruppen, `/api/groups/<id>/items` funktioniert inklusive Anhängen und Annotationen (81 + 41 Haupteinträge, eine Gruppe leer). **17 Citation Keys kommen in mehr als einer Bibliothek vor** (16 zwischen eigener und einer Gruppenbibliothek). Ein Gruppeneintrag ohne Citation Key. |
| Benchmark (Schritt 0a): 7 Dokumente, 21 Sätze, 7 Reihenfolge-Paare, `bench/results/2026-09-17.md` | Sätze: `pdf_oxide` 20/21, `pdf-extract` 21/21, `mutool` 21/21, pdfium 18/21; Reihenfolge überall 7/7. `pdf_oxide` fehlt ε in 2 Dokumenten (Dwork 2006, Shokri 2017), `pdf-extract` liefert es dort; `mutool` 191 × U+FFFD, pdfium 1128 Steuerzeichen (in den nachgezählten Dokumenten devlin2019 und abadi2016 überwiegend U+0002 als Trennmarke am Zeilenende, 527 der dort gezählten 573). → **Standard `pdf_oxide`, Kaskade `pdf-extract`** (Regel 1) |
| Zed with two language servers (step 0b), `docs/research/14-zed-zwei-language-server.md` | Both servers start without settings change; diagnostics, hover and completion are merged; at `@key` tinymist provides no hover, definition, references or completion; merging of definition/references untested (tinymist had no results) |

Fremde Quellen, die das Design geprägt haben: Jergas & Baethge (Zitatfehlerquote rund 25 %, Update 2025 ohne Verbesserung) als Begründung der Prüfung; W3C Web Annotation und Hypothesis für robuste Anker; PaperMage für das Schichtenmodell (Forschungsprototyp, seit 11/2024 ohne Pflege, nutzt pdfplumber und hätte das ε-Problem geerbt); CiteSee, CiteRead, Scim, ScholarPhi, Threddy, Synergi als Ideengeber für spätere Teilprojekte; SemanticCite für die vier Prüfstufen; Discourse Graphs für das Notizmodell.

---

## 18. Offene Punkte

- Öffnet Zed einen externen Link (`zotero://…`), den der Language Server über `window/showDocument` schickt? Falls nicht, übernimmt das CLI.
- Unterstützt die lokale Zotero-API `sort=dateModified`? Die Abfrage lieferte als „neueste" Einträge solche vom Juni, obwohl im September Einträge hinzugekommen sind. Für den Vollabgleich unerheblich, für eine spätere Optimierung zu klären.
- Beitrag an `pdf_oxide` für Glyphennamen aus eingebetteten Type1-Schriften: wünschenswert, nicht eingeplant.
- Der Verdachtsprüfer aus §7 Schritt 2 erkennt nur Zeichen-Signale (Steuerzeichen, `(cid:…)`, kein Text). Der einzige verfehlte Satz von `pdf_oxide` im Benchmark (20/21: vaswani2017, Überschrift „Abstract“ mitten im Absatz statt davor) ist ein Reihenfolgefehler, den er nicht erkennen kann. Ob die separate Schrift-Heuristik (Mathe-Schrift wie CMMI ohne ein einziges griechisches Zeichen) auf den Seiten anschlägt, auf denen `pdf_oxide` ε stillschweigend verliert (dwork2006, shokri2017: 0 Steuerzeichen, 0 `(cid:`, 0 U+FFFD), ist ungemessen → wird in Plan 2 geklärt. Siehe `bench/results/2026-09-17.md`.
- Testkorpus: Jede Korpus-PDF hat MediaBox-Ursprung (0,0), keine eigene CropBox und kein `/Rotate`; die Kreuz-Backend-Koordinatentests können deshalb eine versetzte MediaBox, eine von der MediaBox abweichende CropBox oder gedrehte Seiten nicht erkennen (siehe Kommentare in `crates/bib-extract/src/backends/pdfium.rs` und `stext.rs`). Plan 2 braucht eine synthetische Testdatei mit versetzter MediaBox, eigener CropBox und `/Rotate 90` sowie eine Entscheidung, ob Spans in gedrehtem oder ungedrehtem Seitenraum gemeldet werden.
