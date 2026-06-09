---
create_time: 2026-06-09
status: research
topic: Enforcing and suggesting predefined Obsidian property values
---
# Research: Obsidian Property Value Enforcement

## Question

Some Obsidian note file properties have a known set of accepted values. What are
the practical options for suggesting those values during editing and enforcing
them enough that Dataview, Bases, and automation stay reliable?

## Short Answer

Obsidian's native Properties UI does not currently have a true `select` or
`enum` property type. It supports basic property types and can suggest existing
values, but those suggestions come from values already present in the vault and
are not a scoped, authoritative allowed-values list.

For Bryan's vault, the best fit is a layered solution:

1. Use **Metadata Menu** for the day-to-day editing UX: define per-note-class
   `Select`, `Multi`, or `Cycle` fields so Obsidian can suggest the right
   values from explicit lists.
2. Add a validation layer for actual enforcement. Use **Forge** if we want an
   in-Obsidian schema/lint/repair workflow with explicit enum schemas. Use a
   small `bob` validation command if we want the rules to live in this repo and
   be checked from the shell, cron, git hooks, or SASE workflows.

The main design rule is to scope schemas by note class, folder, or tag. Do not
define one global enum for property names like `status`, because the same
property name already means different things in different parts of `~/bob`.

## Local Vault Context

Observed on 2026-06-09:

- The Obsidian vault is `~/bob`.
- Installed community plugins include Dataview, Tasks, Templater, QuickAdd,
  Task Status Cycler, Note Refactor, Vimrc Support, Relative Line Numbers, and
  custom Bob plugins.
- Not installed: Metadata Menu, Meta Bind, Propsec, Forge, Data Entry.
- `~/bob/.obsidian/types.json` defines Obsidian property *types* such as
  checkbox/text/multitext/tags, but no allowed value lists.
- Existing Bases already depend on consistent values:
  - `refs.base` expects ref statuses such as `unread`, `wip`, `read`,
    `collect_fleeting_notes`, `review_fleeting_notes`, `review_lit_notes`, and
    `abandoned`.
  - `eat.base` expects restaurant statuses such as `liked` and `not_liked`.
- Therefore, `status` is context-sensitive. The same property name should have
  different allowed values under `ref/` versus `eat/`.

## Native Obsidian

Native Obsidian Properties are useful but not enough for enum enforcement.

What native Obsidian gives us:

- File properties are YAML frontmatter with typed values.
- Supported property types are Text, List, Number, Checkbox, Date, Date & time,
  and Tags.
- Once a property type is assigned to a property name, that type applies across
  the vault.
- The Properties view can list, sort, search, and globally rename property
  names.
- Bases can display and edit files and their properties in database-like table,
  list, card, and map views.
- Obsidian has property value autocomplete in places such as Properties and
  Bases, but those suggestions are derived from existing vault values.

What native Obsidian does not give us:

- No built-in `Select`, `Multi-select`, or `Enum` property type.
- No built-in way to say `status` is one of these values only.
- No built-in way to scope suggestions for a property to the current Base,
  folder, note type, or tag.
- No hard prevention against direct YAML edits, because properties can always be
  viewed and edited as source text.

Native-only is acceptable for low-stakes conventions. It is not enough for
properties that drive Bases, Dataview, custom commands, or migrations.

## Option 1: Templates, Templater, and QuickAdd

This vault already has Templater and QuickAdd. They can prompt for valid values
at note creation time.

Templater supports `tp.system.suggester(...)` and
`tp.system.multi_suggester(...)`, so a template can ask for one of:

```js
await tp.system.suggester(
  ["Unread", "Working", "Read", "Abandoned"],
  ["unread", "wip", "read", "abandoned"]
)
```

Strengths:

- Already installed.
- Good for new-note flows.
- Can use different templates per folder or note type.
- Easy to keep template defaults aligned with current workflows.

Weaknesses:

- Does not help much when editing existing properties later.
- Does not validate files after manual edits, sync merges, imports, or scripts.
- Value lists can drift if they are duplicated across many templates.

Use this as a convenience layer, not the source of truth.

## Option 2: Metadata Menu

Metadata Menu is the strongest off-the-shelf editing UX for this problem.

Relevant capabilities:

- It manages YAML frontmatter fields and Dataview-style inline fields.
- It supports field definitions globally or through FileClasses.
- Field types include `Select`, `Multi`, and `Cycle`.
- Select and Multi values can come from a manually maintained list, a note path,
  or a JavaScript function.
- It provides autocompletion and modals for supported field types.
- FileClasses can be mapped by explicit `fileClass` property, tag, folder path,
  bookmark group, query, or global fallback.
- FileClasses can extend other FileClasses and override inherited fields.
- It can edit fields from Dataview tables and fileclass table views.

Why it fits `~/bob`:

- The local problem is mostly "show me the right controlled values while I edit
  notes."
- `status` needs folder/class scoping, and Metadata Menu FileClasses are built
  for that shape.
