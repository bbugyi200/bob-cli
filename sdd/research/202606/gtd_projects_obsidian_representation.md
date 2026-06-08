---
create_time: 2026-06-08
status: research
topic: How to best represent GTD projects in the Bob Obsidian vault
---
# Research: Representing GTD Projects in the Bob Obsidian Vault

## Question

How should Bryan best represent **projects**, in the sense GTD defines them, in
the `~/bob` Obsidian vault?

## Short Answer

Bob already implements most of GTD's project machinery — but at the *Zorg* layer,
not as an *Obsidian-queryable* model. The highest-leverage change is **not** a new
project system; it is to make the projects Bob already has machine-readable and
reviewable inside Obsidian:

1. Adopt one **canonical project record** = a note carrying a small, reliable
   frontmatter schema (`type: project`, `project_key`, `status`, `area`,
   `outcome`, `parent`). Identity stops depending on `prj_` naming + a `@PRJ`
   body marker.
2. Keep the existing **`+marker` as the project-membership join key** — it
   already tags every task with its project. Formalize it as `project_key` and,
   for genuine `#task` checkboxes, surface it as an inline `[project:: key]`.
3. Build the **three canonical GTD project views** with tooling already enabled:
   a `projects.base` Projects List (Horizon 1), a per-project task view, and
   cross-project Next-Action / Waiting-For dashboards — including a **stalled-
   project** view (active projects with no next action), which is GTD's core
   weekly-review check.
4. Map the project **lifecycle** (active / someday / paused / done) onto the
   `status` field so the existing open/close/pause checklists set one queryable
   value instead of being implied by which file a note lives in.
5. Emit all of this from the **Zorg→Obsidian converter** (or pilot it in the
   writable `_pilot/` area) — never by hand-editing mirrored notes, which are
   regenerated.

The recommended shape is a **hybrid**: lightweight membership tag + one canonical
record per project + a native view layer. Pure note-per-project is too heavy for
~200 projects; tag-only (today's state) is too weak to query.

## Verified Local Context

Checked against `~/bob` on 2026-06-08 (and prior counts from sibling research
dated 2026-06-04):

- **Bob is a generated, read-only mirror of Zorg.** `_meta/Bob Home.md` states
  the vault "is a generated, read-only mirror of selected Zorg material.
  `/home/bryan/org` remains the source of truth." Every project note carries
  `generated_from_zorg: true`, a `zorg_source`, and a `zorg_converter`
  (`convert_zorg_core.py`, version `2026-05-27.phase3`).
- **The GTD horizons are already modeled.** `prj.md` is titled *"Master @PRJ List
  (GTD Horizon 1)"*. `now.md`, `prj.md` have `parent: [[horizon]]`. Navigation
  notes exist for `inbox`, `now` (+ `now_work`, `now_dev`, `now_gtd`, `now_own`,
  `now_prjs`, …), `soon` (P2/P3), and `wait` (`@WAIT`). This is Allen's Horizons
  of Focus, hand-built.
- **Projects are `+markers`.** A task belongs to a project via a `+name` marker
  on the task line (e.g. `o 240701 231201#0o +aut Ad Unit Test …`). The
  generated index `_generated/tag_pages/project.md` normalizes these to
  `#project/<name>` and counts **200+ distinct project markers** (e.g. `+rap`
  1161 occurrences, `+gbd` 856, `+neovim` 772, `+zorg` 757, … down a long tail
  of `+laundry`, `+savings`, `+veneers`, …).
- **Only ~14 projects have a heavyweight hub note.** Root-level `prj_*.md`:
  `prj_bbxo, prj_bs_allow, prj_gbd, prj_gtd, prj_ilar, prj_mcr_cats,
  prj_pa_trouble, prj_pd_local, prj_plex, prj_protect, prj_tick, prj_work,
  prj_yserve, prj_zorg`. These are rich planning hubs (Related notes, meeting
  notes, reference sections, sub-task lists).
- **Project identity is inconsistent in Obsidian terms.** Across the whole vault
  only **2 notes** carry `type: project` frontmatter; the `prj_*` hubs are
  instead identified by the `prj_` filename prefix + a `@PRJ` body marker +
  their `+marker`. There is no reliable `status`, `area`, or `outcome`
  frontmatter on project notes. Frontmatter `type:` is dominated by
  `"[[ref]]"` (301) and `restaurant` (91); `area:` appears on only 3 notes.
- **The generated `#project/<name>` index is a marker tally, not a GTD record.**
  It lists normalized tag, source marker, occurrence count, and a link — but no
  outcome, status, or next action per project.
- **Two task syntaxes coexist, and Obsidian sees almost none of the project
  work.** Zorg todos render as org-style lines (`o` open, `x` done, `P0`–`P4`
  priority). The Tasks plugin uses `globalFilter: "#task"` and
  `taskFormat: "dataview"`; sibling research (2026-06-04) found only **383
  `#task` checkboxes** vault-wide (mostly highlight- and daily-generated),
  while the thousands of `o …` project todos are invisible to Tasks queries.
- **Lifecycle is already conceptualized.** `project_checklists.md` defines
  `open_project_checklist`, `close_project_checklist`, and
  `pause_project_checklist`; the pause flow moves a project under
  `[[paused_projects]]`, and `gtd.md` references `[[done_projects]]` for closed
  ones. A `+goals` project even reads: *"Implement horizons_of_focus … in
  zorg."*
- **The view tooling is present and partly used.** Community plugins include
  `dataview` (0.5.68), `obsidian-tasks-plugin` (8.0.0, custom statuses
  `/`=IN_PROGRESS, `B`=ON_HOLD, `-`=CANCELLED), `templater`, and `quickadd`.
  Core **Bases** is enabled with a real `refs.base` dashboard. `bob dataview`
  gives headless DQL access for verification.
- **A writable pilot surface exists.** `_meta/Active Workflow Pilot.md` gates
  active (non-mirror) writes to `_pilot/capture/` only, pending rollback review.

## What GTD Actually Requires of a "Project"

From the source methodology (David Allen):

- A **project** = *any desired outcome that takes more than one action step* and
  is realistically completed within ~a year. This is a low bar — a healthy GTD
  system typically has 30–100+ active projects, which matches Bob's 200-marker
  reality far better than its 14 hub notes.
- The **Projects List** is a single, complete inventory of those multi-step
  outcomes. Its job is reviewability, not storage of the work itself.
- Every active project must have **at least one Next Action** living on a context
  list (`@work`, `@calls`, …). The weekly review's central check is: *does each
  project still have a next action, or has it stalled?*
- Projects sit at **Horizon 1**; **Areas of focus/accountability** are Horizon 2.
  Bob's `#context` tags (`#work`, `#dev`, `#own`, `#love`, `#body`, `#mind`,
  `#gtd`) are effectively its Horizon-2 areas, and `parent` frontmatter already
  threads notes up toward `[[horizon]]`.

