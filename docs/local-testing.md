# Local test deployment

Use a dedicated installation alongside your normal `bib`, with project-scoped
environment settings supplied by [direnv](https://direnv.net/). These instructions
assume a Unix shell. See the README for [app requirements](../README.md#requirements);
`just` is optional, and direnv must be installed and
[hooked into your shell](https://direnv.net/docs/hook.html).

## Install from the app checkout

Run this in the `bib-manager` checkout containing the version you want to try:

```bash
just deploy-local
```

Or, without `just`:

```bash
cargo install --path crates/bib-cli --locked --force \
  --root "$HOME/.local/share/bib-seminar-test/install"
```

The common parent keeps the trial together without mixing Cargo installation
files with application data:

```text
~/.local/share/bib-seminar-test/
├── install/
│   └── bin/bib
└── data/
    ├── bib.db
    └── backups/
```

`data/` is created when you first run the configured app. Deployment only installs
the executable: it does not initialize a project, touch the database, sync Zotero,
or change your shell configuration.

To use another parent, pass an absolute path:

```bash
just local_root="$HOME/.local/share/another-bib-trial" deploy-local
```

Use that same parent in the `.envrc` below. To build without installing, use
`just build`, equivalent to `cargo build --release -p bib-cli --locked`.

## Activate in the seminar repo

In your seminar repository, create `.envrc` with the following content. If one
already exists, add these settings without replacing its other contents:

```bash
PATH_add "$HOME/.local/share/bib-seminar-test/install/bin"
export BIB_DATA_DIR="$HOME/.local/share/bib-seminar-test/data"
```

Keep this machine-local configuration out of Git by adding `/.envrc` to that
repository's `.git/info/exclude` (use `git rev-parse --git-path info/exclude`
to locate it in a Git worktree). This does not untrack an already tracked file;
follow the repository's existing policy if it shares an `.envrc`.

Review the file, then authorize it from the seminar repo:

```bash
direnv allow
```

Wait for your shell prompt to return so the hook applies the environment, then:

```bash
command -v bib
printf '%s\n' "$BIB_DATA_DIR"
```

These should point to `install/bin/bib` and `data` under the dedicated parent.
`PATH_add` prepends the directory, shadowing `~/.cargo/bin/bib` without removing
it. Direnv restores the previous environment when you leave the project.
A plain `.env` is not automatically loaded by this setup; use `.envrc` directly.

If the wrong executable is selected:

```bash
ls -l "$HOME/.local/share/bib-seminar-test/install/bin/bib"
direnv status
```

An absent executable means you need to deploy first. Check that the shell hook is
installed and `.envrc` is authorized. In Zsh, `rehash` clears cached command
locations and `whence -va bib` shows aliases, functions, and executable candidates.
Aliases or functions can override PATH lookup.

## Initialize and use the project

With Zotero running and its local API enabled, run in the seminar repo:

```bash
bib init
```

Edit the generated `bib.toml` to select your Zotero libraries and existing
bibliography file, following the README's
[`bib.toml` configuration](../README.md#bibtoml). Keep your existing bibliography
export workflow. You can commit `bib.toml`; the database stays outside the repo.

```bash
bib sync
bib doctor
```

Sync reads all Zotero libraries; the configured library order controls citation
key resolution, not what gets synced. After changes in Zotero, repeat those two
commands. Consult the README for [diagnostics](../README.md#what-bib-doctor-reports),
[exit codes](../README.md#commands), and
[current feature limitations](../README.md#what-is-not-built-yet).

## Update the trial

Before trying a newer version, run this in the seminar repo with its environment
active:

```bash
bib backup
```

Then run `just deploy-local` again from the desired `bib-manager` checkout
(with the same `local_root` override, if used). It rebuilds and replaces only the
test installation; the sibling data directory is preserved. The Cargo command
above is the equivalent update command without `just`.

Return to the seminar repo and verify `command -v bib` before using it.
Database migrations may run when the updated executable opens the database;
preserving the directory during deployment does not guarantee that an older
binary can read it afterward.
