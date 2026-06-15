# QuickAdd Migration for Hammerspoon Task Capture

Date: 2026-06-15

## Question

Should the existing Hammerspoon task-capture keymap move to an Obsidian
QuickAdd-based workflow? If so, what benefits are worth the migration cost, and
what implementation shape best preserves the current behavior?

## Current Local Workflow

The managed Hammerspoon source is:

`/home/bryan/.local/share/chezmoi/home/dot_hammerspoon/init.lua`

Relevant behavior observed in that file:

- `cmd+ctrl+shift+i` opens a Hammerspoon `hs.webview` prompt.
- Input is normalized to one line.
- Captured tasks are written as `- [ ] #task <text> [created::YYYY-MM-DD]`.
- A route token at either edge, `@route task` or `task @route`, writes to
  `~/bob/<route>.md`.
- Unrouted tasks append to `~/bob/mac_inbox.md`.
- Routed tasks are inserted after the last top-level open `#task` line in the
  target file, after that task's indented continuation block.
- Hammerspoon writes the markdown files directly with Lua file IO.

Local Obsidian/QuickAdd state:

- `~/bob` is the Bob Obsidian vault, and the machine has an
  `ob-sync-bob.service` with `WorkingDirectory=/home/bryan/bob`.
- QuickAdd is already installed and enabled in the Bob vault:
  `~/bob/.obsidian/plugins/quickadd`.
- Installed QuickAdd version is `2.12.3`.
- `~/bob/.obsidian/plugins/quickadd/data.json` currently has no choices:
  `"choices": []`.
- `~/.config/obsidian/obsidian.json` only showed the GUI registry entry for
  `/home/bryan/var/obsidian/vaults/greatday`, not `~/bob`. That does not prove
  Bob cannot be opened in GUI Obsidian, but it is a migration gate for
  `obsidian://quickadd?...` URIs: the Bob vault must be registered/addressable
  by Obsidian.
- Running the local `obsidian` CLI while Obsidian was not running returned:
  `The CLI is unable to find Obsidian. Please make sure Obsidian is running and try again.`

## What QuickAdd Provides

QuickAdd has four workflow building blocks: Template, Capture, Macro, and Multi
choices. The official getting-started docs describe Capture as the appropriate
choice when appending text to an existing journal, log, task list, or file.

QuickAdd Capture choices support:

- Static or dynamic target files.
- Creating missing markdown files.
- Task formatting.
- File write positions such as top, bottom, after a line, and before a line.
- Format syntax such as `{{DATE}}` and named `{{VALUE:...}}` fields.

QuickAdd can also be launched externally:

- URI: `obsidian://quickadd?choice=<choice>[&value-name=...]`
- CLI, on supported versions: `obsidian vault=<vault> quickadd choice="..."`

The URI docs make two details important for this migration:

- The `choice` name must match exactly.
- Variables passed through URI params must be named, for example
  `value-task=...`; bare `{{VALUE}}` cannot be filled by URI.

QuickAdd scripts run inside Obsidian and receive access to `app`,
`quickAddApi`, and shared `variables`. That is enough to implement custom
capture logic against the Obsidian vault API rather than relying only on the
built-in Capture placement options.

## Fit Against Requirements

| Current requirement | Pure QuickAdd Capture | QuickAdd Macro/User Script |
| --- | --- | --- |
| Global macOS hotkey | Needs Hammerspoon or system hotkey to launch it | Same |
| Single-line task prompt | Yes | Yes |
| Preserve `#task` and `[created::date]` format | Yes | Yes |
| Default append to `mac_inbox.md` | Yes | Yes |
| Parse `@route` at beginning or end | Awkward without multiple choices or prompts | Yes |
| Dynamic target `~/bob/<route>.md` | Possible with variables, but route parsing is awkward | Yes |
| Insert routed task after the last open task block | Not a natural fit | Yes |
| Works while GUI Obsidian is closed | Worse than current flow | Worse than current flow |
| Reusable from Obsidian command palette/mobile/Shortcuts | Yes | Yes |
| Testability | Low unless config is generated/tested | Medium if script logic is factored and tested |