The standard Obsidian-community pattern that operationalizes this (Manning,
alangrainger/obsidian-gtd, the Obsidian forum GTD system, Calliope Sounds) is
**three coordinated views**:

1. a **Projects List** auto-generated from project notes,
2. a **per-project view** of that project's tasks, and
3. a **cross-project Next-Actions view** (often driven by `[next-action::]` /
   `[waiting-on::]` Dataview inline fields) that pulls the actionable item out of
   every project.

Bob has #1 (`prj.md`) and #3 (`now*`/`wait*`) as hand-built note lists, and #2 as
the `prj_*` hubs — but none are *queryable* because the underlying records lack
frontmatter.

## Gap Analysis (Obsidian-specific)

| GTD need | Zorg/Bob today | Obsidian-native gap |
| --- | --- | --- |
| One inventory of all projects | `prj.md` + 200-marker index | Not a query; can't filter by status/area; tail markers absent from the master list |
| Reliable "is this a project?" predicate | `prj_` name + `@PRJ` marker + `+marker` | Only 2 notes have `type: project`; Dataview/Bases can't select projects |
| Per-project status (active/someday/paused/done) | Implied by folder/parent + checklists | No `status` field to query; no "active projects" view |
| Area (Horizon 2) per project | `#context` tags on tasks | Not on the project record; can't group projects by area |
| Desired outcome sentence | Free text in hub body | Not a field; can't show "outcome" column in a list |
| ≥1 next action per project | `now*` lists, `@PRJ` todos | No view of *active projects lacking a next action* (stalled detector) |
| Tasks grouped by project | `+marker` grouping in `now_prjs.md` | Tasks plugin can't group — org `o` todos aren't `#task` |

## Options Considered

**A. Pure note-per-project (canonical Obsidian-GTD).** One note per project,
title = project name, tasks live inside, Dataview/Tasks build the three views.
*Rejected as the whole solution:* ~200 projects (many trivial: `+laundry`,
`+dryer`, `+mag`) would each need a maintained note. Too heavy; fights the
lightweight `+marker` that already works.

**B. Tag/marker-only (status quo).** Projects = `+marker` / `#project/<name>` +
generated tally. *Rejected:* not a GTD record — no outcome, status, area, or
next-action guarantee; nothing queryable; the master list and the marker tail
drift apart.

**C. Folder/PARA reorg.** Move projects into a `projects/` folder hierarchy.
*Rejected:* Bob is a generated mirror; folder layout is the converter's output,
and the `parent`-frontmatter graph already encodes hierarchy better than folders.

