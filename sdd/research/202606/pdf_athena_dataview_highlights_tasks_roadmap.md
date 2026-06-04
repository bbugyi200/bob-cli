---
create_time: 2026-06-04
status: research
topic: PDF→Athena mirroring, Dataview task model, and tasks defined in Highlights notes
---
# Research: Bob/Obsidian Knowledge Workflow — PDFs, Dataview, and Highlights Tasks

## Question

Bryan's inbox asks to:

1. Start copying Obsidian PDF files over to Athena somehow.
2. Refresh memory on Dataview.
3. Add support for defining tasks in Highlights app notes.

This memo connects the existing `bob highlights` reference-note sync, the
Dataview task model, the Highlights sidecar/annotation outputs, and the
Athena/headless sync constraints, then proposes a practical roadmap for turning
PDF reading/annotation into actionable Obsidian tasks and searchable reference
notes.

## Short Answer

- **PDFs → Athena:** Split the heavy PDF binary channel away from Obsidian
  Sync. Let Obsidian Sync carry only lightweight Markdown (notes, generated
  `ref/` notes, sidecars). Move PDFs to Athena over a separate one-way channel —
  git is already wired up and tracks the PDFs, so the lowest-risk first step is
  "Athena pulls; the Mac is the only writer." Use Obsidian Sync **selective
  sync** to exclude the PDF file type / `lib` + `old_lib` folders so the same
  815 MB of binaries is not pushed through both channels.
- **Dataview task model:** Tasks captured from Highlights should be real
  `- [ ] #task ...` Markdown lines inside the managed Highlights region, using
  Tasks' Dataview-format **bracket inline fields** (`[p:: 2]`, `[due:: …]`),
  each carrying `#task` (the configured global filter) and a stable `^h-…` block
  id, rendered one-way from the PDF marker / sidecar.
- **Relation to `bob-cli`:** This is a **new renderer stage inside the existing
  generated-body contract**, not a new pipeline. It reuses the marker grammar,
  sidecar parser, managed `<!-- highlights:begin/end -->` region, content-hash
  block ids, atomic writes, dirty-vault refusal, and dry-run-by-default safety
  that `bob highlights` already ships.
- **First vs deferred:** First, formalize the PDF sync-channel split and ship a
  one-way Highlights→task extraction MVP. Defer two-way task write-back to PDFs,
  date inference, a generic bulk task-property mutator, and any scheduled
  automation.

## Verified Local Context (checked 2026-06-04)

Host and topology:

- This Linux host is **Athena** (`robots.md`: `%athena | Desktop Machine Name`;
  `linux.md`: `A = [[athena]]`). The `~/bob` checkout inspected here is the
  Athena copy of the vault.
- The **Highlights app is macOS-only**, so annotated PDFs originate on the
  MacBook. `docs/highlights-ref-sync.md` documents the intended Mac checkout at
  `~/projects/bob-cli` and Mac vault paths `~/bob/lib` (PDFs) and `~/bob/ref`
  (generated notes).

Vault sync surface:

- `~/bob` is **dual-synced**:
  - Obsidian Sync core plugin is enabled (`.obsidian/core-plugins.json` →
    `"sync": true`).
  - Git remote `origin = git@github.com:bbugyi200/bob.git`, branch `master`.
- `~/bob/.gitignore` explicitly **allows** `*.pdf` and other attachments, and
  PDFs are actually tracked: `git ls-files` shows 659 PDFs under `old_lib/` and
  the active PDFs under `lib/`. So PDFs already travel by git today.

PDF library size and shape:

- **815 MB across 665 PDFs.** Distribution: `old_lib/` = 660 (archive:
  `blogs` 267, `docs` 223, `papers` 105, plus `chat`, `slides`, `code`,
  `books`, …); active Highlights `lib/` = 5 (`chat`, `docs`, `papers`).
- Largest PDFs are ~4.96 MB — just under the Obsidian Sync **Standard 5 MB
  per-file cap**. Other media (png/jpg/mp4) is ~52 MB / 174 files.