The important mismatch is insertion semantics. QuickAdd's built-in "After line"
support inserts relative to a configured line and has useful heading/section
controls, but the current workflow needs to find the last top-level open `#task`
in an arbitrary routed file and then skip that task's indented continuation
block. That is custom markdown-list logic. A pure Capture choice would either
lose this behavior or require changing the file structure to add stable section
anchors.

## Benefits of Migrating

1. Less Hammerspoon-specific vault mutation

   The current keymap contains UI code, route parsing, task formatting, markdown
   block scanning, file creation, file reading, file writing, and notifications.
   Moving the Obsidian-specific part into QuickAdd would let Hammerspoon become
   only a launcher or fallback prompt.

2. One Obsidian-native workflow surface

   A QuickAdd choice can be run from the Obsidian command palette, Obsidian
   hotkeys, external URI launchers, and potentially mobile automation. The
   current Hammerspoon flow is macOS-only.

3. Better future routing and metadata hooks

   Once the workflow runs inside Obsidian, a script can later use Obsidian APIs,
   metadata cache, Dataview API, Tasks plugin conventions, or project-note
   metadata to suggest routes and enrich tasks. Hammerspoon has to rediscover or
   reimplement that context from files.

4. Cleaner task-capture extensibility

   QuickAdd already models prompts, variables, templates, macros, and reusable
   choices. Future fields such as due date, priority, project, context, or
   snooze date can be added without expanding the Hammerspoon webview.

5. Existing dependency, not a new one

   QuickAdd is already installed and enabled in the Bob vault. The migration
   would configure an existing plugin rather than add a new plugin dependency.

6. Better Obsidian UX if GUI capture is acceptable

   QuickAdd has built-in draft persistence and Obsidian notices. If the user is
   already in Obsidian or willing to focus Obsidian for capture, QuickAdd's
   prompt experience may be good enough to replace the custom Hammerspoon
   prompt.

## Costs and Risks

1. Capture may become slower and less app-independent

   The current Hammerspoon prompt works without focusing Obsidian and can write
   directly to `~/bob`. QuickAdd requires a running Obsidian plugin host. URI
   launch may open/focus Obsidian, and the local CLI currently cannot talk to
   Obsidian unless Obsidian is running.

2. Bob vault registration is a gate

   The local GUI Obsidian registry I found did not list `~/bob`. Before relying
   on `obsidian://quickadd`, the Bob vault needs to be registered with GUI
   Obsidian, or the workflow needs a reliable way to address it. Relying on "most
   recent vault" is unsafe because the registered recent vault appears to be
   `greatday`.

3. Pure Capture is not enough

   The current insertion behavior is more specific than QuickAdd's built-in
   Capture position settings. A real migration should use a Macro/User Script,
   not just a Capture choice, unless the file organization changes to use fixed
   section anchors.

4. Plugin version drift matters

   The installed QuickAdd is `2.12.3`; the latest release observed was `2.13.1`
   on 2026-06-12. QuickAdd `2.13.0` raised its Obsidian requirement to
   `1.13.0+`, and the release notes say older Obsidian installs stay on
   `2.12.3`. That means migration design should target the installed `2.12.3`
   feature set unless Obsidian itself is upgraded.

5. Sync timing still matters

   QuickAdd's URI docs warn that file-based sync can create duplicate or stale
   writes if Obsidian has not opened and synced the vault first. This is less
   risky for writing to existing stable files than for creating new route files,
   but it is still a reason to keep route-file creation conservative.

6. Configuration-as-code is less obvious

   The current behavior is in chezmoi-managed Lua. QuickAdd stores choices in
   vault plugin JSON and can include user scripts in the vault. Hand-editing
   plugin JSON is possible but brittle. A maintainable migration should keep the
   custom script in a normal file and treat QuickAdd settings as configuration,
   not as the main source of truth for complex logic.

## Options

### Option A: Keep the Hammerspoon implementation

This is best if the highest priority is zero-latency, global capture that works
even when Obsidian is closed. It keeps the workflow simple operationally, but it
continues to put markdown parsing and vault mutation in Hammerspoon.

### Option B: Replace it with a pure QuickAdd Capture choice

This is not recommended. It would be easy for the default inbox append case, but
it does not preserve the routed insertion rule without changing the task files
to have stable anchors or accepting bottom-of-file insertion.

