---
create_time: 2026-06-03
status: research
topic: Parity gap between the bob dataview native engine and Obsidian Dataview DQL
---
# Research: Native Engine Parity with Obsidian Dataview

## Answer

`bob dataview` already has a path to full Dataview parity: the default
`--engine obsidian` runs queries through Dataview's own plugin API, so it *is*
Dataview and matches it exactly. The parity gap you are sensing lives in the two
headless engines that exist so automation can run without a desktop Obsidian
session:

- `--engine native` — a small Rust reimplementation built for one concrete bob
  use case (frontmatter + wikilink parent traversal under `ref/`). It covers a
  thin slice of DQL.
- `--engine dynomark` — a third-party Go binary with broader but still partial
  and explicitly "not guaranteed compatible" coverage.

So the honest framing is not "bob lacks Dataview parity" but "bob has full
parity only when Obsidian is running, and a deliberately narrow subset when it
is not." The recent commits (`native dataview engine`, `native dataview table
queries`) have been growing that subset. The rest of this document inventories
exactly what the native engine is missing relative to full DQL, what it would
take to close each gap, and how much that costs.

The bottom line up front: closing the *full* gap in Rust is a large,
open-ended effort (a real expression evaluator, a type system, a date/duration
library, dozens of functions, and a richer source/parser layer). It is almost
certainly not worth it. The high-value move is targeted: add the handful of DQL
features bob's automation actually uses, and keep deferring true fidelity to the
`obsidian` engine. A concrete recommendation is at the end.

## Verified Local State

From the existing engine map and prior research (`dataview_cli_commandline.md`):

- `~/bob` is the Obsidian vault; Dataview is enabled at version `0.5.68`.
- `bob dataview` supports three engines: `obsidian` (default), `dynomark`,
  `native`. Output formats: `paths` (default), `json`, `markdown`.
- The native engine lives in `src/native/dataview.rs` (~2978 lines total; native
  query types around lines 757–1065, lexer 1093–1192, parser 1194–1426,
  frontmatter read 1477–1506).
- Native engine integration tests are in `tests/cli.rs` (the
  `dataview_native_*` group, roughly lines 1257–1460).

## What the Native Engine Supports Today

Distilled from `src/native/dataview.rs`:

**Query types**
- `LIST`
- `TABLE field, parent.field, ...` (column projection over field chains)

**Source (`FROM`)**
- A single quoted folder only: `FROM "ref"` → pages whose path starts with
  `ref/`. Optional; absent means the whole vault.

**Filter (`WHERE`)**
- Field truthiness: `WHERE source_pdf`
- Equality only: `field = "string"`, `field = true`, `field = [[wikilink]]`
- Boolean literals: `WHERE true` / `WHERE false`
- Logical `AND`, `OR`, and parentheses (AND binds tighter than OR)
- Field chains that traverse wikilinks: `parent.parent.status` resolves each
  intermediate frontmatter value as a link target, then reads the final field.

**Data model**
- Reads YAML frontmatter only. Each scalar key becomes a queryable field.
- Three value types: `String`, `Bool`, `Null`. Wikilinks are stored as strings
  and resolved on demand for link comparison and chaining.

**Output**
- `paths` for both LIST and TABLE; `json` (LIST stays list-shaped; TABLE emits
  `type`/`headers`/`values`, missing fields become `null`).

## The Parity Gap (Native Engine vs Full DQL)

Dataview's authoritative surface (confirmed against the official DQL
data-commands and functions reference on 2026-06-03) is far larger. The gaps,
grouped by severity:

### 1. Query types — missing TASK and CALENDAR
Dataview has four query types: `LIST`, `TABLE`, `TASK`, `CALENDAR`. Native has
the first two. `TASK` in particular is a common automation target (it requires
parsing checkbox/list items out of note bodies, not just frontmatter).

### 2. Data commands — missing SORT, GROUP BY, FLATTEN, LIMIT
Native supports only `FROM` and `WHERE`. Dataview's pipeline also has:
- `SORT field [asc|desc], ...`
- `GROUP BY field` (one row per unique value, with a `rows` array)
- `FLATTEN expr` (explode an array field into one row per element)
- `LIMIT n`

These are query-shaping commands, not data sources, so their absence directly
limits what a single query can express.

### 3. Sources — `FROM` only accepts one quoted folder
Dataview `FROM` accepts tags (`#tag`), folders (`"folder"`), single files,
links (incoming `[[note]]` / outgoing `outgoing([[note]])`), `csv()`, and
boolean combinations with `and` / `or` / negation (`-`). Native supports only a
single quoted folder. No tag sources at all (tags are never parsed).

