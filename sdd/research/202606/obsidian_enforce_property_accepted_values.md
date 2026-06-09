---
create_time: 2026-06-09
status: research
topic: Enforcing & suggesting a fixed set of accepted values for Obsidian note properties
---
# Research: Enforcing & Suggesting Accepted Values for Obsidian Properties

## Question

Several note properties in Bob's Obsidian vault are effectively **enumerations** — they
should only ever hold one of a small, predefined set of values. Bryan wants to (a) **enforce**
those accepted values and (b) have Obsidian **suggest** them at the point of entry. What are
the options, and what should we adopt?

## Current Vault State (observed)

The vault already constrains itself informally. Scanning frontmatter across `/home/bryan/bob`
surfaces three clear "enum-like" properties:

| Property | Observed values (count) |
| --- | --- |
| `status` | `legacy` (282), `liked` (84), `wip` (10), `read` (7), `not_liked` (7), `abandoned` (4), `active` (3) |
| `type` | `[[ref]]` (304), `restaurant` (91), `daily` (10), `[[done]]` (7), `[[day]]` (3), `project` (2), `monthly` (2), `inbox` (2), `yearly` (1) |
| `marker_category` | `project` (446), `topic` (238), `person` (187), `context` (92), `status` (10) |

Two things stand out:

- The value sets are small and stable — good enum candidates.
- They are **already drifting**. `type` mixes wikilinks (`[[ref]]`) with bare strings
  (`restaurant`); `status: "legacy"` is quoted while `status: active` is not. This is exactly
  the inconsistency a constrained-value mechanism should prevent (and exactly what makes a
  *retroactive* audit valuable).

Relevant installed plugins (`/home/bryan/bob/.obsidian/community-plugins.json`):
`dataview`, `obsidian-tasks-plugin`, `templater-obsidian`, `quickadd`, plus several custom
`bob-*` plugins. **Not** installed: Metadata Menu, Meta Bind, Linter.

The vault also already declares property *types* in `.obsidian/types.json`
(`aliases`, `cssclasses` → multitext, `tags`, a batch of `TQ_*` checkboxes, `created` → text).
So assigning native property types is already part of the workflow — but none of the native
types is an "enum/select."

## The problem has two halves (and "enforce" is the hard one)

It helps to separate the two asks, because the available tooling splits along them:

1. **Suggest** — show the accepted values in the UI so the user *picks* instead of *types*.
   This is well solved.
2. **Enforce** — guarantee the stored value stays inside the accepted set. This is only
   *partially* solvable in Obsidian, because a `.md` file's YAML is always hand-editable (and
   values also arrive via sync, imports, and — in this vault — the `zorg` generators). No
   plugin can truly *prevent* an out-of-set value from existing on disk.

So real-world "enforcement" is actually two complementary moves:

- **Constrain the input path** so the normal way of setting the value can only emit a valid
  one (a curated dropdown/picker).
- **Audit for drift** so anything that bypasses the picker (raw edits, generators, paste) gets
  *flagged*. Without this backstop, "enforcement" is an illusion.

The strongest solutions do both. Pure-input tools (Templater, Meta Bind) cover only the first.

## Key Findings

### 1. Native Obsidian Properties — suggests, but does not curate or enforce

Obsidian's built-in property types are **Text, List, Number, Checkbox, Date, Date & time**.
There is **no native "select/enum" type** with a predefined option list. Native Text/List
properties *do* autocomplete — but they suggest **every value ever used for that key anywhere
in the vault**, typos included. For `type` that means the dropdown would already offer the
inconsistent `[[ref]]`/`restaurant`/`[[done]]` soup shown above, with no way to mark some
"wrong."

- Pros: zero install; already active; the autocomplete is genuinely helpful for *consistent*
  keys.
- Cons: no curated list, no enforcement, propagates existing typos, can't express "these five
  and only these five."
