# Obsidian Project Support Review

Date: 2026-06-12

## Scope

Reviewed the new Bob project workflow across:

- `bob projects list` and `bob projects sync` in `src/native/projects.rs`
- `docs/projects.md` and `README.md`
- Live vault project surfaces in `~/bob`: `_templates/new_project.md`, `project.md`, `dash.md`,
  `projects.base`, `bob-navigation-hotkeys`, and `bob-project-tasks`

Read-only live checks:

- `cargo run --quiet -- projects list --bob-dir ~/bob`
  - 7 active projects, 0 waiting, 0 done, 0 canceled
  - no scan errors
- `cargo run --quiet -- projects sync --dry-run --bob-dir ~/bob`
  - 0 status updates, 0 `^prj` edits, 0 warnings
- `cargo test --quiet projects`
  - all filtered project tests passed

The live vault had unrelated dirty files before review. I did not write to `~/bob`.

## Current Design

The strongest part of the implementation is that it uses a single project completion task, anchored as `^prj`, as the
human interaction point. That aligns with GTD: a project has an outcome, and active project review should keep a current
next action visible.

Current behavior:

- A project is any Markdown note with frontmatter `type: "[[project]]"`.
- `^prj` checked or canceled drives frontmatter `status: done` or `status: canceled`.
- Open active projects with no unprioritized open tasks and no open subprojects have `[p::2]` removed from `^prj`, which
  makes the project completion task appear in `dash.md`.
- Active projects with unprioritized open tasks or open subprojects get `[p::2]` added to `^prj`, hiding the project
  task from the daily dashboard while real next actions or child projects exist.
- Open subprojects are inferred from child `parent` wikilinks and represented on one generated marker line under `^prj`.
- `projects.base` uses materialized `task_count` and `open_task_count` fields maintained by `bob-project-tasks`, because
  Obsidian Bases operate over note/file properties and formulas rather than arbitrary task collections.

## External Research

Relevant best-practice signals:

- GTD's clarify step explicitly distinguishes the next action from the project when something takes more than one action:
  https://gettingthingsdone.com/what-is-gtd/
- GTD recommends keeping next actions on action/context lists, with project plans as support material. Linking projects
  and next actions is useful, but review discipline is the real control loop:
  https://gettingthingsdone.com/2020/06/the-gtd-approach-to-linking-next-actions-and-projects/
- The GTD Weekly Review includes reviewing every project and ensuring at least one current next action exists:
  https://gettingthingsdone.com/2018/08/episode-43-the-power-of-the-gtd-weekly-review/
- PARA's project/area split matches the vault's `project` and `area` types: projects are short-term efforts with a goal;
  areas require ongoing attention:
  https://fortelabs.com/blog/para/
- Obsidian properties are YAML frontmatter. Keys should be unique, and internal links in text/list properties should be
  quoted:
  https://obsidian.md/help/properties
- Obsidian Bases are YAML files with filters, formulas, views, note properties, and file properties. There is no
  Dataview-style `FROM`; filters narrow the vault-wide file set:
  https://obsidian.md/help/bases/syntax
- Bases formulas and filters use note/file properties and functions such as `file.hasLink`, `file.hasTag`, and
  `file.inFolder`:
  https://obsidian.md/help/bases/functions
- Dataview's bracketed inline fields are the right syntax for per-task metadata, while frontmatter applies to the whole
  page:
  https://blacksmithgu.github.io/obsidian-dataview/annotation/add-metadata/
- Dataview also reinforces quoting wikilinks in YAML:
  https://blacksmithgu.github.io/obsidian-dataview/annotation/types-of-metadata/
- Tasks tracks tasks vault-wide, can use a global filter such as `#task`, supports custom statuses, and updates the
  source task from query views:
  https://publish.obsidian.md/tasks/Introduction
- Tasks auto-suggest appears only on lines recognized as Tasks tasks; with a global filter, the line must include that
  filter. Tasks also has ordering rules where misplaced metadata can be silently ignored:
  https://publish.obsidian.md/tasks/Editing/Auto-Suggest

## Findings

### 1. Placeholder detection is inconsistent

`bob projects` warns only on `<short_project_completion_criteria_goes_here>`
(`src/native/projects.rs:19`), but the live project template and creation plugin use
`(REPLACE WITH PROJECT COMPLETION CRITERIA)` (`~/bob/_templates/new_project.md:8`,
`~/bob/.obsidian/plugins/bob-navigation-hotkeys/main.js:22`).

Impact: a newly created project can keep the template placeholder indefinitely without `bob projects sync` warning.

Recommendation: define one canonical placeholder across the template, plugin, docs, and CLI tests. Prefer the friendlier
current template string, then update the Rust warning constant and fixture tests. Also add `[created::YYYY-MM-DD]` to the
template's `^prj` task or remove the vault doc claim that hand-created project tasks should include it.

