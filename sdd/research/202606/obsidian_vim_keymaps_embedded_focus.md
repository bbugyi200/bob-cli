---
create_time: 2026-06-15
status: research
topic: Vim normal-mode keymaps not firing when a non-editor element (e.g. a transcluded Bases table) is focused in Obsidian
---
# Research: Vim Keymaps Don't Fire When an Embed Is Focused in Obsidian

## Question

Bob's Obsidian vault runs in Vim mode with a set of custom normal-mode keymaps
(`-`, `[[`, `]]`, `!`, `[<Space>`, `]<Space>`, `<C-j>`, `<C-k>`, `\|`). These keymaps stop
working whenever a **non-editor element is the focused/selected thing in the workspace** —
the canonical example being a **transcluded Bases table**, but the same happens with
embedded queries, interactive Dataview widgets, selected images, PDFs, canvas, and property
fields. Is there a way to work around this so the keymaps (or equivalent actions) remain
reachable? What are the options, and which should we adopt?

## Current Vault State (observed)

From `/home/bryan/bob/.obsidian/` and the vault root:

- `app.json` → `"vimMode": true`, `"showLineNumber": true`.
- `community-plugins.json` enables (relevant ones): `obsidian-vimrc-support`,
  `bob-navigation-hotkeys`, `bob-ledger-tools`, `dataview`, `metadata-menu`,
  `mrj-jump-to-link`.
- `obsidian-vimrc-support` reads `obsidian_vimrc.md` (vault root). **`supportJsCommands` is
  `false`** — JavaScript vimrc commands are intentionally disabled.
- The vimrc keymaps are **thin bridges**: each `nmap` calls an `exmap` that runs an
  `obcommand` (an Obsidian command). For example:

  ```
  exmap bob_prev_link obcommand bob-navigation-hotkeys:open-prev-link
  nmap [[ :bob_prev_link<CR>
  ...
  nmap - :bob_daily<CR>          " bob-ledger-tools:open-today-daily-note
  nmap ]] :bob_next_link<CR>
  nmap ! :bob_toggle_transclusions<CR>
  nmap <C-j> :bob_next_header<CR>
  nmap <C-k> :bob_prev_header<CR>
  nmap \| :bob_dash_tasks<CR>
  ```

- **Crucially, the real actions are already first-class Obsidian commands.** The
  `bob-navigation-hotkeys` plugin registers them all via `this.addCommand({...})` (and one,
  "Yank...", already attaches a native hotkey `Mod+Y`). The vim keymaps are just *one* way
  to reach those commands; they are not the only possible trigger.
- `task-status-cycler` already has both direct `window.CodeMirrorAdapter.Vim` mappings and
  a narrow capture-phase DOM `keydown` fallback for one physical hotkey. So this vault
  already has precedent for bridging outside pure CodeMirror Vim dispatch when needed.

This is the key lever for every workaround below: we don't need to re-implement any behavior
— we only need a **focus-independent way to invoke commands that already exist**.

## Key Findings

### 1. Root cause: Obsidian Vim mode is editor-scoped, not app-scoped

Obsidian's Vim mode is the `codemirror-vim` addon running **inside the CodeMirror 6 markdown
editor**. Vim normal-mode keymaps only receive keystrokes when the **CodeMirror editor's DOM
node has keyboard focus** and Vim is in normal/visual mode. The moment you click or tab onto
a different element, browser focus moves out of CodeMirror, keystrokes are delivered to that
element (or to `document.body`), and `codemirror-vim` never sees them — so your `nmap`s
silently do nothing. This is inherent to how the addon is wired; the `obsidian-vimrc-support`
README confirms keymaps are tested "in Obsidian's normal mode (type `:` in the editor)" —
i.e., they presuppose editor focus.

