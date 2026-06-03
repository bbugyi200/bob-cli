---
create_time: 2026-06-03
status: research
topic: Running Obsidian Dataview queries from the command line
---
# Research: Running Dataview Queries from the Command Line

## Question

Bob wants to run [Dataview](https://blacksmithgu.github.io/obsidian-dataview/) queries
from the command line and get back **all notes that match a particular query** (e.g. for
scripting, automation, or feeding results to other tools/agents). Is this possible, and
what is the best way to implement it?

## Current Vault State (observed)

From `/home/bryan/bob/.obsidian/community-plugins.json` and `.obsidian/plugins/`:

- `dataview` **is** installed and enabled.
- The following are also enabled: `obsidian-tasks-plugin`, `templater-obsidian`,
  `quickadd`, `task-status-cycler`, `mrj-jump-to-link`, `bob-navigation-hotkeys`,
  `bob-ledger-tools`, `block-id-prompt`, `obsidian-relative-line-numbers`,
  `note-refactor-obsidian`.
- The **Local REST API** plugin (`coddingtonbear/obsidian-local-rest-api`) is **NOT yet
  installed**. It is the key dependency for the recommended approach below.

So Bob's notes already carry Dataview metadata (frontmatter + `key:: value` inline fields),
and the query language is already in daily use inside Obsidian.

## The Core Constraint

**Dataview has no official, native CLI.** This is a long-standing, explicitly-acknowledged
gap, not an oversight we can quickly route around:

- The maintainer has said CLI extraction is "on my priority list" but it is **not
  implemented** (request open since 2021, still requested as of mid-2025).
- The blocker is architectural: Dataview's index is built on **Obsidian's APIs** — most
  importantly Obsidian's `CachedMetadata` database. Core functions like `parsePage` expect
  items handed to them *by Obsidian*. Dataview cannot currently build its index from raw
  markdown on disk without the Obsidian app running.
- Dataview publishes TypeScript typings to npm (`obsidian-dataview`), but these are for
  **plugin development inside Obsidian**, not for standalone Node use.

Consequently every viable option falls into one of two buckets:

1. **Use the *real* Dataview engine** — which means driving a *running* Obsidian instance
   from the outside (highest query fidelity).
2. **Use a *reimplementation* of DQL** that reads markdown directly off disk — no Obsidian
   needed, but only a subset of DQL is supported (lower fidelity, fully headless).

## Options

### Option A — Local REST API plugin + `curl` (real Dataview engine) ⭐ recommended

`coddingtonbear/obsidian-local-rest-api` exposes a secure REST API over the running vault.
Its `POST /search/` endpoint accepts a **Dataview DQL `TABLE` query** directly when you set
the content type:

```
Content-Type: application/vnd.olrapi.dataview.dql+txt
```

Because the query runs inside Obsidian, it uses the **actual Dataview index** — exact DQL
semantics, all implicit fields (`file.name`, `file.path`, `file.mtime`, `file.tags`, …),
inline fields, and resolved links all behave exactly as they do in the app.

Example (HTTPS on the default port, self-signed cert ⇒ `-k`):

```bash
curl -sk -X POST \
  -H "Authorization: Bearer $OBSIDIAN_API_KEY" \
  -H "Content-Type: application/vnd.olrapi.dataview.dql+txt" \
  --data 'TABLE file.mtime AS modified, status FROM #project WHERE status = "active" SORT file.mtime DESC' \
  https://127.0.0.1:27124/search/
```

The response is JSON: one entry per matching file, with the file path plus the evaluated
column values — i.e. exactly "all notes that match the query," machine-readable and ready
to pipe into `jq`, scripts, or an agent.

- **Pros:** real Dataview semantics (highest fidelity); JSON out of the box; trivial to
  wrap in a shell function/alias; no new language runtime — just `curl`; the API key lives
  in the plugin settings.
- **Cons:** Obsidian **must be running** with the plugin enabled; query type is limited to
  `TABLE` over this endpoint (no `LIST`/`TASK`/`CALENDAR` directly — though `TABLE` plus
  `file.link`/columns covers most "which notes match" needs); HTTPS uses a self-signed cert.
- **Setup:** install + enable the Local REST API plugin, copy its API key, keep Dataview
  enabled. (Dataview is already enabled in Bob's vault; only Local REST API is missing.)

### Option B — `dnvriend/obsidian-search-tool` (ergonomic CLI wrapper over Option A)

A purpose-built **CLI** that talks to the same Local REST API and is explicitly designed to
be "agent-friendly." It wraps the DQL-`TABLE` endpoint (and a JsonLogic mode) with nicer
ergonomics and multiple output formats (JSON for automation, Markdown, or pretty tables).

```bash
obsidian-search-tool search 'TABLE file.name FROM #project'
obsidian-search-tool search 'TABLE file.name, author WHERE author SORT file.mtime DESC'
```

- **Requires:** Obsidian running; Local REST API plugin **and** Dataview enabled;
  `OBSIDIAN_API_KEY` env var; Python 3.14+ with `uv`.
- **Supports:** `TABLE` queries with `FROM` (tags/folders/files/links), `WHERE`, `SORT`
  (multi-field), `LIMIT`; functions `date()`, `dur()`, `contains()`; comparison/logical
  operators; the common implicit `file.*` fields.
- **Does NOT support:** `GROUP BY`, `FLATTEN`, `LIST`, `TASK`, `CALENDAR`.
- **Net:** same fidelity/availability trade-off as Option A (it *is* Option A under the
  hood) but saves us writing the curl/JSON plumbing — at the cost of a Python+uv dependency.
  Good if we want a ready-made, documented CLI rather than a shell wrapper we maintain.

### Option C — `k-lar/dynomark` (standalone DQL reimplementation, no Obsidian) ⭐ recommended for headless

A **standalone Go binary** that reimplements a Dataview-like query language and reads
markdown **directly off disk** — *no Obsidian instance required*. This is the best fit
whenever queries must run headless (cron jobs, CI, a server, or any context where launching
Obsidian is impractical).

```bash
dynomark 'TASK FROM "examples/test.md" WHERE NOT CHECKED'
dynomark 'TABLE file.cday AS "Date", title FROM todos/'
dynomark 'PARAGRAPH FROM examples/ WHERE [author] IS "Shakespeare"'
```

- **Supports:** `LIST`, `TASK`, `PARAGRAPH`, `ORDEREDLIST`, `UNORDEREDLIST`, `FENCEDCODE`,
  `TABLE` (+ `TABLE NO ID`); `WHERE` with `AND`/`OR`, `CONTAINS`, `IS`; `SORT ASC/DESC`;
  `GROUP BY` (with max-group limits); `LIMIT`; `AS` aliases. Dataview-style `key: value`
  metadata plus ~10 built-in file fields (path, name, size, created/modified timestamps).
- **Does NOT (yet) match real Dataview:** it's a *partial* implementation. Advanced
  operators, regex, rich date arithmetic, functions, and nested queries are not documented
  as supported. Inline `key:: value` vs frontmatter coverage and edge-case semantics will
  diverge from the genuine engine — queries must be validated against expected output.
- **Maturity:** ~v0.2.0 (Nov 2024), early-stage but functional; editor integrations exist
  (Neovim/VS Code/Emacs); prebuilt binaries for Linux/macOS/Windows, or `make && sudo make
  install` with Go ≥ 1.22.5.
- **Pros:** truly headless; fast; single binary; reads the vault as plain files.
- **Cons:** not byte-for-byte Dataview-compatible — fidelity is the price of independence.

### Option D — In-Obsidian export plugins (adjacent, not a true CLI)

Plugins like `udus122/dataview-publisher` (and similar "dataview serializer" tools) run a
Dataview query **inside** Obsidian and write the rendered results back into a markdown file,
keeping it up to date. Useful if the real goal is "materialize query results into a note,"
but they run inside the app on Obsidian's schedule — they are **not** a command-line
interface. Mentioned for completeness; not recommended for CLI/scripting use.

### Option E — `intellectronica/mdbasequery` (different query language)

A standalone CLI/library that queries Markdown-frontmatter "bases" and is **Obsidian
*Bases*-compatible** (the newer native query feature), running on Node 20+/Bun/Deno. It is
**not** Dataview DQL — different syntax and semantics — and it only sees frontmatter, not
Dataview inline `key:: value` fields. Worth knowing about given Obsidian's industry-wide
drift from Dataview toward Bases/Datacore, but it does not satisfy "run *Dataview* queries"
today. Listed as a forward-looking alternative, not a match.

## Recommendation

Pick by whether Obsidian can be running at query time:

1. **Default / highest fidelity (Obsidian available): Option A — Local REST API + `curl`.**
   Install and enable the Local REST API plugin (Dataview is already enabled), grab the API
   key, and wrap the curl call in a small shell function/alias (e.g. `dvq '<TABLE query>'`).
   This gives the *real* Dataview engine, exact semantics, and clean JSON output for
   scripting — with no new language runtime. If we'd rather not maintain the curl/JSON glue,
   **Option B (`obsidian-search-tool`)** is the same approach pre-packaged as a documented,
   agent-friendly CLI (at the cost of a Python+uv dependency).

2. **Headless / automation (Obsidian not running): Option C — `dynomark`.** A single Go
   binary that reads the vault off disk. Accept that it's a *partial* DQL reimplementation
   and validate each query we rely on against expected results; pin the queries we use to
   the subset it supports.

A reasonable end state is **both**: a `dvq` shell wrapper around the REST API for
interactive/high-fidelity use, and `dynomark` for cron/CI paths where spinning up Obsidian
isn't viable.

**Not recommended:** waiting for native Dataview CLI support (no timeline), or relying on
the npm `obsidian-dataview` typings to build our own headless indexer (it needs Obsidian's
`CachedMetadata`; this is effectively reimplementing the index — `dynomark` already did it).

## Open Questions / Follow-ups

- Which DQL query *types* does Bob actually need from the CLI? If it's purely "list the
  notes matching X," `TABLE`/`LIST` cover it and both recommended paths work. If `TASK`,
  `GROUP BY`, or `FLATTEN` are required, note that the REST endpoint is `TABLE`-only and
  `dynomark`'s coverage must be checked per-feature.
- Is the use case interactive (Obsidian usually open) or automated (headless)? That choice
  is what selects Option A/B vs Option C.
- Longer term: given the ecosystem shift toward **Bases/Datacore**, is it worth tracking
  `mdbasequery` (Option E) as Bob's metadata strategy evolves?

## Sources

- [Dataview — Extracting data from CLI (Discussion #471)](https://github.com/blacksmithgu/obsidian-dataview/discussions/471)
- [Dataview — Accessing the API/database outside Obsidian (Discussion #1811)](https://github.com/blacksmithgu/obsidian-dataview/discussions/1811)
- [Export Dataview query results to CSV from command line (Obsidian Forum)](https://forum.obsidian.md/t/export-dataview-query-results-to-csv-from-command-line/48046)
- [k-lar/dynomark (standalone DQL CLI, Go)](https://github.com/k-lar/dynomark)
- [coddingtonbear/obsidian-local-rest-api](https://github.com/coddingtonbear/obsidian-local-rest-api)
- [Local REST API — interactive API docs](https://coddingtonbear.github.io/obsidian-local-rest-api/)
- [dnvriend/obsidian-search-tool (CLI over Local REST API)](https://github.com/dnvriend/obsidian-search-tool)
- [udus122/dataview-publisher (in-Obsidian export)](https://github.com/udus122/dataview-publisher)
- [intellectronica/mdbasequery (Obsidian Bases-compatible CLI)](https://github.com/intellectronica/mdbasequery)
- [Dataview docs — Structure of a Query](https://blacksmithgu.github.io/obsidian-dataview/queries/structure/)
- [Dataview docs — Data Commands](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/)