**D. Hybrid (recommended).** Lightweight `+marker` membership **plus** one
canonical, frontmatter-bearing project record per project **plus** a native
view layer. Heavy hubs stay heavy; the long tail gets thin generated records so
*every* project is queryable. Best fit for 200 projects, the existing tooling,
and the mirror constraint.

## Recommended Solution

### 1. Canonical project record (the data model)

Every GTD project = exactly **one** note carrying this schema. Big projects: add
these fields to the existing 14 `prj_*` hubs. Long-tail projects: have the
converter emit a thin stub record per active marker.

```yaml
---
type: project              # reliable identity predicate (replaces prj_ name + @PRJ)
project_key: gbd           # == the Zorg +marker; join key to this project's tasks
status: active             # active | someday | paused | done
area: work                 # Horizon-2 area, from #context (work/dev/own/love/...)
outcome: "Bidder declarations launched and documented"   # GTD success sentence
parent: "[[prj_work]]"     # keep the existing hierarchy
created: 2025-04-18
closed:                    # date, set only when status: done
---
```

`outcome` is the one genuinely *new* field worth authoring — it is the GTD
"successful outcome" sentence and is what makes a Projects List reviewable at a
glance. Everything else the converter can derive from signals it already parses
(`+marker`, `@PRJ`, `#context`, the open/close/pause checklists, `parent`).

### 2. Membership / linking convention

- A task belongs to a project via its **`+marker` ⇒ `project_key`**. Already
  universal in Zorg; just name it.
- For genuine `#task` checkboxes (highlights, daily capture, pilot), add an
  inline **`[project:: <key>]`** field so the Tasks plugin
  (`task.file.property('project')`) and Dataview can group by project. This
  Tasks-side hook was already identified in
  `obsidian_improvements_consolidated.md` (Finding 4).
- Do **not** mass-convert org `o`/`x` todos to `#task`. The improvements research
  already concluded inline org tasks should stay; emitting `#task` checkboxes
  from Zorg todos is a separate, deferrable decision.

### 3. The three canonical views (the view layer)

**a) Projects List → `projects.base` (Horizon 1).** Bases is native, fast, and
already enabled. One row per project record:

```yaml
# projects.base (sketch)
filters:
  and:
    - type == "project"
    - status == "active"
views:
  - type: table
    name: Active Projects
    group_by: area
    order: [outcome, project_key, file.mtime]
  - type: table
    name: Someday/Maybe
    filters: { status == "someday" }
  - type: table
    name: Stalled            # active projects with no open next action
    # (compute "has next action" via a Dataview companion query or a
    #  next_action frontmatter the converter fills; see 3c)
```

This is the master Projects List made reviewable, replacing the static `prj.md`
text list. The **Stalled** view is the GTD weekly-review detector that the
current system can't produce.

**b) Per-project view.** Inside each project record, a Tasks/Dataview block
scoped to its `project_key`, showing open / next / waiting items. For the
`prj_*` hubs this augments the hand-written org list with a live query; for the
stub records it *is* the body. Example (Dataview over the generated task index,
which sees org todos):

```dataview
TASK FROM "_generated/tag_pages/project"
WHERE project_key = this.project_key AND !completed
```

**c) Cross-project Next Actions & Waiting-For.** Keep `now*` / `wait*` as the
curated human lists, and add native dashboards. For the `#task` surface, the
pattern from the improvements doc applies directly:

```tasks
( status.type is TODO ) OR ( status.type is IN_PROGRESS )
group by function task.file.property('project')
limit groups to 3 tasks
```

Use Dataview `[next-action:: true]` / `[waiting-on:: %person]` inline fields to
mark, within a project, *which* item is THE next action vs. a delegated/waiting
item — the community-standard GTD annotation. A cross-project "what's next per
project" list then becomes a single query, and any active project missing a
`[next-action::]` falls into the Stalled view.

### 4. Lifecycle mapping (Zorg ↔ Obsidian)

Make the existing checklists set one queryable field instead of moving files:

| GTD / Zorg state | `status` | Today's signal |
| --- | --- | --- |
| Active | `active` | under `[[projects]]`/`[[prj]]`, `@PRJ` |
| Someday/Maybe | `someday` | `soon` / someday material |
| Paused | `paused` | `pause_project_checklist`, `[[paused_projects]]` |
| Done | `done` (+`closed:`) | `close_project_checklist`, `[[done_projects]]` |

Every view keys off `status`; the open/close/pause checklists each end by setting
it.

### 5. Implementation path (respect the mirror)

Because `/home/bryan/org` (Zorg) is the source of truth and Bob is regenerated:

1. **Teach the converter** (`convert_zorg_core.py` / the phase pipeline) to stamp
   `type: project`, `project_key`, `status`, `area`, `parent`, and (where
   authored) `outcome` onto every project note, and to emit thin stub records +
   `projects.base`. The `@PRJ` marker, `+marker`, `#context`, and checklist
   signals it already parses are sufficient inputs.
2. **Author `outcome` in Zorg** for at least the 14 hub projects (one sentence
   each) — the only manual content this requires.
3. **Pilot first if desired** in the writable `_pilot/` area (Phase 7 active-
   workflow pilot) before committing the converter change, so the schema and the
   `projects.base` can be validated with `bob dataview` without touching mirrored
   notes.
4. **Verify headlessly:** after each step, confirm with
   `bob dataview -q 'TABLE status, area, outcome FROM "" WHERE type = "project"'`
   that the Projects List is complete and every active project resolves a next
   action.

Do **not** hand-edit `prj_*` or generated notes in `~/bob` directly — those edits
are overwritten on the next conversion.

## Prioritized First Steps

1. Add the project-record frontmatter schema to the converter for the 14 `prj_*`
   hubs (`type`, `project_key`, `status`, `area`, `parent`).
2. Author a one-line `outcome` for each of the 14 hubs in Zorg.
3. Generate thin stub records for active long-tail `+markers` so every project is
   queryable.
4. Add `projects.base` with **Active**, **Someday**, **Paused**, and **Stalled**
   views.
5. Adopt `[next-action::]` / `[waiting-on::]` annotations and add one
   cross-project Tasks dashboard grouped by `project`.
6. Wire the open/close/pause checklists to set `status` (+ `closed:` on done).

## Open Questions

- Should long-tail markers (e.g. `+laundry`, `+veneers`) become real project
  records, or stay as pure tags with only the 14 hubs promoted? (Recommendation:
  generate *thin* records for active markers so the Projects List is complete,
  but keep them stub-light.)
- Is it worth emitting `#task` checkboxes from Zorg `o` todos so the Tasks plugin
  can drive the next-action views natively — or is the Dataview-over-org-index
  path sufficient? (Defer; start with Dataview.)
- Where should `outcome` be authored — a dedicated Zorg property on the project's
  `@PRJ` note, so it round-trips cleanly through the converter?

## Sources

Local files and commands (read-only):

- `_meta/Bob Home.md`, `_meta/Active Workflow Pilot.md` (mirror + pilot status)
- `prj.md`, `prj_gbd.md`, `prj_work.md`, `prj_zorg.md`, `prj_tick.md`, `aut.md`,
  `gtd.md`, `now.md`, `soon.md`, `wait.md`, `inbox.md`, `now_prjs.md`,
  `project_checklists.md`
- `_generated/tag_pages/project.md` (project-marker index)
- `_templates/new_note.md`, `_templates/daily.md`, `_templates/schedule.md`
- Sibling research: `sdd/research/202606/obsidian_improvements_consolidated.md`,
  `sdd/research/202606/bob_obsidian_pdf_dataview_highlights_tasks_roadmap.md`
- `git -C ~/bob` / filesystem counts for `prj_*` notes and `type:` frontmatter
- `bob dataview` (headless DQL) for verification

GTD methodology:

- Getting Things Done — Wikipedia: https://en.wikipedia.org/wiki/Getting_Things_Done
- GTD® 6 Horizons of Focus (David Allen): https://coda.io/@gtdtricks/gtd-6-horizons-of-focus-by-david-allen
- Elemental Guide to GTD: https://www.firetask.com/gtd.html
- Intro to GTD: Horizons, Projects & Context Lists: https://www.linkedin.com/pulse/intro-getting-things-done-gtd-horizons-projects-context-brian-petro

GTD-in-Obsidian patterns (three-view model, project annotations):

- alangrainger/obsidian-gtd: https://github.com/alangrainger/obsidian-gtd
- My Obsidian GTD setup (Daryl Manning): https://daryl.wakatara.com/my-obsidian-gtd-setup/
- GTD with Obsidian (Obsidian Forum, task sequencing / waiting-on / someday): https://forum.obsidian.md/t/gtd-with-obsidian-a-ready-to-go-gtd-system-with-task-sequencing-quick-add-template-waiting-on-someday-maybe-and-more/65502
- Yet another Obsidian, Dataview, and GTD exploration (Calliope Sounds): https://www.calliopesounds.com/2022/05/yet-another-obsidian-dataview-and-gtd.html
- chuckthenerd/GTD-obsidian: https://github.com/chuckthenerd/GTD-obsidian
- GTD Methodology Using Obsidian (Krishna Venkat): https://medium.com/@krishnavenkat1993/getting-things-done-gtd-methodology-using-obsidian-d6388318b84a