- `~/bob` also has 5,695 Markdown notes. `lit/` holds older literature/book
  notes (Markdown), distinct from `lib/` (new PDF library) and `ref/`
  (generated reference notes).

Generated reference note shape (example
`ref/blogs/steve_kinney_agent_memory.md`):

```markdown
---
status: wip
parent: "[[memory_ref]]"
type: "[[ref]]"
ref_type: blogs
source_pdf: lib/blogs/steve_kinney_agent_memory.pdf
highlights_marker_hash: …
pipeline_version: highlights-ref-mvp-3
---

# steve kinney agent memory

- [ ] #task [[lib/blogs/steve_kinney_agent_memory.pdf]] [p::2] ^task

## Highlights

<!-- highlights:begin -->

<!-- highlights:end -->
```

Task model (from `sdd/research/202606/bulk_obsidian_task_properties.md` +
local checks):

- Tasks plugin `8.0.0`, `taskFormat: "dataview"`, `globalFilter: "#task"`,
  automatic created/done/cancelled dates enabled. Dataview `0.5.68`.
- Task-line metadata uses **Dataview bracket inline fields**:
  `[created:: …]`, `[scheduled:: …]`, `[start:: …]`, `[due:: …]`,
  `[completion:: …]`, `[cancelled:: …]`, `[priority:: …]`, `[repeat:: …]`,
  `[id:: …]`, `[dependsOn:: …]`, plus Bob-custom fields like `[p:: N]`.
- `bob dataview --format json --query 'TASK WHERE contains(tags, "#task")'`
  returns task objects with `path`, `line` (zero-based), `status`, `text`,
  `tags`, `link`, `section`, `blockId`, and inline-field values. This is the
  headless verification path on Athena (no running Obsidian required).

Headless sync constraints:

- Neither `bob dataview` nor `bob highlights` runs `ob sync`. The `bob nightly`
  gate owns `ob sync` orchestration; `bob highlights doctor` only *reports*
  whether `ob` is available. So PDF mirroring and task extraction must both be
  verifiable headlessly and must not assume a live Obsidian/Highlights app.

## Dataview Refresher (the parts that matter here)

`bob dataview` (see `docs/dataview.md`) evaluates Dataview source expressions
and DQL against the local Markdown vault with a **native** engine (default),
falling back to `--engine obsidian` only when exact plugin behavior is needed.
Relevant facts for this roadmap:

- Native engine supports `LIST`, `TABLE`, `TASK`, `CALENDAR`, `FROM`, `WHERE`,
  `SORT`, `GROUP BY`, `FLATTEN`, `LIMIT`, and common functions — enough to
  select and audit generated tasks on Athena without a desktop app.
- **Task rows inherit page frontmatter.** A `TASK` over the whole vault returns
  ~10.6k rows, but most carry *inherited* `generated_from_zorg`. Restricting to
  explicit `#task` rows returns ~372. Lesson for new generated tasks: always
  scope queries by `#task` (the global filter) and, where useful, by the
  `ref_type`/`source_pdf` page fields the ref note already carries.
- Stdout is results-only (no sync logs), so generated-task queries compose
  cleanly into scripts.
- Task/list-item metadata **must** use bracket inline fields (`[k:: v]`), not
  parenthesis fields, because the field is not the only content on the line; and
  Tasks' Dataview parser only reads built-in fields from lines that match the
  `#task` global filter. Both constraints apply to anything `bob highlights`
  generates.

## Part 1 — PDFs to Athena without corrupting Obsidian Sync

### The actual problem

`~/bob` is synced by two systems at once. Obsidian Sync is a real-time,
quota-bounded service (Standard = 5 MB/file and a limited storage budget; Plus =
200 MB/file and more storage). Git is an unbounded, history-preserving channel
that already carries the PDFs. Pushing 815 MB of near-cap binaries through
Obsidian Sync as well as git is the failure mode:

