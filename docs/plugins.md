# Bob Plugins

`bob plugins` manages Bryan's custom Bob Obsidian plugins from the
[`bbugyi200/bob-plugins`](https://github.com/bbugyi200/bob-plugins) repo, which
is the source of truth for the six plain-JavaScript community plugins. The repo
holds one folder per plugin under `plugins/<id>/`, each with a `manifest.json`,
a `main.js`, and an optional `styles.css`.

## Commands

```bash
bob plugins [-b|--bob-dir DIR] [-f|--format table|json] [-r|--repo DIR]
bob plugins list [-b|--bob-dir DIR] [-f|--format table|json] [-r|--repo DIR]
```

`list` is read-only. Running `bob plugins` with no subcommand runs `list` with
the same options. (`bob plugins sync`, which deploys the repo into the vault, is
documented separately.)

## Discovery

`list` reads the repo's `plugins/` directory and builds one row per plugin
folder. The plugin id, version, and description come from that folder's
`manifest.json`; an empty or absent manifest `id` falls back to the folder name.
A folder whose `manifest.json` is missing or unparseable is reported as an error
on stderr and the command exits non-zero, but the remaining plugins still list.

Two roots feed the report:

- **Repo root.** Resolves from `-r, --repo`, then the `BOB_PLUGINS_DIR`
  environment variable, then the default
  `~/projects/github/bbugyi200/bob-plugins`. Plugins live under
  `<repo>/plugins/<id>/`.
- **Vault root.** Resolves from `-b, --bob-dir`, then `BOB_DIR`, then `~/bob`.
  Installed plugins live under `<bob-dir>/.obsidian/plugins/<id>/`, and the
  enabled set is read from `<bob-dir>/.obsidian/community-plugins.json`.

## Columns

| Column        | Source                                                                 |
| ------------- | ---------------------------------------------------------------------- |
| `PLUGIN`      | manifest `id` (repo folder name when the manifest omits it)            |
| `VERSION`     | manifest `version`                                                     |
| `SYNC`        | repo files vs. vault files                                             |
| `VAULT`       | `community-plugins.json` plus the installed-folder check               |
| `DESCRIPTION` | manifest `description`, truncated to the remaining terminal width      |

### SYNC state

`SYNC` byte-compares the managed files — `manifest.json`, `main.js`, and
`styles.css` when the repo has one — against the vault copy:

- `synced` — every managed repo file is present and byte-identical in the vault.
- `drift` — the vault has the plugin folder, but at least one managed file is
  missing or differs.
- `missing` — the vault has no folder for this plugin.

Only the managed files are compared. Runtime files such as `data.json` are
never read.

### VAULT state

`VAULT` reports the plugin's enable state in the vault:

- `enabled` — the id is listed in `community-plugins.json`.
- `disabled` — the plugin folder exists in the vault but the id is not enabled.
- `not installed` — the vault has no folder for this plugin.

A missing or unreadable `community-plugins.json` is treated as "nothing
enabled" rather than an error, so installed plugins then read as `disabled`.

## Header and Footer

The header names the repo and the plugin count, such as
`Bob Plugins · 6 · /home/bryan/projects/github/bbugyi200/bob-plugins`. The
footer summarizes the sync states, such as
`6 synced · 0 drift · 0 not installed`. On a non-color or piped stream the
separator renders as `-` and the colored state glyphs are dropped.

## Exit Status

`list` exits `0` even when plugins drift or are not installed — those are
reportable states, not failures. It exits `1` only on a real error, such as an
unreadable repo `plugins/` directory or an unparseable manifest, and writes the
error to stderr.

## JSON Output

`-f, --format json` prints a single stable object for scripting:

```json
{
  "ok": true,
  "repo": "/home/bryan/projects/github/bbugyi200/bob-plugins",
  "bob_dir": "/home/bryan/bob",
  "count": 6,
  "synced": 6,
  "drift": 0,
  "not_installed": 0,
  "plugins": [
    {
      "id": "block-id-prompt",
      "version": "1.0.0",
      "description": "Prompt for a custom block ID when a wiki block link uses the ^^ marker.",
      "sync": "synced",
      "vault": "enabled"
    }
  ]
}
```

The `sync` field is `synced`, `drift`, or `missing`; the `vault` field is
`enabled`, `disabled`, or `not_installed`. On error, JSON mode prints
`{"ok": false, "error": "..."}` instead.

## Examples

```bash
bob plugins
bob plugins list
bob plugins list -f json
bob plugins list -b ~/bob -r ~/projects/github/bbugyi200/bob-plugins
```
