---
create_time: 2026-06-03
status: research
topic: Consolidated research on bob dataview parity with Obsidian Dataview
---
# Research: `bob dataview` Parity with Obsidian Dataview

## Answer

`bob dataview` has strong parity for the highest-value path, but not full
parity with every Obsidian Dataview feature.

The default `--engine obsidian` path runs inside a live desktop Obsidian process
and calls the installed Dataview plugin API. For DQL source expressions and DQL
block queries, that means parsing, source resolution, expression evaluation,
functions, data commands, implicit metadata, tasks, and result construction are
handled by Dataview itself.

The gaps are elsewhere:

- the CLI supports source expressions and DQL queries, but not inline DQL,
  DataviewJS blocks, or inline DataviewJS as first-class input modes;
- `paths` output is a Bob projection over Dataview results, not a native
  Dataview output format, and some queries do not have a clean source-note
  identity;
- `markdown` output uses Dataview's Markdown export API, not Obsidian's live DOM
  renderer, and Dataview cannot export `CALENDAR` queries to Markdown;
- interactive task behavior is not representable on stdout;
- the headless engines are deliberately partial: `native` is a small Rust subset
  for Bob-specific frontmatter queries, and `dynomark` is a broader external
  Dataview-like engine, not Obsidian Dataview.

So the precise conclusion is: **Bob has exact DQL evaluation when the Obsidian
engine can reach a running desktop Obsidian app; Bob does not have exact
headless Dataview parity or full Dataview UI/JS parity.**

## Verified Current State

Checked in this workspace on 2026-06-03:

- `~/bob` is the Obsidian vault.
- `~/bob/.obsidian/community-plugins.json` enables `dataview`.
- `~/bob/.obsidian/plugins/dataview/manifest.json` reports Dataview `0.5.68`.
- `bob dataview --help` exposes `--source`, `--query`, `--query-file`,
  `--format paths|json|markdown`, `--engine obsidian|dynomark|native`,
  `--origin`, `--vault`, `--bob-dir`, and `--strict-paths`.
- `docs/dataview.md` says `bob dataview` does not run `ob sync` or
  `ob sync-status`; vault freshness is owned by the external background or cron
  sync path.

The implementation is in `src/native/dataview.rs`. The default engine generates
JavaScript and runs:

```text
obsidian [vault=<NAME_OR_ID>] eval code=<generated JavaScript>
```

The generated code locates `app.plugins.plugins.dataview?.api`,
`window.DataviewAPI`, or `globalThis.DataviewAPI`, waits briefly for the
Dataview index, then calls:

- `api.pagePaths(source)` for `--source`;
- `api.tryQuery(query, origin, { forceId: true })` for structured DQL;
- `api.tryQueryMarkdown(query, origin)` for Markdown output.

The command's output contracts are Bob-specific:

- `paths`: one vault-relative Markdown path per line, best effort unless
  `--strict-paths` is used;
- `json`: a stable wrapper containing `engine`, `query_kind`, `format`,
  extracted `paths`, the Dataview or engine `result`, and `warnings`;
- `markdown`: Dataview-rendered Markdown for DQL, available only through the
  Obsidian engine.

## Obsidian Dataview Surface

Dataview has four user query modes:

1. DQL code blocks.
2. Inline DQL expressions.
3. DataviewJS code blocks.
4. Inline DataviewJS expressions.

DQL itself has four query types:

- `LIST`
- `TABLE`
- `TASK`
- `CALENDAR`

DQL queries can use data commands such as `FROM`, `WHERE`, `SORT`, `GROUP BY`,
`FLATTEN`, and `LIMIT`. Sources include tags, folders, specific files, incoming
links, outgoing links, and boolean combinations. Dataview also indexes
frontmatter, inline fields, implicit `file.*` metadata, tasks, lists, tags,
links, and typed values such as numbers, dates, durations, arrays, objects,
links, booleans, and nulls.

The default Bob Obsidian engine covers DQL block-style queries by delegating to
Dataview. It does not expose inline DQL or DataviewJS as separate CLI modes.

## Gap Inventory