### Option C: Hammerspoon launches a QuickAdd Macro/User Script

This is the best migration target. Hammerspoon keeps the macOS global hotkey,
but the task-capture behavior lives in a QuickAdd choice that can also be run
inside Obsidian.

Two variants are viable:

- Hammerspoon launches QuickAdd and lets QuickAdd prompt:
  `obsidian://quickadd?vault=<bob-vault>&choice=Bob%3A%20Capture%20Task`
- Hammerspoon keeps the custom prompt and passes text to QuickAdd:
  `obsidian://quickadd?vault=<bob-vault>&choice=Bob%3A%20Capture%20Task&value-task=<encoded>`

The first variant removes more Hammerspoon code. The second preserves the
current "capture without leaving the current app" feel better, but still moves
vault mutation into QuickAdd.

### Option D: Shared capture helper plus QuickAdd wrapper

This is the most testable architecture if the capture algorithm keeps growing:
extract route parsing and insertion into a shared, tested helper, then call it
from Hammerspoon and/or QuickAdd. The downside is more plumbing, and QuickAdd
desktop user scripts are not as clean a shell-command host as Hammerspoon.

## Prototype Shape

Create a QuickAdd Macro choice named `Bob: Capture Task` with command enabled.

The macro should run one user script:

1. Read `variables.task`; if missing, call
   `quickAddApi.inputPrompt("Capture task")`.
2. Normalize whitespace the same way Hammerspoon does today.
3. Parse route tokens at either edge:
   - `@route task`
   - `task @route`
4. Build `- [ ] #task <text> [created::YYYY-MM-DD]`.
5. If unrouted, append to `mac_inbox.md`.
6. If routed, target `<route>.md` and insert after the last top-level open task
   block, using the same continuation rules as the current Lua implementation.
7. Show a QuickAdd/Obsidian notice with the target and task text.

Before making Hammerspoon depend on it:

1. Register/open the Bob vault in GUI Obsidian and verify that
   `obsidian://quickadd?...` reaches the Bob vault, not `greatday`.
2. Configure the QuickAdd choice in the Bob vault.
3. Run manual test captures for:
   - unrouted inbox append
   - `@work task`
   - `task @work`
   - routed insertion after a task with nested bullets
   - missing route file behavior
4. Keep the old Hammerspoon write path for at least a short trial period as a
   fallback.
5. Only after the trial, shrink Hammerspoon to a URI launcher or remove the
   direct file writer.

## Source Notes

- QuickAdd Getting Started:
  https://quickadd.obsidian.guide/docs/
- QuickAdd Capture choices:
  https://quickadd.obsidian.guide/docs/Choices/CaptureChoice/
- QuickAdd Obsidian URI:
  https://quickadd.obsidian.guide/docs/Advanced/ObsidianUri/
- QuickAdd CLI:
  https://quickadd.obsidian.guide/docs/Advanced/CLI/
- QuickAdd scripting overview:
  https://quickadd.obsidian.guide/docs/Advanced/ScriptingGuide/
- QuickAdd API:
  https://quickadd.obsidian.guide/docs/QuickAddAPI/
- QuickAdd releases:
  https://github.com/chhoumann/quickadd/releases
- Obsidian URI docs:
  https://obsidian.md/help/uri

## Recommended Solution

Migrate incrementally to a hybrid QuickAdd workflow, not to a pure QuickAdd
Capture choice.

Keep Hammerspoon as the global macOS trigger, but create a `Bob: Capture Task`
QuickAdd Macro/User Script that owns the Obsidian task-capture logic. The script
should preserve the current task format, `@route` parsing, default
`mac_inbox.md` append behavior, and "insert after the last open task block"
semantics. Use `obsidian://quickadd` as the first launcher path once the Bob
vault is registered and verified in GUI Obsidian. Keep the current Hammerspoon
direct writer as a fallback until the QuickAdd path has been exercised against
real routed tasks.

This gives the real benefits of the migration: an Obsidian-native workflow,
future reuse from command palette/mobile/Shortcuts, less Hammerspoon-specific
vault mutation, and easier future metadata prompts. It avoids the main failure
mode: replacing a precise, working capture algorithm with a simpler QuickAdd
Capture configuration that cannot naturally preserve the current routed
insertion behavior.
