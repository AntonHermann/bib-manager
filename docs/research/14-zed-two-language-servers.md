# Zed with two Typst language servers (step 0b)

Date: 2026-09-17. Zed 1.20.2 (7c451e69), tinymist 0.15.8 (from the Zed `typst` extension), Linux/Wayland.

Setup: throwaway server `spikes/lsp-coexist` (installed via `cargo install`), loaded through the dev extension
`spikes/zed-coexist-ext`; test document `spikes/typst-sample/main.typ`. Every response of the spike is labelled
"bib-spike". (During the test the labels were German; they were translated afterwards and are quoted here in
their current English form.) The tests were performed by the user in the Zed UI; server start-up was checked in `Zed.log`.

**No settings change was needed.** After installing the dev extension, Zed started both servers for the Typst
buffer (`lsp-coexist` at 14:25:28, tinymist at 14:25:29). While loading the extension the log contained
`ERROR [extension_host] invalid type: map, expected a string at line 10 column 21` without naming a file; it did
not prevent the server from starting.

## Round 1: both servers, on the citation `@dwork2006`

| Function | tinymist visible | bib-spike visible | Observation |
|---|---|---|---|
| Diagnostics | yes (error for `@unknown2020`) | yes (HINT "bib-spike sees @…") | both servers' diagnostics are shown |
| Hover | no | yes | bib-spike hover plus the bib-spike diagnostic |
| Go to Definition | no | yes | jumps to the start of the file (the spike's fixed target) |
| Find All References | no | yes | "References to @dwork2006": both occurrences, no duplicates; nothing when the cursor is not on a key |
| Code Actions | no | yes | "bib-spike: action for dwork2006" |
| Completion after `@` | no | yes | only `bibspike2026` |

## Round 2: control, tinymist only

bib-spike was disabled with `"languages": {"Typst": {"language_servers": ["tinymist", "!bib-spike", "..."]}}`
(the log confirms `stopping language server bib-spike`). A heading `== Label target <intro>` with
`#set heading(numbering: "1.")` and a reference `@intro` were added to the sample.

| Function | tinymist answers? |
|---|---|
| Hover on `@dwork2006` | no |
| Go to Definition on `@dwork2006` | no |
| Completion after `@` | no |
| Hover on `bibliography` (function) | yes (function docs) |
| Hover on `@intro` | no |
| Go to Definition on `@intro` | no |
| Find All References on `@intro` | no |
| Completion after `#bib` | yes (`bibliography`) |

Round 1 therefore did not test merging: tinymist had nothing to contribute at citations.

## Round 3: merging where both servers answer

The spike was changed to answer hover at every position ("bib-spike hover (no key here)"); its completion already
answers everywhere. Settings restored to the original (both servers active), language servers restarted.

| Function | Result |
|---|---|
| Hover on `bibliography` | **both**, bib-spike first, tinymist below |
| Completion after `#bib` | **both** (`bibspike2026` and `bibliography`), bib-spike's item first |

## Conclusions for spec §9

- Diagnostics, hover and completion from a second server are merged with tinymist's. The order observed was
  bib-spike first; bib-spike was also started first. Whether the order follows start order or registration order
  was not tested.
- Code actions, go to definition and find all references of the second server work. Whether Zed merges them with
  tinymist's results is **not tested**, because tinymist returned nothing at any position tried.
- At `@key` citations tinymist (in this setup) offers no hover, definition, references or completion, so the
  planned editor features of the bibliography server do not compete with tinymist there. Caveat: the sample is a
  single file without a pinned main file; tinymist may offer more in a configured project.
- tinymist reports unknown citation keys itself, which confirms that our "unknown key" diagnostic should be
  switchable (already in §9).

## Cleanup

`cargo uninstall lsp-coexist` done; `~/.config/zed/settings.json` restored byte-identical to the backup taken
before the test; the dev extension is uninstalled by the user in Zed.
