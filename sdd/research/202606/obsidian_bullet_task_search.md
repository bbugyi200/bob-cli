---
create_time: 2026-06-26
status: research
topic: Best ways to search for particular bullets or tasks in the Bob Obsidian vault
---
# Research: Searching for Bullets and Tasks in Obsidian

## Question

What is the best way to search for a *particular* bullet (plain list item) or
task (checkbox item) in Bryan's `~/bob` Obsidian vault — both for quick
interactive lookups and for repeatable, scriptable queries?

## Short Answer

There is no single tool. The right answer is a small, layered toolkit, picked by
how the search is used:

1. **Interactive, one-off "where did I write that?" lookups → core Search
   operators.** Already enabled, zero setup. Use `task:`, `task-todo:`,
   `task-done:` for tasks and `line:` / `block:` / `section:` for bullets, plus
   `path:`, `tag:`, regex `/.../`, and boolean `OR` / `-` / `()`. Promote the
   ones you run often into **bookmarked searches** and **embedded ` ```query `
   blocks** on a dashboard note.
2. **Living task dashboards and rich task filtering → the Tasks plugin.** Its
   query blocks filter on real task properties (status type, due / scheduled /
   start, priority, recurrence, `description regex`) and sort, group, and limit.
   This is the best tool for tasks, but it only sees `#task`-filtered checkbox
   lines — not plain bullets.
3. **Repeatable, scripted, or headless cross-vault search → `bob dataview`.**
   Verified working today for both tasks (`TASK WHERE contains(...)`) and plain
   bullets (`LIST ... FLATTEN file.lists`), with JSON output and no desktop app
   required. This is the uniquely-Bryan layer and the natural home for any
   future automation.

Optional: install **Omnisearch** only if core Search's *ranking* (not its
operators) becomes the pain point — it adds fuzzy, typo-tolerant, BM25-ranked
full-text search.

The recommended solution (bottom of this doc) is to standardize on these three
layers and, if bullet/task search becomes a recurring CLI need, add a thin
`bob find` wrapper over `bob dataview` so common searches don't require
hand-writing DQL.

## Verified Local Context

Checked on 2026-06-26 against `~/bob` and this `bob-cli` workspace:

- Vault `~/bob` exists with **5,231** Markdown notes (excluding `.obsidian` and
  `.trash`).
- Core plugins relevant to search are **enabled**: `global-search`, `switcher`
  (Quick Switcher), `backlink`, `tag-pane`, `bookmarks`, and `bases`.