- Status: enumerated/select properties are a long-standing, still-**unimplemented** feature
  request (forum threads #63900, #75414, #105142 below).

Baseline verdict: necessary hygiene, insufficient on its own.

### 2. Metadata Menu — the purpose-built tool (best "suggest", soft "enforce")

`mdelobelle/metadatamenu` is the dedicated metadata-management plugin and the canonical answer
to this exact need.

- **Field types that restrict to a set:** `Select` (single value from a list), `Multi`
  (several values from a list), and `Cycle` (steps through the list in order). `Number`
  additionally supports `min`/`max`/`step` validation.
- **Three ways to define the accepted values:**
  1. a values list typed into the settings form (add/edit/remove one by one);
  2. a **note path** — "each line of the target note will be an option for the dropdown
     selector"; or
  3. a **JavaScript function** returning a `string[]`, with access to the Dataview API (`dv`)
     and the current page (`current`).
- **fileClass = per-note-type schema.** A fileClass note's frontmatter defines which fields a
  note type has and each field's type/options. fileClass settings override global settings, so
  you can have one schema for `restaurant` notes, another for `day` notes, etc.
- **Insert missing fields:** "When fileClasses are defined for a note, you can bulk insert all
  fields defined in those fileClasses that aren't yet included in the note" — individually or
  in bulk, ordered, at a chosen line or the end of the frontmatter.
- **Input UX:** modal selector, command-palette setter, and **inline suggestion with
  filtering** — the richest "pick from a curated list" experience, and it lands right in the
  note's properties.

What it does **not** do: the documentation describes no vault-wide indexing that *flags notes
whose value is outside the allowed set*. Field controls validate at the point of input, but
Metadata Menu won't hard-block a raw YAML edit and ships no "compliance report." So it nails
**suggest** and **soft-enforce-on-input**, but still wants a Dataview audit (Finding #6) as the
drift backstop.

- Pros: curated lists in the native Properties panel; per-type schemas; option list can be a
  vault note or a JS/Dataview function (→ single source of truth); "insert missing fields."
- Cons: the heaviest plugin here; introduces its own field-config layer to learn/maintain; no
  built-in out-of-set auditing; doesn't prevent raw edits.

### 3. Meta Bind — inline dropdowns bound to a property (good "suggest", no schema/audit)

`mProjectsCode/obsidian-meta-bind-plugin` renders interactive inputs inside a note that are
**bound to a frontmatter property** and kept in sync. A select looks like:

```
INPUT[select(option(active), option(wip), option(legacy)):status]
```

`option(value, name)` lets the stored `value` differ from the displayed `name`.

- Pros: a real dropdown of *curated* options; can be placed in templates or note bodies;
  conceptually light.
- Cons: options are defined **per embed** (use "input field templates" to avoid repeating the
  list); the control lives in the note body / a bound block rather than the native Properties
  panel; it's not a vault-wide schema and offers no auditing; known quirks — it **strips double
  quotes**, only single quotes allow commas in a value, and it won't create the bound property
  until first interacted with.

Good as an *editing* affordance, weaker as a system of record than Metadata Menu.

### 4. Templater suggester — creation-time enforcement (already installed)

Templater (installed) exposes `tp.system.suggester(text_items, items)`, a fuzzy picker over a
**fixed list** you hard-code in a template:

```js
status: <% await tp.system.suggester(["Active","WIP","Legacy"], ["active","wip","legacy"]) %>
```

The two arrays let you show friendly labels while storing canonical values.

- Pros: already installed; *forces* a valid value at note creation; pairs naturally with
  per-type creation templates so new notes are born compliant.
- Cons: only fires at creation (or a manual re-run); does nothing for the thousands of existing
  notes; offers no suggestion in the Properties UI afterward; the list lives in template code.

### 5. QuickAdd — same idea, capture/macro flavored (already installed)

QuickAdd (installed) can drive the same fixed-list pick through its capture/macro/choice flows
(and can call Templater). Same scope and trade-offs as Finding #4: excellent at *creation
time*, silent thereafter.

### 6. Dataview audit — the enforcement backstop (already installed)

Dataview (installed) can't *prevent* a bad value, but it can **find** every note that holds
one, which is the only mechanism here that catches drift regardless of how it arrived. An audit
note can list violators:

```dataview
TABLE status
FROM "" WHERE status AND !contains(list("active","wip","legacy","liked","not_liked","read","abandoned"), status)
```

- Pros: already installed; catches raw edits, sync, paste, and `zorg`-generated drift alike;
  the same allowed-values note used by Metadata Menu (Finding #2, option 2) can feed both, so
  the list lives in exactly one place.
- Cons: detect-not-prevent; **case-sensitive** (`Active` ≠ `active`); a note *missing* the
  field reads as `null`, so "required but absent" needs its own clause; mixed formats like
  `type: [[ref]]` vs `type: restaurant` must be normalized for the membership test to work.

### 7. Obsidian Linter — not the right tool

`platers/obsidian-linter` (not installed) formats/normalizes YAML (arrays, tags, inserting
*static* attributes) but has **no rule to constrain a field to an allowed value set**. It can
tidy formatting, not validate enums. Mentioned only to rule it out.

## Trade-offs at a glance

| Option | Curated suggest in UI | Enforces on input | Audits drift | Covers existing notes | Already installed |
| --- | --- | --- | --- | --- | --- |
| Native properties | Partial (uncurated) | No | No | n/a | ✅ |
| **Metadata Menu** | **Yes (best)** | Soft (input UI) | No | Yes (insert/edit) | ❌ |
| Meta Bind | Yes (inline) | Soft (input UI) | No | Only where embedded | ❌ |
| Templater suggester | At creation only | Yes (creation) | No | No | ✅ |
| QuickAdd | At creation only | Yes (creation) | No | No | ✅ |
| Dataview audit | No | No | **Yes** | Yes (reports) | ✅ |
| Linter | No | No | No | No | ❌ |

The columns make the core insight concrete: **no single tool fills every column.** Input tools
don't audit; the audit tool doesn't suggest. A good solution layers an input tool with the
audit.

## Recommended Solution

Adopt a **layered** approach — one source of truth for the allowed values, a curated picker on
the way in, and a Dataview audit as the enforcement backstop.

1. **Single source of truth: one note per enum.** Create e.g. `_meta/allowed/status.md`,
   `_meta/allowed/type.md`, `_meta/allowed/marker_category.md`, each listing the accepted
   values one per line. Everything downstream reads from these, so the list is edited in one
   place and is itself queryable.

2. **Install Metadata Menu and define `Select`/`Multi` fields sourced from those notes.**
   Point each field's options at its allowed-values note (option-source #2, "note path").
   Group the fields into **fileClasses** per note type (`ref`, `restaurant`, `day`, `project`,
   …) so each kind of note gets exactly the enum fields it should. This delivers the curated
   dropdown directly in the native Properties panel and the "insert missing fields" command for
   bringing existing notes up to schema. This is the primary **suggest + soft-enforce** layer
   and, unlike the creation-time tools, it also works while *editing* the existing ~thousands
   of notes.

3. **Add a Dataview "property compliance" dashboard note** that lists any note whose `status` /
   `type` / `marker_category` is set but **not** in the matching allowed-values note (and,
   per-fileClass, any required-but-missing field). This is the actual *enforcement*: it catches
   whatever bypasses the picker — raw YAML edits, sync, and the `zorg` generators that produce
   much of this vault. Because the query reads the same allowed-values notes from step 1,
   there's a single list to maintain. Run it periodically (or embed it in a daily/weekly note)
   and fix what it surfaces. **Before first use, normalize the existing drift** flagged in
   "Current Vault State" (the `[[ref]]`-vs-`restaurant` and quoting inconsistencies) so the
   membership test is meaningful.

4. **Optional, low-effort win now: a Templater (or QuickAdd) suggester in your note-creation
   templates.** Both are already installed. Adding a `tp.system.suggester([...], [...])` for
   `status`/`type` to the per-type templates means *new* notes are born compliant without even
   opening the Metadata Menu UI. Keep the suggester arrays generated from the same allowed-
   values notes if you want to avoid a second copy of the list.

**If you'd rather not add Metadata Menu** (it's the heaviest piece), a fully no-new-system-of-
record stack using only already-installed plugins is viable: **Templater suggester at creation
+ Dataview audit for drift**, optionally adding **Meta Bind** purely for inline editing
dropdowns. The cost is losing the curated picker inside the native Properties panel and the
per-type schema/"insert missing fields" convenience — which is precisely what makes Metadata
Menu worth the weight if these enums matter long-term.

Net recommendation: **Metadata Menu (Select/Multi via note-sourced options, organized by
fileClass) for suggest-and-soft-enforce, backed by a Dataview compliance dashboard for true
drift enforcement, with an optional Templater suggester for creation-time guarantees** — all
fed by one allowed-values note per property.

## Sources

- [Metadata Menu — Fields (Select/Multi/Cycle, option sources)](https://mdelobelle.github.io/metadatamenu/fields/)
- [Metadata Menu — FileClasses](https://mdelobelle.github.io/metadatamenu/fileclasses/)
- [Metadata Menu — Controls (insert missing fields)](https://mdelobelle.github.io/metadatamenu/controls/)
- [Metadata Menu — docs home](https://mdelobelle.github.io/metadatamenu/)
- [Metadata Menu (GitHub)](https://github.com/mdelobelle/metadatamenu)
- [Meta Bind — Input Fields](https://www.moritzjung.dev/obsidian-meta-bind-plugin-docs/guides/inputfields/)
- [Meta Bind — Input Field Templates](https://www.moritzjung.dev/obsidian-meta-bind-plugin-docs/guides/templates/)
- [Meta Bind (GitHub)](https://github.com/mProjectsCode/obsidian-meta-bind-plugin)
- [Templater — tp.system.suggester](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/system-module.html)
- [Templater Prompts & Suggestion Menus (Obsidian Observer)](https://medium.com/obsidian-observer/prompts-suggestion-menus-with-templater-22f8e62d28b3)
- [Obsidian Help — Properties (native types)](https://help.obsidian.md/properties)
- [A Complete Guide to Obsidian Properties (autocomplete behavior)](https://practicalpkm.com/complete-guide-to-obsidian-properties/)
- [Forum FR: Enumerated properties — select from a fixed set (#63900)](https://forum.obsidian.md/t/enumerated-properties-unique-select-from-a-prefixed-set-of-values/63900)
- [Forum FR: Add predefined values to multi-select properties (#75414)](https://forum.obsidian.md/t/add-predefined-values-to-multi-select-properties/75414)
- [Forum FR: Customizable Property Suggestions Lists (#105142)](https://forum.obsidian.md/t/customizable-property-suggestions-lists/105142)
- [Obsidian Linter — YAML rules (no enum-validation rule)](https://platers.github.io/obsidian-linter/settings/yaml-rules/)
- [Dataview — query/annotation docs](https://blacksmithgu.github.io/obsidian-dataview/)