| Area | Current behavior | What is lacking | Practical severity |
| --- | --- | --- | --- |
| DQL source expressions | `--source` calls Dataview `pagePaths()` through Obsidian | No headless native source expression support | Low for default; high headless |
| DQL `LIST` / `TABLE` | Exact via Obsidian; narrow in `native`; partial in `dynomark` | Native lacks computed list values, aliases, `WITHOUT ID`, richer projections | Low for default; medium headless |
| DQL `TASK` | Exact structured result via Obsidian; path extraction best effort | No native task parser; no CLI interactivity/checking | Medium |
| DQL `CALENDAR` | Exact structured result via Obsidian | No Markdown export; no native calendar support | Medium |
| Data commands | Exact via Obsidian | Native lacks `SORT`, `GROUP BY`, `FLATTEN`, `LIMIT` | High headless |
| Sources | Exact via Obsidian | Native accepts only one quoted folder, no tags/files/links/source algebra | High headless |
| Expressions/operators | Exact via Obsidian | Native has field truthiness, equality, `AND`, `OR`, parentheses only | High headless |
| Functions | Exact via Obsidian | Native has none | High headless |
| Dataview types | Exact via Obsidian | Native stores only string/bool/null, with wikilink strings resolved on demand | High headless |
| Metadata index | Exact via Obsidian | Native reads top-level scalar YAML only; no inline fields, tasks/lists, tags, link graph, or implicit `file.*` object | High headless |
| `this` / origin | Obsidian engine forwards `--origin` | Dynomark warns and ignores origin; native has no real `this` | Medium |
| Inline DQL | Not exposed | No `--expression` or inline-result mode | Medium |
| DataviewJS | Not exposed | No `--js` / `--js-file`; no safe stdout-oriented JS contract | High if needed |
| Live rendering | Not attempted | No DOM renderer, CSS fidelity, lifecycle, live reload, or interactive task state | Usually acceptable |
| Path output | Bob extracts paths from source/list/table/task/calendar-ish shapes | Aggregates, grouped rows, `WITHOUT ID`, heavy `FLATTEN`, computed rows, and task-level identities can be ambiguous | Medium |
| Exact headless Dataview | Not available | Requires reimplementing or embedding a plugin runtime plus Obsidian's metadata cache | XL / not recommended |

## Native Engine Detail

The native engine is useful, but it should not be described as a Dataview clone.
It currently supports:

- query types: `LIST` and limited `TABLE field, parent.field`;
- source: optional single quoted folder, e.g. `FROM "ref"`;
- filters: field truthiness, `field = "string"`, `field = true|false`,
  `field = [[wikilink]]`, `AND`, `OR`, and parentheses;
- parent/wikilink chains: `parent.parent.status` resolves intermediate
  frontmatter values as note links;
- data model: top-of-file YAML-like scalar frontmatter only, represented as
  `String`, `Bool`, or `Null`;
- output: `paths` for matching pages and `json` for list/table-shaped results.

It lacks the hard parts of Dataview: a typed value system, a full DQL parser,
source algebra, comparison and arithmetic operators, functions, inline fields,
implicit file metadata, tags, tasks/lists, link graph indexing, computed
columns, rendering, and DataviewJS.

This is acceptable if `native` remains a Bob-owned subset for local automation.
It becomes expensive only if it is asked to chase general Dataview behavior.

## Dynomark Detail

The dynomark engine provides more headless breadth than `native`. Upstream
describes it as a Markdown query language engine similar to Dataview and
"barebones for now." Its README lists support for features such as `LIST`,
`TASK`, `TABLE`, sorting, grouping, limits, metadata conditionals, and default
file metadata.

Bob's integration is correctly conservative:

- explicit opt-in with `--engine dynomark`;
- DQL only, no `--source`;
- `paths` and `json`, no Dataview-rendered Markdown;
- compatibility warning in JSON/stderr;
- no Obsidian `--origin` semantics.

Dynomark is a good fallback for cron/server workflows where "close enough" is
acceptable. It should not be treated as parity.

## Work to Fill the Gaps

### Small, high-value work

1. **Keep the contract explicit in docs/help**: Obsidian engine is exact for
   DQL evaluation; headless engines are partial; Bob output formats are
   Bob-specific. The current docs already do much of this, but the parity
   language should stay precise as features are added.
   - Estimate: 0.5 day when docs drift.

2. **Add a parity smoke suite**: a fixture vault plus a manual/live checklist
   comparing `--engine obsidian` against native Dataview for source expressions,
   `LIST`, `TABLE`, `TASK`, `CALENDAR` JSON, `SORT`, `GROUP BY`, `FLATTEN`,
   inline fields, `this` via `--origin`, and a query where `paths` warns.
   Automated tests can keep using fake `obsidian`; live tests should be gated
   because they require a running desktop Obsidian app.
   - Estimate: 1-2 days.