- Enabled community plugins include `dataview` (**0.5.68**),
  `obsidian-tasks-plugin` (**8.0.0**), `metadata-menu`, `quickadd`,
  `templater-obsidian`, `task-status-cycler`, and the custom
  `bob-project-tasks` plugin ("Keep project task counts materialized in
  frontmatter"). **Omnisearch is not installed.**
- Tasks settings: `globalFilter: "#task"`, `taskFormat: "dataview"`, with
  created / done / cancelled date tracking on. Custom statuses: `/` →
  In Progress (`IN_PROGRESS`), `B` → Blocked (`ON_HOLD`), `-` → Canceled
  (`CANCELLED`).
- `bob dataview` works headlessly against the local Markdown index. Confirmed
  end-to-end today:
  - **Task search:** `bob dataview --format markdown --query 'TASK WHERE
    contains(tags,"#task") AND contains(lower(text),"obsidian") LIMIT 3'`
    returned rendered task lines such as
    `- [x] (0800-0915) #task Make Obsidian Better!`.
  - **Bullet search:** `bob dataview --format json --query 'LIST WITHOUT ID
    L.text FROM "2026" FLATTEN file.lists AS L WHERE
    contains(lower(L.text),"obsidian") LIMIT 3'` returned `type: list` rows with
    their source path. So `bob dataview` can search arbitrary list items, not
    just tasks.
- `bob` has no dedicated `search` / `find` / `grep` subcommand today. The
  closest surfaces are `bob dataview` (general query) and `bob projects` (which
  manages project notes via their `^prj` tasks).

## Framing: "Bullet" vs "Task", and the Four Search Axes

A precise tool choice depends on two distinctions.

**Bullet vs task.** In this vault every checkbox task is also a list item, but
not every list item is a task:

```markdown
- a plain bullet                      ← list item only
- [ ] #task a real task               ← list item AND task
```

This matters because the *task-aware* tools only see checkbox lines:

- The **Tasks plugin** only indexes lines matching the `#task` global filter.
- Core Search's `task:` / `task-todo:` / `task-done:` only match checkbox items.
- To search **plain bullets**, use core `line:` / `block:`, or Dataview's
  `FLATTEN file.lists` (which yields both list items *and* tasks).

**Four axes** that decide the tool:

| Axis | Options |
| --- | --- |
| Target | plain bullet · checkbox task · either |
| Match on | free text · tag · inline field (`[due:: …]`) · task status / date / priority |
| Where consumed | interactive (sidebar / switcher) · embedded in a note · terminal / script |
| Match style | exact / boolean / regex · fuzzy / ranked |

## Tool Survey

### 1. Core Search (interactive default; bullets *and* tasks)

Obsidian's built-in Search (the `global-search` core plugin, already enabled) is
the fastest, lowest-friction option and the right default for "find that one
bullet/task" while working. It searches the whole vault and supports a rich
operator set:

| Operator | Finds | Notes |
| --- | --- | --- |
| `task:(term)` | any task whose text matches `term` | `task:""` matches all tasks |
| `task-todo:(term)` | incomplete tasks | |
| `task-done:(term)` | completed tasks | |
| `line:(term)` | terms on the **same single line** | best for "this one bullet has X and Y" |
| `block:(term)` | terms within the **same block** | a task plus its sub-bullets count as one block |
| `section:(term)` | terms within the same heading section | |
| `path:`, `file:` | by location / filename | scope a search to a folder or note |
| `tag:#task` | by tag | combine to narrow tasks |
| `["key":"value"]` | by note property (frontmatter) | |
| `/regex/` | regular expression | case-insensitive unless flagged |

Operators combine with boolean `OR`, `-` (negation), `()` grouping, and `""`
for exact phrases, and nest: `task:(call OR email)` matches tasks mentioning
either word. Examples:

```text
task-todo:(invoice)                  → open tasks about invoices
task-todo:("follow up") -tag:#someday → open follow-ups, excluding someday
line:(obsidian search)               → one bullet mentioning both words
block:(roadmap) section:(Pomodoros)  → blocks under a "Pomodoros" heading
path:"2026/" task-done:(migrate)     → completed "migrate" tasks in 2026 dailies
```

Make recurring searches durable two ways, both already available:

- **Bookmark a search.** With the `bookmarks` core plugin (enabled), save a
  Search query so it is one click to re-run. Good for "Open #task triage" or
  "Unprocessed inbox bullets".
- **Embed a live query in a note.** A fenced ` ```query ` block renders live
  Search results inside any note — e.g. a `search.md` dashboard with one block
  per saved view. (Embedded queries can't use the Search-pane settings unless
  you add the `obsidian-query-control` plugin.)

Strengths: instant, no query language, works on bullets and tasks, regex.
Weakness: substring/boolean matching with no relevance ranking — a common word
buries the bullet you want, and there is no fuzzy / typo tolerance.

### 2. Tasks Plugin (best for tasks; dashboards and rich filters)

For checkbox tasks specifically, the installed Tasks plugin (8.0.0) is the
strongest tool. A fenced ` ```tasks ` block filters on genuine task properties
rather than raw text, and sorts / groups / limits the result:

```tasks
not done
description regex matches /invoice|receipt/i
path includes 2026
sort by due
group by path
limit groups to 3
```

It understands this vault's custom statuses, so you can target work state
directly instead of guessing checkbox symbols:

```tasks
(status.type is TODO) OR (status.type is IN_PROGRESS)
```

Because the vault uses `taskFormat: dataview`, Tasks reads inline fields like
`[due:: 2026-07-01]` and `[p:: 1]`, so date/priority filters work on the
existing data. Boolean combinators (`AND` / `OR` / `NOT`, capitalized) and
`description regex matches /.../` cover precise text search.

Strengths: the only tool that filters on task semantics (due, scheduled,
priority, recurrence, status type) and renders an interactive, editable list.
Weakness: only sees `#task`-filtered checkbox lines — useless for plain bullets
— and lives inside the desktop app.

### 3. Dataview / `bob dataview` (bullets via FLATTEN; headless & scriptable)

Dataview is the most general matcher and the only one of the three that is also
available headless through `bob dataview` — which fits Bryan's CLI/automation
workflow and needs no running Obsidian.

**Tasks**, with full inline-field and tag access:

```bash
bob dataview --format markdown --query '
TASK
WHERE contains(tags, "#task")
  AND !completed
  AND contains(lower(text), "invoice")
'
```

**Plain bullets** (and tasks), by flattening every note's list items — the key
move for non-task bullet search:

```bash
bob dataview --format json --query '
LIST WITHOUT ID L.text
FROM "2026"
FLATTEN file.lists AS L
WHERE contains(lower(L.text), "obsidian")
'
```

You can filter flattened items by `meta(L.section)` (heading), `L.tags`, or
inline fields, and `--format json` returns `path` + `line` per row for scripts.
Both queries above were verified against `~/bob` today.

Strengths: searches bullets and tasks, filters on inline fields/sections/tags,
returns structured JSON, runs from the terminal or cron. Weakness: you must
write DQL, and it is read-only (a matcher, not an interactive jump-to).

### 4. Optional: Omnisearch and Query Control

- **Omnisearch** (not installed) adds fuzzy, typo-tolerant, BM25-ranked
  full-text search with a Quick-Switcher-style UI. It solves core Search's
  *ranking* weakness, not its operator set — and it is note/excerpt ranked, not
  a per-bullet/per-task operator engine. Worth installing only if "the right
  note ranks too low" is the actual pain; it does not replace Tasks or Dataview
  for structured filtering.
- **Query Control** (not installed) upgrades embedded ` ```query ` blocks with
  sorting, context, and collapse controls. A nice-to-have if the embedded-search
  dashboard route (layer 1) becomes central.

## Decision Matrix

| You want to… | Use |
| --- | --- |
| Jump to one bullet/task you half-remember, right now | Core Search operators (`line:` / `block:` / `task-todo:`) |
| Re-run the same lookup often, inside Obsidian | Bookmarked search **or** embedded ` ```query ` block |
| A filtered, sortable, editable task list (due/priority/status) | Tasks plugin ` ```tasks ` block |
| Find **plain bullets** (non-task list items) by text/tag/section | Core `line:`/`block:`, or `bob dataview` `LIST … FLATTEN file.lists` |
| Search from the terminal / a script / cron, structured output | `bob dataview` (`TASK …` or `LIST … FLATTEN`) |
| Better relevance ranking / typo tolerance across notes | Omnisearch (optional install) |

## Recommended Solution

Adopt the three native layers already present, mapped to intent, and add one
small CLI affordance if bullet/task search becomes a recurring terminal need.

1. **Make core Search the default interactive tool, and learn six operators.**
   `task-todo:`, `task-done:`, `line:`, `block:`, `section:`, plus regex
   `/.../`, combined with `path:`, `tag:`, `OR`, `-`, `()`. This covers almost
   every "find that bullet/task" moment with zero setup and works on both
   bullets and tasks.

2. **Persist the recurring searches.** Create a `search.md` (or extend a
   dashboard note) with embedded ` ```query ` blocks for the handful of searches
   run weekly (e.g. open inbox bullets, open `#task` by area), and bookmark the
   same queries for one-click reuse. This converts ad-hoc operator strings into
   durable, named views without any plugin install.

3. **Use the Tasks plugin for anything task-shaped that needs filtering or
   sorting.** Lean on `status.type`, `description regex matches /.../`, `due` /
   `scheduled` filters, and `group by` / `limit`. It is already configured for
   this vault's `#task` filter, dataview format, and custom statuses, so it is
   the highest-leverage tool for tasks specifically.

4. **Reach for `bob dataview` for plain-bullet search, repeatability, and
   automation.** Keep a couple of canned `.dql` files (one `TASK` template, one
   `LIST … FLATTEN file.lists` bullet template) so a headless search is a single
   command. This is the only layer that searches non-task bullets from the
   terminal and returns JSON for scripts.

5. **Optional, only if a specific gap bites:** install **Omnisearch** if note
   ranking/typo tolerance is the frustration; install **Query Control** if the
   embedded-query dashboard becomes central.

**Forward-looking enhancement (optional, SDD-worthy).** Because Bryan already
wraps vault operations in ergonomic subcommands (`bob projects` over `^prj`
tasks, `bob capture`, `bob move-done-tasks`), the cleanest long-term answer to
"search for a particular bullet or task" is a thin **`bob find`** (or
`bob search`) subcommand layered over `bob dataview`. It would translate simple
flags into the verified DQL patterns above — e.g.:

```bash
bob find --task --open --text invoice          # → TASK WHERE !completed AND contains(...)
bob find --bullet --in 2026 --text obsidian     # → LIST … FLATTEN file.lists WHERE contains(...)
bob find --task --tag gtd --section Pomodoros
```

This keeps the powerful query engine but removes the need to hand-write DQL for
common bullet/task searches, and it composes with the rest of `bob-cli` and
cron. It is a candidate for a follow-up tale/epic, not a prerequisite — layers
1–4 are usable today.

## Sources

- Obsidian Search (operators, embedded queries):
  https://obsidian.md/help/plugins/search
- Obsidian Search — Five Hidden Features (`line:`, `block:`, `section:`,
  `task:`, nested terms, embedding):
  https://obsidian.rocks/obsidian-search-five-hidden-features/
- Tasks — Filters: https://publish.obsidian.md/tasks/Queries/Filters
- Tasks — Regular Expressions:
  https://publish.obsidian.md/tasks/Queries/Regular+Expressions
- Tasks — About Queries: https://publish.obsidian.md/tasks/Queries/About+Queries
- Dataview — list items via `FLATTEN file.lists` (show list items containing a
  tag / under a heading):
  https://s-blu.github.io/obsidian_dataview_example_vault/20%20Dataview%20Queries/Show%20list%20items%20containing%20a%20certain%20tag/
- Dataview — Query structure:
  https://blacksmithgu.github.io/obsidian-dataview/queries/structure/
- Omnisearch: https://github.com/scambier/obsidian-omnisearch
- Query Control (embedded query enhancements):
  https://github.com/nothingislost/obsidian-query-control
- Local: `docs/dataview.md`; `sdd/research/202606/bulk_obsidian_task_properties.md`;
  `sdd/research/202606/obsidian_improvements_consolidated.md`
- Local verification: `bob dataview` TASK and `LIST … FLATTEN file.lists`
  queries plus `~/bob/.obsidian` plugin/settings inspection on 2026-06-26.
