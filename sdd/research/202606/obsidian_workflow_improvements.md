---
create_time: 2026-06-04
status: research
topic: Improving Bryan's Obsidian workflow
---
# Research: Obsidian Workflow Improvements

## Question

What current Obsidian features and plugins are most likely to improve Bryan's
use of the `~/bob` vault without disrupting the existing Markdown, Sync,
Dataview, Tasks, and CLI-oriented workflow?

## Short Answer

The highest-leverage improvement is not a new note system. It is tightening the
interfaces around the system already in place:

1. Use Bases more aggressively as the native review/dashboard layer for notes
   with frontmatter properties.
2. Configure QuickAdd choices for repeatable capture, especially choices that
   enforce `parent`, `type`, `status`, and `#task` conventions.
3. Use Web Clipper templates for external reading capture into the existing
   `ref/` and daily-note workflow.
4. Keep using `ob`/Obsidian Headless for headless Sync, and use the official
   `obsidian` CLI only when desktop Obsidian is available.
5. Keep the current Tasks plugin model unless there is a deliberate migration
   reason to make every task a separate Markdown file.

## Local Context

Checked on 2026-06-04:

- Project memory says `~/bob/` is Bryan's Obsidian vault.
- Project memory says this machine uses `obsidian-headless` through `ob` for
  Obsidian Sync without needing a full GUI Obsidian session.
- Project memory says new Markdown notes under `~/bob/` should include a
  `parent` frontmatter field linking to another Markdown file.
- `~/bob` currently has about 5,694 Markdown files.
- About 5,669 of those Markdown files contain YAML frontmatter delimiters.
- Only 313 files currently have a `parent:` line.
- Core plugin `bases` is enabled.
- Only two `.base` files were found: `refs.base` and `Untitled.base`.
- `refs.base` is already a useful native dashboard over `ref/` notes, with
  formulas for title links, categories, status badges, and source links.
- Enabled community plugins are:
  - `dataview` 0.5.68
  - `obsidian-tasks-plugin` 8.0.0
  - `quickadd` 2.12.3
  - `templater-obsidian` 2.20.5
  - `task-status-cycler` 1.0.0
  - `note-refactor-obsidian` 1.8.2
  - `obsidian-relative-line-numbers` 3.0.0
  - local/custom plugins: `block-id-prompt`, `bob-ledger-tools`,
    `bob-navigation-hotkeys`
- QuickAdd is installed but has zero configured choices.
- Templater has a template folder at `_templates`, with `daily.md` and
  `schedule.md`.
- Tasks is configured with `globalFilter: "#task"` and `taskFormat:
  "dataview"`, with created/done/cancelled date tracking enabled.
- Local `ob` is version `0.0.8`; npm and upstream GitHub currently show
  `obsidian-headless` `0.0.10`.
- Local Node is `v22.14.0`, which satisfies Obsidian Headless' current Node 22+
  requirement.
- `obsidian version` failed in this non-GUI shell with "The CLI is unable to
  find Obsidian. Please make sure Obsidian is running and try again." That is
  consistent with official Obsidian CLI docs: the CLI controls a running
  Obsidian app and is not the same as Obsidian Headless.

## Recommendation 1: Build More Bases

Obsidian Bases is now a core plugin for database-like views over local Markdown
files and their properties. Views can filter, sort, group, copy, export, and
create files. Base definitions live as plain-text `.base` files or embedded
Markdown code blocks.

This fits Bob because the vault already stores most notes as frontmatter-bearing
Markdown files, and `refs.base` already proves the pattern locally.

Suggested new Bases:

| Base | Purpose | Key filter idea |
| --- | --- | --- |
| `inbox.base` | Notes that need triage | missing `parent`, missing `type`, missing `status`, or recent file mtime |
| `notes_by_parent.base` | Review notes organized by topic | group by `parent` |
| `active_work.base` | Non-task notes that are active | `status == "wip"` or `status == "active"` |
| `daily_review.base` | Recent daily notes and review notes | `type == "daily"` or folder/date filter |
| `ref_review.base` | Extend existing `refs.base` or replace `Untitled.base` | `file.path.startsWith("ref/")` |

Use Bases for note-level metadata and dashboards. Keep Dataview/Tasks for
task-line queries, because Tasks metadata lives inside individual Markdown task
lines rather than note frontmatter.

## Recommendation 2: Turn QuickAdd Into the Capture Router

QuickAdd is installed and current, but no choices are configured. That is a good
opportunity: add a small number of choices instead of a large plugin migration.

Good first choices:

| Choice | Type | Behavior |
| --- | --- | --- |
| `Capture inbox` | Capture | Append a timestamped item to today's daily note or an inbox note. |
| `New ref` | Template | Create a note under `ref/` with `type`, `status`, `url`, `parent`, and title fields. |
| `New project note` | Template | Prompt for `parent`, `status`, and project link, then create a note from a standard template. |
| `New task` | Capture or Macro | Append `- [ ] #task ...` with optional `[scheduled::]`, `[due::]`, and `[p::]` fields. |
| `Schedule pomodoro` | Macro | Reuse the existing `_templates/schedule.md` logic from a command/hotkey path. |

QuickAdd 2.12.x also exposes native Obsidian CLI handlers when Obsidian 1.12.2+
supports plugin CLI commands. That means a future shell path can look like:

```bash
obsidian vault=Bob quickadd choice="Capture inbox" value-text="..."
```

Caveat: this requires desktop Obsidian to be available, because it is an
Obsidian CLI/plugin-runtime workflow. For headless server jobs, prefer direct
Markdown writes guarded by `ob sync`.

## Recommendation 3: Add Web Clipper Templates

The official Obsidian Web Clipper can use templates to create new notes, append
to existing notes, or append to daily notes. Templates can use page variables in
the note name, note location, properties, and content. Useful variables include
title, URL, author, published date, domain, site, description, image, selected
text, highlights, and Markdown content.

Suggested Web Clipper templates:

### Reading Queue

Target: create a note under `ref/web/` or `ref/blogs/`.

Properties:

```yaml
type: ref
status: unread
parent: "[[reading]]"
url: "{{url}}"
site: "{{site}}"
author: "{{author}}"
published: "{{published}}"
clipped: "{{date}}"
```

Content:

```markdown
# {{title}}

{{description}}

## Highlights

{{highlights}}

## Content

{{content}}
```

### Daily Capture

Target: append to the daily note.

Content:

```markdown
- {{date}} {{time}} [{{title}}]({{url}})
  - {{selection}}
```

This keeps quick external capture aligned with the current daily-note workflow,
while richer captures become property-bearing reference notes that Bases can
review.

## Recommendation 4: Separate Headless Sync From App Runtime

There are two useful command-line surfaces, but they solve different problems:

| Tool | Best use | Important constraint |
| --- | --- | --- |
| `ob` / Obsidian Headless | Sync and Publish without desktop Obsidian | Does not load community plugins such as Dataview, Tasks, or QuickAdd |
| `obsidian` CLI | App commands, Bases queries, properties, task toggles, plugin commands, QuickAdd CLI | Requires desktop Obsidian to be installed/enabled and reachable |

For Bob automation:

- Use `ob sync --path ~/bob` before scripts that inspect or mutate the vault.
- Upgrade `obsidian-headless` from local `0.0.8` to `0.0.10` when ready,
  especially if relying on continuous sync or recent fixes.
- Treat `obsidian base:query`, `obsidian properties`, `obsidian tasks`, and
  `obsidian quickadd` as high-fidelity desktop-session commands.
- For cron or server flows, keep using filesystem Markdown tools plus `ob`
  rather than depending on desktop Obsidian.

## Recommendation 5: Do Not Migrate Tasks Yet

The current Tasks plugin setup is coherent:

- Tasks 8.0.0 is installed.
- Global filter is `#task`.
- Task metadata format is Dataview inline fields.
- Date tracking is enabled.
- Existing daily template already uses a Tasks query for the day view.

TaskNotes is worth knowing about, but not adopting by default. It follows a
"one note per task" model where each task is a Markdown file with YAML
frontmatter, and its views are powered by Bases. That is attractive for
calendar, agenda, Kanban, time tracking, APIs, and note-level task metadata. It
is also a real model change that would create many task notes and move away
from the existing inline task convention.

Best policy:

- Keep Tasks for inline work and daily review.
- Consider TaskNotes only for a bounded pilot, such as long-running project
  tasks that benefit from calendar/time-tracking views.
- Do not mix both systems for ordinary tasks until there is a clear rule for
  which task belongs where.

## Implementation Experiments

These are small enough to try without committing to a new system:

1. Create `inbox.base` with filters for missing `parent`, missing `status`, and
   recent modification time.
2. Configure one QuickAdd `Capture inbox` choice and bind it to a hotkey.
3. Configure one QuickAdd `New ref` template choice that always prompts for
   `parent` and `status`.
4. Add one Web Clipper "Reading Queue" template that creates `status: unread`
   reference notes.
5. Upgrade `obsidian-headless` after checking `ob sync-list-local` and the
   current vault setup.
6. Add a weekly review note or Base embed that shows:
   - unread/wip refs,
   - notes missing `parent`,
   - active work notes,
   - scheduled/due `#task` items from the Tasks plugin.

## Sources

- Obsidian Bases introduction:
  https://obsidian.md/help/bases
- Obsidian Bases views:
  https://obsidian.md/help/bases/views
- Obsidian Bases syntax:
  https://obsidian.md/help/bases/syntax
- Obsidian CLI:
  https://obsidian.md/help/cli
- Obsidian Headless:
  https://github.com/obsidianmd/obsidian-headless
- QuickAdd getting started:
  https://quickadd.obsidian.guide/docs/
- QuickAdd CLI:
  https://quickadd.obsidian.guide/docs/Advanced/CLI/
- QuickAdd URI sync limitation:
  https://quickadd.obsidian.guide/docs/Advanced/ObsidianUri/
- Obsidian Web Clipper:
  https://obsidian.md/help/web-clipper
- Web Clipper templates:
  https://obsidian.md/help/web-clipper/templates
- Web Clipper variables:
  https://obsidian.md/help/web-clipper/variables
- Tasks plugin:
  https://community.obsidian.md/plugins/obsidian-tasks-plugin
- TaskNotes docs:
  https://tasknotes.dev/
- TaskNotes community plugin page:
  https://community.obsidian.md/plugins/tasknotes