- Per-file cap: the largest PDFs are within a rounding error of the 5 MB
  Standard cap; any future >5 MB scan/annotation export silently fails to sync.
- Storage budget: 815 MB of PDFs plus 5.7k notes pressures the Sync quota and
  makes the Mac↔mobile experience slow.
- **Conflict surface:** if Athena writes a PDF (e.g. marker write-back) while
  Obsidian Sync also has that PDF, Sync produces `*.sync-conflict-*` siblings,
  and git + Sync can disagree about which copy is canonical.

### Recommendation: one writer, two narrow channels

1. **Designate the Mac as the sole PDF/marker writer.** Highlights and
   `bob highlights … --write-pdf` run only on the Mac. Athena treats PDFs as
   read-only inputs. This preserves the existing "PDF marker writes are opt-in,
   keep apps idle" safety model and removes cross-host PDF write races by
   construction.
2. **Carry PDFs to Athena over git only (already wired).** PDFs are tracked and
   pushed to `origin`. Athena gets them with a normal `git pull`. No new
   mechanism is required for the *binary* hop — it already exists; it just needs
   to be the *designated* hop.
3. **Take PDFs out of Obsidian Sync** via Settings → Sync → **Selective sync**:
   disable the **PDF** file type (and/or exclude `lib`, `old_lib` folders). Then
   Obsidian Sync carries only Markdown — the notes, generated `ref/` notes, and
   `.md` sidecars — which is small, well under caps, and conflict-light. Caveat
   from Obsidian docs: excluding a type does **not** retroactively delete
   already-synced PDFs from the remote vault; do a deliberate one-time prune if
   reclaiming Sync storage matters.
4. **Keep `.md` sidecars on the Markdown channel.** The Highlights `.md`
   sidecars next to each PDF are small and are the actual sync source for note
   bodies; they should ride Obsidian Sync + git like any other note, even though
   the PDFs beside them do not ride Obsidian Sync.

Net effect: Markdown (including everything `bob highlights` reads to render
notes) stays real-time via Obsidian Sync; the heavy, write-once PDF binaries
flow Mac→git→Athena one-way. Obsidian Sync's expectations (small files, no large
binary churn, no foreign writers) are respected.

### Optional hardening (defer unless needed)

- If git history bloat from 815 MB of binaries becomes a problem, move PDFs to a
  dedicated channel (a separate `bob-pdfs` repo, `git-annex`/LFS, or a plain
  `rsync` mirror Mac→Athena) and keep only Markdown in `bob.git`. This is a
  bigger migration; do not do it as part of the first step.
- `bob highlights doctor` could grow an **Athena-side mirror check**: verify
  every `source_pdf` referenced by a `ref/*.md` note actually exists locally, so
  a missing pull is detected before a query dereferences a dead PDF link. This
  is read-only and fits doctor's existing "report, never write" contract.

## Part 2 — Dataview task model for tasks captured from Highlights notes

### Where the tasks come from

Bryan already authors action items inside Highlights. The documented linked
sidecar fragment in `docs/highlights-ref-sync.md` shows a user comment after a
highlight:

```md
> It only writes the PDF marker when marker write-back is needed…

- Support sase tool call replay?
```

Today that line is rendered as a one-way highlight **comment**. The feature
request is to let such lines become real Obsidian **tasks**. Two capture
surfaces are available, both already parsed by the pipeline:

- **Sidecar annotations** (per-highlight comments and standalone notes), which
  carry page context and can sit next to the highlight they relate to.
- **The page-1 marker note** (currently `status`/`parent`/user properties),
  which is the natural home for document-level tasks.

### Representation

Generated tasks should be real Markdown task lines **inside the managed
`<!-- highlights:begin/end -->` region**, modeled on the existing `^task`
completion line:

```markdown
- [ ] #task Support sase tool call replay [p:: 2] [src:: [[lib/chat/highlights-ref-sync.pdf|p.2]]] ^h-<hash>
```