By contrast, **native Obsidian hotkeys** are dispatched by Obsidian's global keymap `Scope`
and fire at the **application level regardless of editor focus** (with one caveat — see #4).
That is the architectural difference we can exploit.

### 2. A transcluded Bases table is exactly the kind of element that steals focus

A `.base` embedded in a note renders as an **interactive HTML widget**, not as editor text.
Clicking it (or tabbing into it) makes it — or one of its cells — the focused element, which
pulls focus out of CodeMirror. The same applies to other interactive embeds (embedded
queries, Dataview interactive tables, canvas, PDF viewers, property fields, and selected
images).

Obsidian's own changelog corroborates that embeds capture keys and that the team is patching
these cases **one at a time** rather than via a general fix:

- **Obsidian 1.13.0 (2026-05-28):** *"Live Preview: Fixed `Ctrl/Cmd-A` not working inside
  embedded inputs (e.g. a cell in an embedded Bases table)."*
- Same release: when an image is selected, *"backspace and delete will delete the image …
  This should also support Vim out of the box."* — i.e., Vim integration over non-editor
  elements is being hand-wired element by element.

Takeaway: **upstream will not restore arbitrary user `nmap`s over embeds** any time soon. The
fixes are scoped to specific built-in keys (select-all, delete image), not to custom Vim
keymaps. We need our own workaround.

### 3. Bases now has a better native keyboard model, but not Vim keymaps

Obsidian 1.10.3 public (2025-11-11) added substantial Bases table keyboard support:
selection, full keyboard navigation, copy/paste, undo/redo, and table-specific hotkeys such
as Enter, Tab, Shift-Tab, Home, End, PageUp/PageDown, row/column selection, and clearing
cells.

That helps with keyboard use inside Bases, but it does not mean custom Vim normal-mode
mappings from `obsidian_vimrc.md` will fire while a Base owns focus. It gives Bases its own
keyboard model, not CodeMirror Vim's model.

### 4. The real tension: bare keys vs. modifier chords

Vim keymaps can safely use **bare, unmodified keys** (`-`, `]]`, `!`, `\|`) because they only
fire in *normal* mode — they never collide with typing. Native Obsidian hotkeys fire in
*every* mode, including while typing, so a bare key is unusable as a global hotkey (binding
`-` globally would break typing a dash). **Any app-level workaround for the bare-key maps
must either add a modifier, add a leader prefix, or gate itself on "focus is not in an
editable field."** This constraint shapes the options below.

Note also the caveat to #1: when an embedded input is *actively being edited* (e.g. you've
clicked into a Bases cell and are typing), even native hotkeys can be swallowed by that input
(that's the very bug 1.13.0 fixed for `Ctrl/Cmd-A`). But when a Bases table is merely
**selected/focused as a widget** (not mid-edit), its focused node is a `div`, and app-level
hotkeys generally still fire. The everyday "I clicked the table and now my keymaps are dead"
case is the recoverable one.

## Alternative Approaches

### A. Manual refocus (zero-config)

Press **`Esc`** or **click back into the note body** to return focus to CodeMirror; the Vim
keymaps then work again. Optionally bind a single global hotkey to a "focus the editor"
action for a keyboard-only path back.
*Pros:* nothing to build or install. *Cons:* it is exactly the friction you're trying to
avoid — a manual step every time, and it interrupts flow.

### B. Use native Bases keyboard navigation for table-internal work

If Obsidian is older than 1.10.3 public, update first. Bases keyboard support changed
materially there. For actual Bases table work, prefer the native table hotkeys because they
are maintained by Obsidian and know about selected cells, property editors, formulas,
row/column selection, and table copy/paste.

*Pros:* native and maintained by Obsidian; no vault code. *Cons:* does not restore custom Vim
normal-mode mappings; it only reduces the need for them while focus is inside Bases.

### C. Add native Obsidian hotkeys (modifier chords) for the same commands

Because the actions are already Obsidian commands, open Settings → Hotkeys and bind modifier
chords to them (e.g. `Alt+J`/`Alt+K` for next/prev header — note `<C-j>`/`<C-k>` are already
modifier-friendly; `Alt+-`, `Alt+[`, `Alt+]` for the bare-key ones). These dispatch through
the global `Scope` and fire over a selected embed.
*Pros:* native, no new code, focus-independent, survives Obsidian upgrades. *Cons:* a second
set of chords to remember alongside the bare-key Vim maps; must avoid conflicts; still
swallowed while *editing* a cell (acceptable — you wouldn't want them firing there anyway).

### D. Leader-key plugin (Spacekeys or Leader Hotkeys)

Install a leader-key plugin, bind **one** modifier leader (e.g. `Ctrl+M`), then reach any
command — including all `bob-*` commands — via a mnemonic sequence (`<leader> h j` →
next header, etc.). Works app-wide.
*Pros:* one global hotkey unlocks every nav command regardless of focus; mnemonic, which-key
style menu; covers commands you never bothered to bind. *Cons:* leader + sequence is more
keystrokes than a single bare key; adds a third-party plugin/dependency; same
edit-in-progress caveat as native hotkeys.

### E. Custom capture-phase key router in `bob-navigation-hotkeys` (most powerful)

We already own and ship `bob-navigation-hotkeys` (it imports `@codemirror/view` and registers
all these commands). Add a `this.registerDomEvent(window, "keydown", handler, {capture:true})`
(or an Obsidian `Scope` pushed high in the keymap stack) that, **only when
`document.activeElement` is not a CodeMirror editor and not an editable field**
(`INPUT`/`TEXTAREA`/`contenteditable`), maps the *same bare keys* (`-`, `[[`, `]]`, `!`, …) to
the corresponding commands — optionally calling `app.commands.executeCommandById(...)` and/or
refocusing the editor first.
*Pros:* recreates the exact bare-key muscle memory over transcluded Bases tables and other
embeds; in-editor Vim behavior is untouched; precise gating (stands down inside any editable
field, so it never hijacks typing in a Base cell); no new dependency; no reliance on upstream
Obsidian changes. *Cons:* it's custom code we maintain against Obsidian/CM API drift; the
focus/"is this safe?" heuristic must be written carefully and tested against the embed types
we actually use.

### F. Sidestep the focus trap (palliative)

View notes containing Bases in **Reading view**, interact with embeds via mouse only, or
replace interactive transclusions with a non-interactive representation.
*Pros:* trivial. *Cons:* gives up the interactivity that made the embed worth transcluding;
not a real fix.

### G. Wait for upstream

Rely on Obsidian continuing to patch embed key handling (as in 1.13.0).
*Pros:* zero effort. *Cons:* fixes are per-key and per-element, won't cover our custom
`nmap`s, and timing is out of our control. Not dependable.

## Trade-offs (summary)

| Approach | Effort | Keeps bare-key feel | Focus-independent | Dependency |
| --- | --- | --- | --- | --- |
| A. Manual refocus | none | n/a (manual) | — | none |
| B. Native Bases keyboard navigation | low | no | yes inside Bases | none |
| C. Native modifier hotkeys | low | no (modifiers) | yes (when not mid-edit) | none |
| D. Leader-key plugin | low | no (leader+seq) | yes (when not mid-edit) | + plugin |
| E. Custom key router in bob plugin | med | **yes** | yes (when not mid-edit) | owned code |
| F. Sidestep | none | n/a | — | none |
| G. Wait upstream | none | n/a | partial | none |

## Recommended Solution

**A tiered plan that separates Bases table navigation from Bob command dispatch.**

1. **Use Obsidian's native Bases keyboard navigation for table-internal work (Approach B).**
   Confirm the desktop app is at least 1.10.3 public, and preferably test current
   public/insider behavior because 1.13.0 includes more fixes around embedded inputs and
   selected elements. This is the right layer for moving through cells, selecting rows, and
   editing Base data.

2. **Immediately, no code — restore focus-independent access to Bob commands (Approach C,
   optionally D).**
   In Settings → Hotkeys, bind modifier chords to the `bob-navigation-hotkeys` /
   `bob-ledger-tools` commands you most want to reach over an embed (start with next/prev
   header, prev/next link, and open-daily). This gives you a working keyboard path over a
   selected Bases table *today*, with zero new code or dependencies. If you'd rather memorize
   one leader than several chords, install **Spacekeys** and bind a single `Ctrl+M` leader to
   reach the whole `bob-*` command set mnemonically.

3. **Best long-term — add a guarded bare-key router to `bob-navigation-hotkeys`
   (Approach E).** Since we already build and ship this plugin, add a capture-phase `keydown`
   handler that fires the existing commands from the *same bare keys* **only when focus is
   outside a CodeMirror editor and outside any editable field**, optionally refocusing the
   editor first. This is the only option that preserves your exact muscle memory (`-`, `]]`,
   `!`, …) when a transcluded Bases table is selected, leaves in-editor Vim untouched, never
   hijacks typing inside a Base cell, and doesn't depend on Obsidian's piecemeal upstream
   fixes. The gating heuristic (`activeElement` not `.cm-editor`, not
   `INPUT`/`TEXTAREA`/`contenteditable`) is small and testable, and matches the validation
   approach we already use for this plugin (`node -c main.js` plus stubbed-module checks).

Net: rely on native Bases keys for table work; bind native/leader hotkeys now for an instant
Bob-command path; graduate to a small owned key router in `bob-navigation-hotkeys` to make
the bare-key keymaps "just work" over embeds without waiting on Obsidian.

## Sources

- [Obsidian 1.13.0 Desktop changelog (2026-05-28) — embedded-input & Vim fixes](https://obsidian.md/changelog/2026-05-28-desktop-v1.13.0/)
- [Obsidian 1.10.3 Desktop changelog (2025-11-11) — Bases keyboard navigation](https://obsidian.md/changelog/2025-11-11-desktop-v1.10.3/)
- [Obsidian Help: Introduction to Bases](https://obsidian.md/help/bases)
- [esm7/obsidian-vimrc-support (GitHub)](https://github.com/esm7/obsidian-vimrc-support)
- [obsidian-vimrc-support README](https://github.com/esm7/obsidian-vimrc-support/blob/master/README.md)
- [CodeMirror Vim bindings documentation](https://codemirror.net/5/demo/vim.html)
- [Vimrc Support on Obsidian Stats](https://www.obsidianstats.com/plugins/obsidian-vimrc-support)
- [Obsidian Hub — for Vim users](https://publish.obsidian.md/hub/04+-+Guides,+Workflows,+&+Courses/for+Vim+users)
- [jlumpe/obsidian-spacekeys (leader-key / which-key plugin)](https://github.com/jlumpe/obsidian-spacekeys)
- [Spacekeys on Obsidian Stats](https://www.obsidianstats.com/plugins/spacekeys)
- [tgrosinger/leader-hotkeys-obsidian](https://github.com/tgrosinger/leader-hotkeys-obsidian)
- [Forum: Full keyboard support for Bases navigation & editing](https://forum.obsidian.md/t/bases-full-keyboard-support-for-bases-navigation-editing/103067)
- [Forum: Vim support for more commands in Live Preview tables](https://forum.obsidian.md/t/vim-support-more-commands-in-live-preview-tables/73992)
- [Forum: Vim-mode gj movement breaks with live preview, bullets, and transclusion](https://forum.obsidian.md/t/vim-mode-gj-movement-command-break-with-live-preview-bullets-and-transclusion/31417)
- [Forum: Keyboard shortcut to focus properties from note](https://forum.obsidian.md/t/keyboard-shortcut-to-focus-properties-from-note/72005)
- [obsidian-keyboard-analyzer (hotkey scope inspection)](https://github.com/x3c3/obsidian-keyboard-analyzer)