- A `restaurant` class can define `status: liked | not_liked`.
- A `ref` class can define `status: unread | wip | read | abandoned | ...`.
- Existing `type` or folder conventions can map notes to classes without
  rewriting the whole vault first.

Weaknesses:

- It is primarily a guided editing and metadata-management tool, not a hard
  security boundary.
- Raw YAML edits can still introduce invalid values.
- The plugin is mature and widely used, but the maintainer is explicitly asking
  for contributors, so we should avoid depending on exotic features unless we
  test them in the vault.

Use this for suggestions and normal editing.

## Option 3: Meta Bind

Meta Bind can render inline inputs, toggles, dropdowns, and buttons in notes,
then bind them to frontmatter properties.

Relevant capabilities:

- Inline or block input fields can be bound to frontmatter.
- `inlineSelect` and `select` controls can write controlled values.
- The control can display a label while writing a different raw value.

Strengths:

- Excellent for dashboard-like notes, project pages, RPG/database-style notes,
  and compact controls inside note bodies.
- Popular and actively updated.
- Works well when the desired UX is "click this control in the note."

Weaknesses:

- The allowed values live in each Meta Bind control unless we add another layer
  to generate them.
- It does not define a vault-wide schema.
- It does not stop edits through the native Properties UI or raw YAML.

Use this for special high-touch note templates. It is not the best primary
property schema system for `~/bob`.

## Option 4: Form Plugins

Form-oriented plugins can constrain data entry more strongly than the native
Properties UI.

Candidates:

- **Modal Forms**: defines reusable forms with fields such as select lists and
  can be called from Templater, QuickAdd, DataviewJS, or other JavaScript.
- **Data Entry**: turns metadata into forms using JSON Schema and JSON Forms.

Strengths:

- Good when note creation or editing should happen through a custom form.
- JSON Schema based tools naturally support enum-style constraints.

Weaknesses:

- Forms are an alternate workflow, not native Properties or Bases.
- Data Entry's plugin page still calls out work-in-progress behavior and an
  editing bug, and its last listed update is old.
- Forms add friction for everyday note editing unless the workflow is already
  capture-form centric.

Use forms only for specific capture workflows where a modal is a better UX than
editing frontmatter.

## Option 5: Validation Plugins

Validation is the enforcement half of the problem. It will not necessarily make
editing pleasant, but it catches drift.

### Propsec

Propsec validates frontmatter against schemas.

Relevant capabilities:

- Targets notes by folder path, tags, and property conditions.
- Supports required/warn flags, uniqueness, cross-field constraints, and
  conditional validation.
- Supports primitive field types and constraints such as string regex patterns,
  number ranges, array item counts, and date ranges.
- Validation is read-only; notes are not modified.

Fit:

- Good lightweight warning system.
- Enum values can be represented with a regex pattern such as
  `^(liked|not_liked)$`, but the public docs do not show a first-class enum
  field type.
- Better as a validation companion to Metadata Menu than as the source of
  suggestions.

### Forge

Forge is a broader vault-maintenance plugin.

Relevant capabilities:

- Schema validation with explicit allowed values, including examples using
  `type: enum` and `values`.
- Vault linting for missing metadata, malformed YAML, invalid metadata, stale
  fields, schema violations, and structural drift.
- Frontmatter/tag normalization, repair workflows, patch operations, exports,
  and relationship indexes.
- Designed to work alongside Dataview, Bases, Templater, QuickAdd, and Metadata
  Menu.

Fit:

- Best in-Obsidian option if "enforce" means lint, report, normalize, and repair
  property drift.
- The direct enum syntax matches this problem well.
- It is very new compared with Metadata Menu, so pilot it on a narrow subset
  before relying on its repair operations across the vault.

## Option 6: Custom Bob Validator

A small `bob` command can enforce the actual source of truth outside Obsidian.

Possible shape:

```yaml
rules:
  - name: restaurant status
    paths: ["eat/**/*.md"]
    property: status
    allowed: [liked, not_liked]
  - name: ref status
    paths: ["ref/**/*.md"]
    property: status
    allowed:
      - unread
      - wip
      - read
      - abandoned
      - collect_fleeting_notes
      - review_fleeting_notes
      - review_lit_notes
```

The command could:

- Parse Markdown frontmatter with a real YAML parser.
- Validate values by path/tag/type.
- Produce a readable report.
- Support `--fix` only for explicit mappings such as `in_progress -> wip`.
- Run from shell, cron, pre-commit, CI, or SASE.

Strengths:

- Rules live in this repo, so they can be reviewed and versioned.
- Does not depend on Obsidian plugin UI internals.
- Can be tested with normal Python tests.
- Can be integrated with existing Bob/SASE workflows.

Weaknesses:

- It does not provide in-editor suggestions.
- It is another tool to build and maintain.
- It only enforces when run.

Use this if we want rigorous validation without betting on a new Obsidian
maintenance plugin.

## Trade-off Matrix

