# Zotero sync smoke test (plan 2, task 9)

Date: 2026-09-21. Zotero 9.0.1, `bib` at commit 1a51d92. Read-only; data
directory was a temporary directory and has been deleted.

| Measure | Value |
|---|---|
| Libraries (user + groups) | 4 (3 groups) |
| Active sources per library | user 161, groups 41, 77, 0 |
| Top-level items skipped (notes, standalone attachments) | 11 |
| Sources without citation key | 0 |
| PDF attachments: with local file / without file / file missing | 207 / 1 / 1 |
| Citation keys in more than one library | 17 (spec §17 measured 17) |
| Of those, different works (`key_conflict`) | 0 |
| First sync duration | 0.34 s |
| Second sync: new / changed / retired | 0 / 0 / 0 |

## Observations

- `sqlite3` was not installed on this machine. All database counts were collected with a short
  Python snippet using the standard-library `sqlite3` module against the throwaway database at
  `$BIB_DATA_DIR/bib.db` only; the real Zotero database was never opened.
- The second sync was a clean no-op for every one of the four libraries: 0 new, 0 changed,
  0 retired and 0 reactivated sources each. No evidence of a Zotero-side change between the two
  runs was observed.
- `bib doctor` exited 1, as expected once real data is present. Findings by code: `database`
  (info, 1), `zotero` (info, 1), `library` (info, 4), `pdf_without_file` (info, 1),
  `missing_pdf_file` (warning, 1). No `missing_citation_key`, `never_synced`, or `review_open`
  findings occurred. The one warning is a synced attachment whose path no longer resolves to a
  file on disk; the one info is an attachment with no local copy in Zotero at all — both are
  legitimate states of a real library, not sync defects.
- The project-level `bib doctor` run, with all four libraries listed in one project's `bib.toml`
  (so `conflicts()` evaluated every one of the 17 shared keys, not a subset), produced the same
  finding codes plus `project` (info, 1) and **no** `key_conflict` findings: for every one of the
  17 keys, all sources sharing it pairwise satisfy the §6 same-work rule (DOI match, or normalized
  title and year match) — none are conflicts.
- The measured count of citation keys shared across libraries (17) matches the count already
  recorded in spec §17 exactly.
- Spec §17 records **top-level entries** per library (168 for the user library; 81 + 41 for the
  two non-empty groups, one group empty), which include the notes and standalone attachments this
  sync's "active sources" figure excludes. Pairing each library's active-source count with its own
  skipped-top-level count (identified by which library it belongs to, not by position in any row):
  the user library, 161 active + 7 skipped = 168; the group with **77** active sources, + 4
  skipped = 81; the group with **41** active sources, + 0 skipped = 41; the empty group, 0 active +
  0 skipped = 0 (all skipped counts sum to 11, the table's total). All four reproduce §17's figures
  exactly. The apparent 77-vs-81 difference in the table above is fully accounted for by this
  definitional difference between the two documents' measures; there is no evidence of any actual
  change to that group's contents.
- **Difference from spec §17:** §17 records "one group entry without a citation key," but this
  run measured 0 active sources without a citation key (table above). The cause was not
  determined. One possibility, not verified: that entry is now among the skipped top-level items,
  or has since been retired, either of which would remove it from the "active sources without a
  citation key" count without being a sync problem — but this has not been checked and is not
  established.
