---
create_time: 2026-06-04
status: research
topic: Bulk updates to Obsidian task properties across files
---
# Research: Bulk Obsidian Task Property Updates

## Question

Given a list of Obsidian tasks that live in different Markdown files, what is
the best way to set the same per-task property on all of them in bulk?

## Answer

For Bryan's vault, the best approach is a small dry-run bulk rewrite tool:

1. Use Dataview, preferably `bob dataview --format json`, to identify the exact
   source tasks by vault-relative path and zero-based line number.
2. Group selected tasks by source file.
3. Read each file once, verify each selected line still matches the expected
   task text/status, then replace or append the target Dataview inline field.
4. Preserve Markdown structure, line endings, existing fields, and trailing
   block IDs.
5. Show a diff before writing, then write only the verified candidate files.

This is better than trying to do it in the Obsidian UI because the Tasks plugin
does not expose a general "bulk set property" command for arbitrary selected
tasks. It is also better than a naive `rg | sed -i` edit because task lines have
structured fields, block IDs, custom statuses, and stale line-number risk.

## Verified Local State

Checked on 2026-06-04:

- The Obsidian vault is `~/bob`.
- Obsidian Headless is available through `ob`; it is useful for Sync freshness,
  not as a community-plugin runtime.
- `~/bob/.obsidian/plugins/obsidian-tasks-plugin/manifest.json` reports Tasks
  `8.0.0`.
- Tasks settings use `globalFilter: "#task"` and `taskFormat: "dataview"`.
- Tasks has automatic created, done, and cancelled date tracking enabled.
- Dataview is installed at `0.5.68`.
- `bob dataview --format json --query 'TASK FROM #task LIMIT 2'` returns task
  objects with `path`, `line`, `status`, `text`, `tags`, and other metadata.
- The Dataview task `line` value is zero-based. A task returned with
  `"line": 26` was physically on line 27 in `nl -ba` output.
- Existing Bob task fields commonly use Dataview-style bracketed inline fields,
  for example `[p::1]`, `[scheduled:: 2026-06-04]`, `[completion:: 2026-06-03]`.

## Task Property Format

Because Bryan's Tasks plugin is configured with `taskFormat: "dataview"`, new
task properties should be represented as bracketed Dataview inline fields on the
task line:

```markdown
- [ ] #task Follow up with Kelly  [scheduled:: 2026-06-08]  [priority:: high]
```

For built-in Tasks fields, use the names Tasks documents for Dataview format:

| Intended field | Dataview task-line field |
| --- | --- |
| Created date | `[created:: YYYY-MM-DD]` |
| Scheduled date | `[scheduled:: YYYY-MM-DD]` |
| Start date | `[start:: YYYY-MM-DD]` |
| Due date | `[due:: YYYY-MM-DD]` |
| Done date | `[completion:: YYYY-MM-DD]` |
| Cancelled date | `[cancelled:: YYYY-MM-DD]` |
| Priority | `[priority:: lowest|low|medium|high|highest]` |
| Recurrence | `[repeat:: every day]` |
| On completion | `[onCompletion:: keep|delete]` |
| Task id | `[id:: abc123]` |
| Dependencies | `[dependsOn:: abc123,def456]` |

For custom metadata such as `p`, `snooze`, or other Bob-specific values, use the
same bracketed syntax:

```markdown
- [ ] #task Example  [snooze:: 2026-06-10]  [p:: 1]
```

Two spacing details matter:

- Separate adjacent square-bracket Dataview fields with either two spaces or a
  comma-space. Tasks' own modal writes two spaces.
- Insert new fields before a trailing Obsidian block ID such as `^abc123`, so
  the block ID remains the final structural marker on the line:

```markdown
- [ ] #task Example  [scheduled:: 2026-06-08] ^abc123
```

## What Existing Tools Can and Cannot Do

### Tasks Plugin UI