Rules, derived from the verified task model:

- **Always include `#task`** — Tasks' Dataview parser only reads built-in
  fields from lines matching the `globalFilter: "#task"`. Without it the task is
  invisible to Tasks queries/UI.
- **Bracket inline fields only** (`[p:: 2]`, `[due:: 2026-06-10]`), never
  parenthesis fields. Separate adjacent fields with two spaces (Tasks'
  convention) so Live Preview renders cleanly.
- **Stable `^h-…` block id last**, reusing the existing content-hash scheme
  (over source PDF path, page label, annotation kind, sidecar order, and task
  text). This makes generated tasks idempotent and lets edits to surrounding
  comments not churn the id — exactly the property the highlight blocks already
  have. Insert any new inline field *before* the trailing block id.
- **Provenance back-link** to the originating highlight/page (`[src:: …#page=N]`
  or `[[lib/…pdf]]`), so a Dataview `TASK` row can navigate to the PDF location.
- **Inherited page fields are free metadata.** Because Dataview tasks inherit
  frontmatter, every generated task automatically exposes the ref note's
  `ref_type`, `parent`, and `source_pdf`. Queries can do
  `TASK WHERE contains(tags,"#task") AND ref_type = "papers"` without writing
  those fields per line.
- **Priority mapping:** the marker can optionally set a default priority; absent
  one, mirror the existing `[p:: 2]` default the `^task` line already uses.

### Marker / sidecar grammar for tasks

Keep it small and unambiguous to fit the existing parser:

- A **sidecar** task is a standalone note or highlight-comment line that begins
  with a task sentinel — reuse Obsidian's own `- [ ] ` checkbox, or a
  lightweight `TODO:`/`- [ ] ` prefix — so non-task comments keep their current
  one-way rendering and only opted-in lines become `#task` lines.
- A **marker** task is a marker-list item under a reserved key (e.g.
  `- tasks: [...]` or repeated `- task: …` lines), kept out of the synced
  user-property projection the same way `type`/`ref_type` are excluded today.
- Parsing should be **YAML-subset tolerant** like the existing marker values,
  and reject ambiguous shapes with a clear error, matching the pipeline's
  current failure-message style.

### Sync direction

Generated tasks are **one-way (PDF/sidecar → note)**, exactly like highlights,
comments, and standalone notes today. This avoids opening a second two-way
conflict surface beyond marker/frontmatter. Completion can still flow back
through the *existing* affordance: a `[x]`/`[X]` on the generated `^task` line
already signals `status: read`. Extending per-task completion write-back to the
PDF is possible but is a deferred, opt-in, `--write-pdf`-gated step (see Part 4),
because it multiplies the marker-write race surface.

## Part 3 — Relationship to existing `bob-cli` Highlights work

This feature is an **incremental renderer stage**, not a new subsystem. It
inherits, and must not break, the contracts in `docs/highlights-ref-sync.md`:

- **Reused as-is:** marker grammar + required `status`/`parent`; simple and
  linked sidecar shapes; the managed `<!-- highlights:begin/end -->` region as
  the only tool-owned body; `^h-` content-hash block ids with `### Removed
  highlights` tombstones; atomic temp-file-rename note writes; byte-identical
  skip; dirty-vault refusal; output-collision detection; `scan`/`sync`/`marker`/
  `doctor` commands; dry-run-by-default; `--write-pdf`/`--write-pdfs` opt-in for
  any PDF mutation.
- **Extended:** the generated body gains a `## Tasks` (or inline) block of
  `#task` lines alongside `## Highlights`. The asymmetric sync model is
  preserved — tasks are generated content; user edits inside the managed region
  may be overwritten, so durable task metadata must live in the PDF/sidecar
  source, not be hand-added inside the managed block.
- **Unchanged ownership:** `bob highlights` still does not run `ob sync`;
  `bob nightly` owns sync orchestration. Generated tasks become queryable on
  Athena purely through `bob dataview` against synced Markdown.