3. **Polish path extraction only where source identity really exists**: add live
   examples for grouped/flattened/task/calendar results, keep `--strict-paths`
   strict, and consider a raw result-only JSON mode if scripts dislike the Bob
   wrapper.
   - Estimate: 1-3 days.

4. **Add inline DQL expression mode if useful**: Dataview's plugin API exposes
   `evaluate`, `tryEvaluate`, and `evaluateInline`. A CLI mode such as
   `bob dataview --expression 'this.file.name' --origin Home.md --format json`
   would cover inline DQL-shaped shell use without pretending to render a note.
   - Estimate: 1-2 days.

### Medium work

5. **Add data-only DataviewJS only for a concrete use case**: a mode such as
   `--js` / `--js-file` could run inside Obsidian and serialize returned data.
   This needs security language because DataviewJS has access to the Obsidian
   plugin environment and can read or mutate files. DOM-rendering APIs such as
   `dv.table()` and `dv.taskList()` should be rejected or out of scope for this
   mode.
   - Estimate: 3-6 days for a data-returning subset.

6. **Grow the native engine surgically**: comparison operators, `not`, `LIMIT`,
   `SORT`, and perhaps `GROUP BY` would deliver the most headless value without
   building a full Dataview runtime. Lazy numeric/date comparison could support
   sorting and range filters before a complete type system exists.
   - Estimate: M-L, roughly several days to 1-2 weeks depending on how much type
   behavior is included.

### Large or not recommended

7. **Full DataviewJS rendering**: requires real Obsidian DOM containers,
   component lifecycle, async render completion, CSS/view semantics, and
   `dv.view()` behavior. This is possible to spike but brittle for a CLI.
   - Estimate: 2-6 weeks plus ongoing fragility.

8. **Full native Dataview parity**: requires a mature parser, source resolver,
   expression evaluator, type system, coercion/comparison semantics, function
   library, metadata index, task/list model, link graph, renderer/export
   semantics, compatibility tests, and upstream tracking.
   - Estimate: months plus ongoing maintenance. Not recommended.

## Recommended Solution

Do **not** pursue full native or dynomark parity with Obsidian Dataview. Keep
`--engine obsidian` as the canonical exact-DQL path, because it already uses the
installed Dataview plugin.

The pragmatic path is:

1. Preserve precise docs: exact DQL through Obsidian, partial headless engines,
   Bob-specific path/json/markdown output.
2. Add a small live parity smoke suite so regressions in the Obsidian engine and
   path extraction are visible.
3. Add inline DQL expression support if a real shell workflow needs it.
4. Invest in native only as a demand-driven Bob subset: comparisons, `not`,
   `LIMIT`, `SORT`, and maybe `GROUP BY`.
5. Consider data-only DataviewJS later, but avoid DOM/render capture unless it
   becomes a hard requirement.

This keeps the valuable behavior and avoids turning `bob-cli` into a second
Dataview implementation. The practical gap worth closing is incremental and
moderate; the theoretical gap of exact headless Dataview parity is too large
for the likely return.

## Sources

Local:

- `src/native/dataview.rs`
- `docs/dataview.md`
- `README.md`
- `tests/cli.rs`
- `sdd/research/202606/dataview_cli_commandline.md`

External:

- Dataview query modes:
  https://blacksmithgu.github.io/obsidian-dataview/queries/dql-js-inline/
- Dataview query types:
  https://blacksmithgu.github.io/obsidian-dataview/queries/query-types/
- Dataview data commands:
  https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/
- Dataview sources:
  https://blacksmithgu.github.io/obsidian-dataview/reference/sources/
- Dataview metadata and types:
  https://blacksmithgu.github.io/obsidian-dataview/annotation/add-metadata/
  https://blacksmithgu.github.io/obsidian-dataview/annotation/metadata-pages/
  https://blacksmithgu.github.io/obsidian-dataview/annotation/types-of-metadata/
- Dataview plugin API:
  https://raw.githubusercontent.com/blacksmithgu/obsidian-dataview/master/src/api/plugin-api.ts
- Obsidian CLI:
  https://obsidian.md/help/cli
- dynomark:
  https://github.com/k-lar/dynomark