| Option | Suggestions while editing | Scoped allowed values | Detects invalid existing values | Repairs values | Fits current vault |
| --- | --- | --- | --- | --- | --- |
| Native Properties/Bases | Partial, based on existing values | No | No | No | Useful baseline only |
| Templater/QuickAdd | Yes, during capture/template flow | Yes, per template | No | No | Already installed |
| Metadata Menu | Yes | Yes, via FileClasses | Partial/indirect | Some editing/bulk workflows | Best UX fit |
| Meta Bind | Yes, inside notes with controls | Yes, per control/template | No | No | Good for special pages |
| Modal Forms/Data Entry | Yes, through forms | Yes | Depends on plugin/schema | Depends on workflow | Use case specific |
| Propsec | No | Yes, via schemas and filters | Yes | No | Good lightweight validator |
| Forge | No direct property-dropdown UX | Yes, via schemas | Yes | Yes, with dry-run/repair workflows | Promising but new |
| Custom `bob` validator | No | Yes | Yes | Optional | Most controllable enforcement |

## Recommended Solution

Adopt a two-layer approach.

First, install and pilot **Metadata Menu** for the editing experience. Define
FileClasses for the obvious note classes:

- `restaurant` mapped to `eat/` or `type: restaurant`
- `ref` mapped to `ref/` or `type: "[[ref]]"`
- later: daily/monthly/yearly/project/inbox classes as needed

Define `Select` fields where the value set is small and closed:

- `restaurant.status`: `liked`, `not_liked`
- `ref.status`: `unread`, `wip`, `read`, `abandoned`,
  `collect_fleeting_notes`, `review_fleeting_notes`, `review_lit_notes`
- `type` only if we decide whether values should be plain strings or links

Second, add validation. If we want the validation to happen visibly inside
Obsidian, pilot **Forge** on `eat/` and `ref/` only, because it has explicit enum
schemas and can lint/repair metadata drift. If we want the rules to be
versioned and automation-friendly, implement a small `bob` validator instead
and run it from the shell/SASE. Either way, treat validation as the source of
truth and Metadata Menu as the editing UI.

Do not rely on native Obsidian property autocomplete alone. It is useful, but it
learns from existing values and is not scoped enough for this vault.

## Suggested First Experiment

1. Install Metadata Menu.
2. Create one FileClass for `restaurant`.
3. Map it to `eat/` or `type: restaurant`.
4. Define `status` as a `Select` with `liked` and `not_liked`.
5. Open `eat.base` and several restaurant notes; confirm that editing `status`
   offers only the restaurant values.
6. Add either a Forge schema or a simple script check for `eat/**/*.md`.
7. Repeat for `ref/` only after the restaurant pilot feels stable.

## Sources

- [Obsidian Help: Properties](https://obsidian.md/help/properties)
- [Obsidian Help: Properties view](https://obsidian.md/help/Plugins/Properties%2Bview)
- [Obsidian Help: Bases](https://obsidian.md/help/bases)
- [Obsidian Forum: Enumerated properties feature request](https://forum.obsidian.md/t/enumerated-properties-unique-select-from-a-prefixed-set-of-values/63900)
- [Obsidian Forum: Property suggestions are global/noisy](https://forum.obsidian.md/t/obsidian-base-plugin-limit-property-value-suggestion-to-filtered-folder/109469)
- [Obsidian Forum: Disable suggestions for certain properties](https://forum.obsidian.md/t/disable-suggestions-for-certain-properties/72851)
- [Metadata Menu plugin listing](https://community.obsidian.md/plugins/metadata-menu)
- [Metadata Menu docs: Fields](https://mdelobelle.github.io/metadatamenu/fields/)
- [Metadata Menu docs: FileClasses](https://mdelobelle.github.io/metadatamenu/fileclasses/)
- [Metadata Menu docs: Controls](https://mdelobelle.github.io/metadatamenu/controls/)
- [Templater docs: System module](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/system-module.html)
- [Meta Bind plugin listing](https://community.obsidian.md/plugins/obsidian-meta-bind-plugin)
- [Meta Bind docs: Input fields](https://www.moritzjung.dev/obsidian-meta-bind-plugin-docs/guides/inputfields/)
- [Meta Bind docs: Select input](https://www.moritzjung.dev/obsidian-meta-bind-plugin-docs/reference/inputfields/select/)
- [Modal Forms plugin listing](https://community.obsidian.md/plugins/modalforms)
- [Data Entry plugin listing](https://community.obsidian.md/plugins/data-entry)
- [Propsec plugin listing](https://community.obsidian.md/plugins/propsec)
- [Propsec GitHub README](https://github.com/ccmdi/propsec)
- [Forge plugin listing](https://community.obsidian.md/plugins/forge)
- [Obsidian Developer Docs: `processFrontMatter`](https://obsidian-developer-docs.pages.dev/Reference/TypeScript-API/FileManager/processFrontMatter)
- [Obsidian Developer Docs: `registerEditorSuggest`](https://obsidian-developer-docs.pages.dev/Reference/TypeScript-API/Plugin/registerEditorSuggest)