- **CLI conformance** (per `memory/long/cli_rules.md`): any new flags (e.g.
  `--tasks/--no-tasks` to toggle extraction) need a short alias, alphabetical
  ordering in help, and clean colored `-h/--help` output. Prefer a default-on
  extraction with a `--no-tasks` escape hatch over a new subcommand, since this
  is a rendering option on the existing `sync`/`scan` surface.

A separate, larger idea from `bulk_obsidian_task_properties.md` — a generic
`bob task-prop set` mutator for arbitrary task-line fields across files — is
**related but out of scope** here. Highlights task extraction writes only inside
the managed region of generated notes; the bulk mutator edits arbitrary
user-authored task lines anywhere. Keep them separate.

## Part 4 — What to implement first vs defer

### First (high leverage, low risk)

1. **Formalize the PDF sync-channel split.** Mac = sole PDF writer; Obsidian
   Sync selective-sync excludes the PDF type / `lib`+`old_lib`; Athena pulls
   PDFs via existing git. This is mostly configuration + documentation and
   directly satisfies "copy PDFs to Athena somehow" while protecting Obsidian
   Sync. No `bob-cli` code required for the MVP.
2. **One-way Highlights→task extraction MVP.** Parse opted-in task lines from
   sidecar annotations (and optionally marker task items) into `#task` Markdown
   lines inside the managed region, with `[p:: N]`, a `^h-…` block id, and a PDF
   back-link. Default-on rendering, `--no-tasks` to disable, dry-run shows the
   planned task lines, writes reuse the existing atomic/dirty-refusal path.
3. **Headless verification recipe.** A documented `bob dataview --format json
   --query 'TASK WHERE contains(tags,"#task") AND source_pdf'` (or scoped by
   `ref_type`) that proves generated tasks are queryable on Athena.

### Defer (until the MVP is proven)

- **Per-task completion write-back to the PDF** beyond the existing single
  `^task` toggle — multiplies the marker-write race surface; keep PDF writes
  opt-in and document the risk first.
- **Date/priority inference** (auto `[due:: ]`/`[scheduled:: ]` from highlight
  text) — start with explicit fields only.
- **Generic `bob task-prop set`** bulk task-property mutator — separate tool,
  separate research already exists.
- **Scheduled automation on Athena** (cron/launchd pulls, auto-`scan`) — land
  the manual flow first; the Mac-side scheduled `scan --dry-run` LaunchAgent in
  the docs is the template when ready.
- **Dedicated PDF storage channel** (LFS/git-annex/`rsync` mirror, separate
  repo) — only if git binary bloat becomes a real problem.
- **`doctor` Athena mirror check** — nice-to-have, read-only, after the channel
  split is in place.

## Open Questions for Bryan

- Which **Obsidian Sync plan** is active (Standard 5 MB vs Plus 200 MB)? It sets
  whether any current PDF already fails to sync and how urgent the channel split
  is.
- Should the **task capture surface** be sidecar comments, the page-1 marker, or
  both? MVP can start with sidecar `- [ ]` lines only.
- Is there appetite to eventually move PDFs **out of `bob.git`** into a dedicated
  channel, or is git-tracked-PDF acceptable long-term?

## Sources

- Local: `docs/highlights-ref-sync.md`, `docs/dataview.md`
- Local: `sdd/research/202606/bulk_obsidian_task_properties.md`
- Local: `memory/long/cli_rules.md`
- Local verification: `~/bob` git remote/tracking, `.obsidian/core-plugins.json`
  + `community-plugins.json`, `.gitignore`, PDF size/distribution counts, and
  `ref/blogs/steve_kinney_agent_memory.md` (checked 2026-06-04).
- Obsidian Help — Sync settings and selective syncing:
  https://help.obsidian.md/sync/settings
- Obsidian Help — Plans and storage limits:
  https://help.obsidian.md/Obsidian+Sync/Plans+and+storage+limits
</content>
</invoke>