### 4. Operators — only `=`
Native supports equality. Dataview supports the full set: `!=`, `<`, `>`,
`<=`, `>=`; arithmetic `+ - * / %`; logical `and` / `or` / `not`; string
concatenation; list/object indexing. Without `<`/`>` there is no range or date
filtering; without `!=` there is no negative match; without `not` no negation.

### 5. Functions — none
Dataview ships dozens of functions across constructors (`date`, `dur`, `link`,
`number`, `typeof`...), numeric (`sum`, `round`, `min`, `max`, `average`,
`reduce`...), string (`contains`, `icontains`, `regexmatch`, `regexreplace`,
`split`, `lower`, `replace`...), list/object (`length`, `filter`, `map`, `sort`,
`flat`, `any`, `all`, `none`, `unique`...), date/format (`dateformat`,
`striptime`, `durationformat`...), and utility (`default`, `choice`, `meta`...).
Native supports zero. `contains()` and `date()` comparisons alone account for a
large share of real-world queries.

### 6. Data types — only String / Bool / Null
This is the deepest gap, because everything above depends on it. Dataview has a
real type system: numbers, strings, booleans, **dates**, **durations**, links,
**arrays**, **objects**, html, null — with coercion rules. Native stores numbers
and arrays as raw strings, so numeric comparison, date math, and list operations
are impossible even before the missing operators/functions. Any serious parity
work starts here.

### 7. Implicit `file.*` metadata fields — effectively absent
Dataview exposes a rich implicit page object: `file.name`, `file.folder`,
`file.path`, `file.link`, `file.size`, `file.ctime`/`cday`, `file.mtime`/`mday`,
`file.tags`/`etags`, `file.inlinks`/`outlinks`, `file.aliases`, `file.tasks`,
`file.lists`, `file.frontmatter`, `file.day`. Native column chains like
`file.name` parse syntactically but resolve to `null` because pages only carry
frontmatter keys. So sorting/filtering by creation time, size, tags, or link
graph is unavailable.

### 8. Inline fields not parsed
Dataview reads `key:: value` inline fields (and `[key:: value]`) from note
bodies. Native reads YAML frontmatter only.

### 9. Expression / projection features
No computed columns, no column aliases (`expr AS "Header"`), no `WITHOUT ID` /
`TABLE WITHOUT ID`, no inline queries (`= expr`), no `dataviewjs`. TABLE columns
must be bare field chains.