The Tasks `Create or edit task` modal is the right UI for a single task. If the
cursor is on an existing task, it modifies that task; if the cursor is on a
blank line, it creates a task.

The modal is not a general bulk editor. The documented exception is dependency
editing: when saving dependencies, Tasks may add `id` fields to depended-on
tasks and adjust `dependsOn` fields, potentially across files. That is specific
to dependencies and is not a general "set this field on these N tasks" workflow.

### Tasks Query Blocks

Tasks query blocks are good selectors and dashboards. They can filter by built-in
properties, custom JavaScript expressions, `task.originalMarkdown`, file path,
tags, and status. They can show an edit button per task.

They still do not provide a bulk-update action. They are a good way to confirm
which tasks would be selected, not the bulk mutation mechanism.

### Dataview

Dataview can query task-level metadata through `TASK` queries. It indexes tasks
with implicit fields such as status, completion booleans, text, source path,
line, tags, links, children, and block IDs. It also indexes inline fields on task
lines.

Dataview is primarily for display/query. A `TASK` query can update the source
file when a rendered checkbox is checked, but Dataview does not generally edit
arbitrary inline fields. Use it to select exact tasks, then let a rewrite tool
edit Markdown.

### Obsidian CLI

The official Obsidian CLI can list tasks with file paths and line numbers, and
the `task` command can show or update a task by `path:line`. Its documented task
updates cover status/toggle/done/todo, not arbitrary task-line properties. The
CLI's `property:set` command targets note frontmatter properties, not individual
task-line fields.

So the Obsidian CLI is useful if the property is literally task status, or as a
source of task references. It is not enough for "set `[scheduled:: ...]` on this
list of tasks".

### Frontmatter Bulk-Property Plugins

Bulk property plugins and Obsidian Bases workflows are about note properties
stored in frontmatter. They do not solve per-task metadata unless each task is a
separate note. Bryan's vault uses ordinary Markdown task lines, so this would be
the wrong layer for this specific problem.

## Recommended One-Off Workflow

Use this for a task set that can be expressed as a Dataview query:

```bash
bob dataview --format json --query '
TASK
WHERE !completed
  AND contains(tags, "#sase")
  AND !scheduled
' |
jq -r '.result.values[] | [.path, .line, .status, .text] | @tsv'
```

Then feed that TSV or JSON into a purpose-built updater that:

- treats `line` as zero-based when using `bob dataview` output;
- re-reads the target file before editing;
- verifies the selected line is still a Markdown task;
- verifies the selected line still contains the expected task text or original
  Markdown;
- replaces an existing `[key:: ...]` field if present;
- otherwise appends `  [key:: value]` before a trailing ` ^block-id` if present;
- preserves other inline fields, links, tags, and line endings;
- reports a dry-run diff by default.

For a list already copied from Obsidian, normalize it to one of these stable
forms first:

```text
path/to/file.md:27
path/to/file.md:27<TAB>expected task text
path/to/file.md<TAB>26<TAB>expected task text
```

Be explicit about whether the line numbers are one-based or zero-based:

- `bob dataview` and Tasks `task.lineNumber` are zero-based.
- Obsidian CLI task references are documented as `path:line`; Tasks docs warn
  that Obsidian CLI task line numbers begin at 1.

## Rewrite Algorithm

Recommended implementation shape:

1. Build a candidate list.
2. Check `git -C ~/bob status --short` and refuse to edit dirty candidate files
   unless explicitly overridden.
3. For each candidate, store `path`, zero-based `line`, and expected old task
   text.
4. Group candidates by `path`.
5. Read the whole file as bytes/text and preserve its newline style.
6. For each candidate line:
   - verify the line starts like a Markdown task after indentation:
     `- [ ]`, `- [x]`, `- [-]`, `* [ ]`, `+ [ ]`, or an ordered task marker;
   - verify the line still includes the expected old text;
   - if setting the checkbox status, change only the checkbox marker;
   - if setting an inline field, replace the existing bracketed field with the
     same key, or append a new field before any trailing block ID.