### 2. CLI and dashboard task counts use different definitions

`bob projects list` counts open `#task` lines anywhere after frontmatter and includes the `^prj` task
(`src/native/projects.rs:1473-1495`). `bob-project-tasks` counts only tasks under `## Tasks`
(`~/bob/.obsidian/plugins/bob-project-tasks/main.js:132-149`), and `projects.base` displays those materialized counts
(`~/bob/projects.base:10`).

Observed mismatch:

- `sase_install`: CLI `OPEN=4`; frontmatter/dashboard `open_task_count=3`, because the CLI includes `^prj`.
- `needs_attn_tasks`: CLI `OPEN=15`; frontmatter/dashboard `open_task_count=0`, because the note has task lines outside
  `## Tasks`.

Recommendation: choose one canonical count model. I recommend:

- Workload counts: only `## Tasks`, excluding `^prj`.
- Surfacing logic: may scan the whole project body for unprioritized open `#task` if that is intentional, but label it as
  `UNPRI` or `dash blockers`, not as the project task count.
- `bob projects list` should either read the materialized frontmatter counts or use the same `## Tasks` parser as the
  plugin.
- Add a warning for project notes with body `#task` lines outside `## Tasks`, so `needs_attn_tasks` cannot silently look
  empty in `projects.base`.

### 3. Project documentation still describes retired scheduled behavior

`docs/projects.md` correctly says sync removes existing `[scheduled::...]` fields and uses `[p::2]` for dash surfacing
(`docs/projects.md:68-83`). `README.md` still says projects with no open P0 tasks get `[scheduled::YYYY-mm-dd]`
(`README.md:136-143`). The live vault type note repeats the same stale behavior (`~/bob/project.md:27-29`).

Impact: the docs disagree on whether `scheduled` is current or legacy, and `bob projects sync` will remove the field from
open active project tasks.

Recommendation: update `README.md` and `~/bob/project.md` to describe the `[p::2]` behavior and the generated
Sub-projects line.

### 4. "Project Notes" vs. "Project Support" contract drift

The vault type note says supporting material belongs under `## Project Support` (`~/bob/project.md:31`), while the live
template creates `## Project Notes` (`~/bob/_templates/new_project.md:13`) and existing project notes follow that
template.

Recommendation: use one heading. GTD terminology favors "Project Support", but the current template says "Project
Notes". Either is fine; update the type note, template, and any future automation to agree.

### 5. Parent/subproject identity is stem-based

The CLI resolves parent links to a lowercase file stem and writes generated child links as `[[ChildStem]]`. That is
simple and currently fits the root-level project notes, but it will collide if projects ever move into folders or if two
notes share a basename.

Recommendation: either enforce root-level unique project basenames as a documented invariant, or move to path-aware
parent matching and generated links. If root-level-only is intentional, add a `bob projects doctor` check for duplicate
project stems and non-root project files.

### 6. Status and review metadata are thin

Current status is enough for lifecycle control, but Bases would become more useful with review metadata:

- `last_reviewed`
- `next_review`
- `closed`
- optional `outcome` copied from the `^prj` description

Recommendation: keep `^prj` as the source of truth, but consider syncing `closed` from checked/canceled `^prj` metadata
when present. Add a weekly-review command or view that highlights active projects with no current `## Tasks` next action,
missing counts, stale `last_reviewed`, or placeholder `^prj` text.

## Prioritized Improvements

1. Fix the placeholder mismatch between `bob projects`, `_templates/new_project.md`, and `bob-navigation-hotkeys`.
2. Update stale scheduled-field docs in `README.md` and `~/bob/project.md`.
3. Align task-count semantics across `bob projects list`, `bob-project-tasks`, and `projects.base`.
4. Normalize the support heading to either `## Project Support` or `## Project Notes`.
5. Add `bob projects doctor` checks for malformed project contracts:
   - placeholder `^prj`
   - body tasks outside `## Tasks`
   - missing `status`
   - missing/invalid `parent`
   - duplicate project stems
   - stale `scheduled` on active open `^prj`
6. Consider `last_reviewed` or a review dashboard after the core contract is consistent.

## Suggested Acceptance Tests

- A project created from the template triggers the placeholder warning until the completion criterion is replaced.
- A project with `#task` lines outside `## Tasks` reports a warning or has matching CLI/dashboard counts by design.
- `README.md`, `docs/projects.md`, and `~/bob/project.md` all describe the same `[p::2]` surfacing behavior.
- `bob projects sync --dry-run --bob-dir ~/bob` remains clean after the docs/template cleanup.
- A duplicate project stem fixture either errors or produces path-aware generated subproject links.
