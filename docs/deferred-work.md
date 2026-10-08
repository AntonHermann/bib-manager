# Deferred work

Initially recorded on 2026-09-21, after plan 2 (steps 1–2: data model and Zotero
sync); updated with spec follow-ups during the documentation review.

Unless identified below as a spec follow-up, items here were raised by a code review
during plan 2, judged real, and consciously deferred —
either because it cannot produce a wrong answer today, or because the plan that would naturally fix
it has not been written yet. None of them block the current code.

Two kinds of entry:

- **Behaviour** — the tool does something a user could notice. These are the ones worth draining.
- **Coverage / hygiene** — a code path with no test, or a readability trap. No wrong answer is
  possible today; the risk is that a future change breaks something silently.

Items resolved during plan 2 are not listed. The `sync_all` short-circuit and the stale
`#[allow(dead_code)]` on `resolve.rs`'s test-only `OTHER` constant were both fixed before the merge.

## Spec follow-ups

These known implementation and coverage gaps were previously tracked in spec §18.
Moving them here does not expand the implementation scope or claim they are done.

### Offline Zotero database fallback

The read-only `zotero.sqlite` fallback in
[spec §6](superpowers/specs/2026-09-16-library-core-typst-design.md#6-zotero-sync)
was deferred from plan 2. When Zotero is unavailable, the current tool uses its last
synced state rather than reading Zotero's database directly.

### Cited keys missing from the bibliography

The "key only in Zotero, citation won't compile" case from spec §6 needs the keys
actually cited in Typst. It remains scheduled with the Typst parser (step 5);
the current `.bib` comparison cannot determine this from library contents alone.

### Project persistence and duplicate project IDs

The steps 1–2 plan deferred the `project` table and detection of the same project
ID at multiple paths to the Typst parser plan. Currently `bib.toml` is read from
disk only. Implement the persistence and moved-versus-copied review behavior
described in [spec §10](superpowers/specs/2026-09-16-library-core-typst-design.md#10-cli)
with that feature; neither is provided by the existing configuration reader.

### Extraction coordinate coverage

The current corpus has MediaBox origins of (0,0), no separate CropBox, and no
`/Rotate`, so coordinate comparisons do not cover offsets, cropping, or rotation.
Add synthetic coverage with an offset MediaBox, a distinct CropBox, and `/Rotate 90`
when extending extraction. The rotated-versus-unrotated coordinate convention is
still a [design question in spec §18](superpowers/specs/2026-09-16-library-core-typst-design.md#18-open-questions);
do not treat existing tests as evidence for either convention.

## Behaviour

### `bib doctor` exits 1 forever once anything is queued

`doctor.rs` — `review_open` counts open review-queue entries, and nothing in this codebase can
resolve one. Per spec §18, `bib review` arrives in subproject 4. Until then a user who retires a
single source has a permanently non-zero `bib doctor`. Not a defect — a sequencing consequence, and
the reason it is written down here rather than fixed.

Related, and cheap whenever that is touched: the finding reports only a count. Listing the kinds
(`2 open review item(s): source_retired, key_conflict`) would make the line actionable today.

### `bib doctor --json` shape is not uniform

`crates/bib-cli/src/main.rs` — `doctor` prints a bare JSON array while `init`, `sync` and `backup`
print objects, and the error path prints `{"error": …}`. A consumer cannot tell findings from an
error by shape alone. `{"findings": [...]}` would make the CLI's JSON contract consistent.

### An invalid `bib.toml` and an unreadable `.bib` are handled asymmetrically

`main.rs` vs `doctor.rs` — an unreadable or invalid `bib.toml` makes `bib doctor` exit 2 printing
nothing else, while an unreadable `.bib` becomes a `bib_unreadable` Error *finding*. The doctor is
the tool you reach for when the project is broken, so the two project files it exists to check
should behave the same way. A `project_invalid` Error finding would be consistent. Note that
`crates/bib-cli/tests/cli.rs` currently pins the exit-2 behaviour.

### `zotero_unreachable` is emitted for any `groups()` failure

`main.rs`, `doctor.rs` — including `ZoteroError::Http` and `ZoteroError::Parse`. The message text is
accurate in each case; the machine-readable code is not.

### A sync killed mid-run leaves an invisible `running` row

`doctor.rs` — `sync_run.status='running'` with `finished_at` and `error` both NULL persists forever,
and the doctor reads only `error`, so it says nothing. A stale `running` row deserves at least an
Info finding.

### `bib_only`'s advice can be wrong

`doctor.rs` — the message says "add it in Zotero", but the finding also fires when the key *is* in
Zotero, in a library not listed in the project's `[zotero] libraries`. A `library_unknown` finding
may or may not accompany it. (The conflicted-key case of this was fixed in plan 2; this is the
remaining one.)

### Text and JSON output carry different information

`main.rs` — `SyncCounts.skipped_top_level` is serialized into `--json` but never printed in the text
report.

### An item with no `itemType` becomes a source with an empty type

`crates/bib-core/src/zotero/model.rs` — an item whose `data.itemType` is absent parses to
`item_type: ""`, and `is_source()` accepts it. Harmless against real Zotero data today; a guard in
`parse_item` is cheaper now than diagnosing it later.

## Design traps

### `keyed_sources` and `conflicts` can disagree about the same key

`crates/bib-core/src/resolve.rs` — `keyed_sources` returns the rank-first row for a key, without
consulting `same_work`; `conflicts`/`resolve_key` classify that same key as `Conflict`. Two public
functions of one module therefore give contradictory answers.

Nothing is wrong today: `doctor.rs` is the only caller and compensates correctly. But plan 2 spent
two `doctor` fix rounds on exactly this trap, which is evidence the caller-discipline contract is
hard to hold. The hardening is one line — have `keyed_sources` drop groups that `decide` returns
`Conflict` for. Do it when a second caller appears. (`doctor`'s own filter is still needed for the
`.bib` side.)

### `doctor.rs` is the file to split first

682 lines, with `run()` inlining eight checks. Not a defect at this size. It is where to start when
the next plan adds checks.

## Coverage and hygiene

Grouped by module. None of these can produce a wrong answer today.

**`db`**
- No test for `open()` against a corrupt database file. SQLite's own error surfaces with file
  context, so no wrong answer is possible.
- `db/mod.rs`: a local `backup` binding shadows the imported `backup` function. Correct as written —
  the shadow starts after the call — but a readability trap; rename when the code is next touched.
- `TEST_MIGRATIONS` is duplicated verbatim between `migrate.rs` and `mod.rs` tests. Four lines of
  fixture.

**`project`**
- No test for `find_project_root` starting at the filesystem root. `Path::ancestors` terminates, so
  this is correct by construction.
- Duplicate entries in `[zotero] libraries` are not deduplicated. `rank` uses `position()`, so a
  duplicate is a no-op after the first.

**`zotero` client and fake**
- No client-level test for `ZoteroError::Parse` from a malformed HTTP body: the model layer's parse
  errors are tested, but the client's own parse-error wiring is not.
- The `Total-Results`-header-missing fallback in `get_all` is never exercised — the fake always sends
  the header, and real Zotero always does too.
- `from_env()` has no dedicated unit test.
- `fake.rs`: the acceptor thread is never joined, so threads live for the test process. Harmless —
  they die with it.
- `fake.rs`: `serve()` sets no read timeout, so a client that opens a connection and never completes
  a request line would hang one worker thread. Not reachable from any current test.

**`sync`**
- The `dedupe_key` expectation is built with the same `Display` impl under test, so it is mildly
  tautological. A literal string would be more resilient.
- The review-queue assertion queries on `resolved_at IS NULL` only, not on `source_id`. Safe in a
  single-source fixture.
- No test for a fully empty library (zero items, attachments and collections).
- `upsert_library` is called twice per library sync — once before the fetch to obtain the id, once
  inside the transaction to persist the fetched name. Both calls are needed; the call site deserves
  a one-line comment saying why.

**`resolve` and `bibfile`**
- The DOI empty-guard has no dedicated test; it is exercised indirectly.
- `parse_bib` surfaces biblatex's raw byte-offset message (e.g. `unexpected end of file: 29-29`),
  which is informative about the failure kind but not anchored to a file and line. That message is
  biblatex's, not ours.
- No test for "field absent in `.bib`, present in Zotero" for `doi` or `year`. The `(Some, Some)`
  destructure makes the behaviour correct by construction.

**`doctor`**
- No test for a project with no `[bibliography]` section. Covered end to end by the CLI tests.
- Several tests assert a finding's code and severity but not its message text. The load-bearing
  strings are asserted; the rest are display text.

**`bib-cli`**
- `init()`'s JSON `project_file` uses `(!no_project).then_some(&config_path)`. Correct, and the
  end-to-end test pins the resulting JSON, but an indirect way to express "path if we are managing a
  project".

## Documentation

- `docs/research/15-zotero-sync-smoke-test.md` and the new spec §17 row list the two non-empty
  groups smaller-first, while the older §17 sentence lists them larger-first. Each document is
  unambiguous on its own terms — every group is identified by its count, not by position — but the
  two conventions differ.
- Spec §6's "Fallback" bullet now reads "…the last known state, whose age is shown everywhere.
  Deferred (plan 2): until then, the tool works from the last synced state." Mildly redundant; worth
  tidying in the next spec pass.
- Formatting leftovers from the plan-1 reformat: one match arm in `normalize.rs` contracted to a
  118-character line, and a few `assert_eq!` struct literals now span 8–10 lines.