7. If multiple selected tasks are in one file, rewrite the in-memory line array
   once instead of editing the file repeatedly.
8. Print a summary and unified diff.
9. Write only after `--apply`.
10. Run a verification query showing the selected tasks now have the expected
    property.

Insertion rule for a field named `scheduled`:

```text
before: - [ ] #task Call Pat ^pat-call
after:  - [ ] #task Call Pat  [scheduled:: 2026-06-08] ^pat-call
```

Replacement rule:

```text
before: - [ ] #task Call Pat  [scheduled:: 2026-06-07] ^pat-call
after:  - [ ] #task Call Pat  [scheduled:: 2026-06-08] ^pat-call
```

## Safety Notes

- Do not depend on line numbers alone. They go stale as soon as a source file is
  edited. Always validate the current line content before rewriting.
- Prefer a block ID or unique task `id` for repeatable bulk operations. If tasks
  are going to be updated repeatedly, first assign stable IDs.
- Do not use plain `sed -i` to append text to every matching line unless the
  property is disposable and the selector is trivial. It is too easy to update
  completed/cancelled tasks, code examples, nested non-task lines, or lines that
  already have the property.
- Bracketed Dataview fields are the safest per-task metadata shape. Bare tokens
  like `ID::foo` inside a task description may be plain text rather than a
  Dataview field attached to the task.
- For custom fields, Tasks may not expose a typed property like `task.snooze`.
  In a Tasks query, use `task.originalMarkdown` if needed. In Dataview, query
  the inline field directly.
- If Obsidian Sync is involved, run or wait for the normal `ob` sync workflow
  before and after the edit. Avoid editing while another Obsidian instance is
  actively changing the same files.

## Best Fit for Bob

The cleanest Bob-specific implementation would be a general native command, not
a one-off shell snippet:

```bash
bob task-prop set --property scheduled --value 2026-06-08 --query-file /tmp/tasks.dql --dry-run
bob task-prop set --property scheduled --value 2026-06-08 --tasks-file /tmp/task-list.tsv --apply
```

Minimum command contract:

- Accept a Dataview `TASK` query or an explicit task list.
- Support `--dry-run` by default and `--apply` for writes.
- Support `--bob-dir`.
- Refuse dirty candidate files.
- Stage/commit/push only if a future workflow deliberately wants automation;
  otherwise leave the diff for review.
- Reuse the existing `bob dataview` native index for selection.
- Share tested helpers with any future command that removes or rewrites task
  inline fields.

This can start as a small script if it is a one-time cleanup. If this need comes
up more than twice, it is worth promoting into `bob-cli` with tests.

## Sources

- [Tasks Dataview Format docs](https://github.com/obsidian-tasks-group/obsidian-tasks/blob/main/docs/Reference/Task%20Formats/Dataview%20Format.md)
- [Tasks Create or edit Task docs](https://github.com/obsidian-tasks-group/obsidian-tasks/blob/main/docs/Editing/Create%20or%20edit%20Task.md)
- [Tasks Task Properties docs](https://github.com/obsidian-tasks-group/obsidian-tasks/blob/main/docs/Scripting/Task%20Properties.md)
- [Tasks plugin README](https://github.com/obsidian-tasks-group/obsidian-tasks)
- [Dataview Adding Metadata docs](https://blacksmithgu.github.io/obsidian-dataview/annotation/add-metadata/)
- [Dataview Metadata on Tasks and Lists docs](https://blacksmithgu.github.io/obsidian-dataview/annotation/metadata-tasks/)
- [Dataview TASK query docs](https://blacksmithgu.github.io/obsidian-dataview/queries/query-types/#task)
- [Obsidian CLI docs](https://obsidian.md/help/cli)
- Local: `docs/dataview.md`
- Local: `sdd/tales/202606/snooze_task_property.md`
