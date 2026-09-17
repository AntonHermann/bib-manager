# Hardware und lokale LLMs

Gemessen am 2026-09-14 auf dem Entwicklungs-Laptop.

## Ausstattung

| Komponente | Wert |
|---|---|
| CPU | AMD Ryzen 7 PRO 5850U, 8 Kerne / 16 Threads |
| GPU | nur integriert: AMD Radeon Vega (Cezanne) |
| VRAM | 1,0 GiB fest (`mem_info_vram_total`) + bis 15,1 GiB dynamisch aus dem RAM (`mem_info_gtt_total`) |
| RAM | 30 GiB, zum Messzeitpunkt 24 GiB belegt, 8 GiB Swap |
| Treiber | Vulkan-ICDs vorhanden (`radeon_icd.json` = RADV) → llama.cpp mit Vulkan-Backend möglich |
| ROCm | nicht installiert; Cezanne offiziell nicht unterstützt |
| LLM-Laufzeiten | keine installiert (weder Ollama noch llama.cpp) |
| Kernel | Linux 6.17 |

## Was lokal realistisch ist (Schätzung, nicht gemessen)

Engpass: DDR4-Speicherbandbreite für die Generierung; CPU/iGPU für das Einlesen langer Eingaben.

| Aufgabe | Lokal? | Einschätzung |
|---|---|---|
| Embeddings, Suchindex, Reranking | ✅ | kleine Modelle; Index für einige Hundert Papers in Minuten |
| Zitat-Prüfung Vorstufe (Stelle finden, grob einordnen) | ✅ | SemanticCite zeigt, dass kleine spezialisierte Modelle reichen |
| LLM 3–4B | ✅ interaktiv | grob 10–20 Tokens/s |
| LLM 7–8B oder MoE mit ~3B aktiven Parametern | ⚠️ Hintergrund | lange Eingaben dauern; MoE braucht ~17 GB RAM, bei aktueller Belegung knapp |
| Extraktions-Matrix über viele Papers, lange Zusammenfassungen | ☁️ | Cloud oder lokaler Nachtlauf |
| olmOCR lokal | ❌ | braucht NVIDIA ≥ 12 GB VRAM |

## Muster, das daraus folgt

- Alles Interaktive (Hover, Suche, Vorschläge) ohne großes LLM, nur Index und Embeddings.
- LLM-Aufgaben in einer Warteschlange, lokal im Hintergrund abgearbeitet.
- Cloud bewusst pro Aufgabe wählbar (Nutzerwunsch: „c), aber je mehr lokal geht, desto besser").
- Echte Geschwindigkeit später mit kurzem Benchmark messen.