### 10. Rendering
`--format markdown` is Obsidian-only by design (it uses Dataview's own renderer).
This is an intentional non-goal for headless engines, not an accidental gap.

## How dynomark Already Covers Part of the Gap

`--engine dynomark` (k-lar/dynomark, ~v0.2.1) is a headless Go engine that reads
Markdown directly and supports `LIST`, `TASK`, `TABLE`, `TABLE NO ID`,
`GROUP BY`, `LIMIT`, sorting, and metadata conditionals — i.e. it already
covers several of the gaps above (TASK, SORT, GROUP BY, LIMIT) without any bob
work. Its tradeoff is fidelity: its query language and output are explicitly
*not guaranteed* to match Dataview, and bob's own docs tell users to validate
against the Obsidian engine first. So bob's headless story is really a spectrum:
native (tiny, fully owned, exact-for-its-subset) → dynomark (broad, third-party,
approximate) → obsidian (full, exact, requires GUI).

## What It Would Take to Close the Gaps, and the Cost

Effort is relative to a Rust implementation in `src/native/dataview.rs`, with
the existing lexer/parser/evaluator as the starting point. Sizes are rough
engineering estimates (S ≈ <1 day, M ≈ 1–3 days, L ≈ 4–8 days, XL ≈ multi-week).

| Gap | What it takes | Size |
| --- | --- | --- |
| Comparison operators `!=,<,>,<=,>=` | Extend lexer tokens + parser primary; add ordering to value eval | S–M |
| `not` / negation | One parser rule + eval arm | S |
| `LIMIT n` | Trivial post-filter truncation | S |
| `SORT field [asc|desc]` | Parse clause; stable sort on resolved field values (needs comparable types) | M |
| Richer `FROM` (tags, links, `and`/`or`/`-`) | Parse tags (requires parsing tags from notes), link sources, boolean source algebra | M–L |
| `GROUP BY` | Parse clause; bucket rows; emit `rows` array — pushes JSON shape toward Dataview's | M |
| `FLATTEN` | Parse clause; requires real array values first | M (after types) |
| Real type system (number/date/duration/array/object + coercion) | New value enum, YAML→typed conversion, comparison/coercion rules, a date/duration lib (e.g. `chrono`) | L–XL |
| Function library (subset) | Per-function impl + arg typing; even 15–20 common ones is meaningful work | L (per ~20 fns) |
| Implicit `file.*` fields | Capture file metadata at read time (mtime/ctime/size/path), parse tags, build link graph for inlinks/outlinks | M–L |
| Inline `key:: value` fields | Body scanner + merge into page fields | M |
| TASK queries | Parse list/checkbox items with positions from note bodies; new result shape | L |
| CALENDAR queries | Date-bucketed output; depends on date type | M (after dates) |
| Computed columns / `AS` aliases / `WITHOUT ID` | Expression evaluator in projection + parser support | M (after expr eval) |
| Inline queries / dataviewjs | Out of scope for a headless Rust subset | N/A |

**Aggregate read:** the "small wins" tier (comparison operators, `not`,
`LIMIT`, `SORT`, `GROUP BY`) is collectively **M–L** and delivers most of the
everyday expressiveness people miss. The "deep" tier (type system → functions →
file.* → TASK/CALENDAR/FLATTEN) is collectively **XL** and is effectively
reimplementing Dataview. Chasing true parity in Rust is a multi-week project
that then has to track upstream Dataview changes forever — a standing
maintenance tax.

## Alternatives to Building It in Rust

1. **Lean on the Obsidian engine for fidelity (status quo).** Anything that
   needs real DQL already works today via `--engine obsidian`. The only cost is
   requiring a running desktop session. For interactive/dev use this is free.

2. **Promote dynomark for headless breadth.** dynomark already covers TASK,
   SORT, GROUP BY, and LIMIT. Investing in better dynomark integration,
   validation harnesses, and docs could close much of the headless gap with no
   Rust engine work — at the cost of dynomark's approximate fidelity and an
   external binary dependency.

3. **Grow the native engine surgically.** Add only the features bob's own
   automation queries actually use. The native engine was born to answer one
   real query (ref/parent traversal); extend it the same way — demand-driven.

4. **Full native parity.** Reimplement Dataview in Rust. Maximum control and
   zero external deps, but XL effort plus perpetual upstream-tracking. Not
   recommended unless headless exact-Dataview becomes a hard product
   requirement.

## Recommendation

Treat full parity as a non-goal for the native engine and say so explicitly in
the docs. Position the three engines as a deliberate fidelity/portability
spectrum: **obsidian = exact (needs GUI)**, **dynomark = broad but approximate
(headless)**, **native = exact for a small owned subset (headless, no external
deps)**.

Then make one **targeted, demand-driven** investment in the native engine —
the M–L "small wins" tier, which buys the most expressiveness per unit work and
needs no type-system overhaul:

1. Comparison operators (`!=`, `<`, `>`, `<=`, `>=`) and `not`. (S–M)
2. `LIMIT` and `SORT`. (S–M)
3. `GROUP BY` if/when an actual bob workflow needs grouped output. (M)
4. A minimal numeric/date awareness *only* where `SORT`/comparison needs it —
   parse numbers and ISO dates lazily at compare time rather than building the
   full type system up front. (M)

Defer the XL tier (full type system, function library, TASK/CALENDAR/FLATTEN,
inline fields, file.* graph) unless a concrete, recurring headless use case
forces it. When that day comes, re-evaluate dynomark first — adopting its
coverage is almost always cheaper than reimplementing it.

Net: the practical gap worth closing is **M–L, incremental, and optional**. The
theoretical gap (true Dataview parity in Rust) is **XL and not worth it**, and
bob already neutralizes it through the obsidian engine.

## Sources

- Dataview DQL data commands:
  https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/
- Dataview DQL functions reference:
  https://blacksmithgu.github.io/obsidian-dataview/reference/functions/
- Dataview query types:
  https://blacksmithgu.github.io/obsidian-dataview/queries/query-types/
- Prior research: `sdd/research/202606/dataview_cli_commandline.md`
- Implementation: `src/native/dataview.rs`; tests: `tests/cli.rs`
  (`dataview_native_*`); user docs: `docs/dataview.md`
- dynomark: https://github.com/k-lar/dynomark
</content>
</invoke>
