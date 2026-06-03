# bob dataview

`bob dataview` runs Dataview queries through a running desktop Obsidian app by
default. The Obsidian vault must be open and the Dataview community plugin must
be enabled before running live smoke tests.

## Manual Smoke Test

Use a disposable query first:

```bash
bob dataview --source '#project'
bob dataview --format json --query 'LIST FROM #project'
bob dataview --format markdown --origin Home.md --query 'TABLE file.link FROM #project'
```

To include vault freshness, install/configure `ob` and run:

```bash
bob dataview --sync --format paths --source '#waiting'
bob dataview --sync --format json --query 'LIST FROM #waiting'
```

The `--sync` path runs `ob sync --path <bob-dir>` and `ob sync-status --path
<bob-dir>` before querying. Sync logs are written to stderr so stdout remains
scriptable paths, JSON, or rendered Markdown only. `--bob-dir` defaults to
`BOB_DIR` or `~/bob`; pass `--bob-dir <path>` when testing a fixture vault.

`--origin` must be a vault-relative note path. It is required when a DQL query
depends on relative links or Dataview's `this` context.
