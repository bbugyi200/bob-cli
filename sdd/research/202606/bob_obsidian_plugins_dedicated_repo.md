---
create_time: 2026-06-20
status: research
topic: Best way to host Bob's custom Obsidian plugins in a dedicated GitHub repo
---
# Research: A Dedicated GitHub Repo for Bob's Custom Obsidian Plugins

## Question

Bryan wants to move the custom "Bob" Obsidian plugins currently living under
`~/bob/.obsidian/plugins/` into a dedicated GitHub repository. What does the
best solution look like — repo topology (one repo vs. monorepo), how the vault
keeps getting the plugin files, how releases/distribution work, and how to
migrate without creating two sources of truth?

## Short Answer

Use a **single monorepo** (e.g. `bbugyi200/bob-obsidian-plugins`) as the source
of truth for all six plugins — one top-level folder per plugin, each folder
being exactly what Obsidian loads. Keep them as **plain JavaScript with no build
step** (the current setup), so the committed `main.js` is both source and
deployed artifact. Deliver the files back into the vault with a small
**`bob` sync command** (or symlinks for a single desktop), and **stop tracking
them in the vault repo** so the new repo is the only source of truth.

Distribution caveat that drives the topology: both **BRAT** and the **official
community store require the GitHub release tag to equal the `manifest.json`
version exactly** (no prefix, no subfolder). A monorepo therefore cannot feed
the store/BRAT for more than one plugin at a time. So: keep everything in the
monorepo for personal use now, and **split an individual plugin out into its own
dedicated repo only if/when you decide to publish that specific plugin**. This
is the same pattern the community monorepos (`polyipseity/obsidian-monorepo`,
`auxvirtua/obsidian`) follow — none of their plugins ship through the official
store from the monorepo itself.

## Current State (verified locally on 2026-06-20)

Six hand-authored custom plugins live under `~/bob/.obsidian/plugins/`:

| Folder | Manifest `name` | ver | Files | LOC (`main.js`) |
| --- | --- | --- | --- | --- |
| `block-id-prompt` | Block ID Prompt | 1.0.0 | `main.js`, `manifest.json` | 2,220 |
| `bob-ledger-tools` | Bob Ledger Tools | 1.0.0 | `main.js`, `manifest.json` | 1,891 |
| `bob-navigation-hotkeys` | Bob Navigation Hotkeys | 1.0.0 | `main.js`, `manifest.json`, `styles.css` | 7,420 |
| `bob-project-tasks` | Bob Project Tasks | 1.0.0 | `main.js`, `manifest.json` | 276 |
| `bob-vim-surround` | Bob Vim Surround | 1.2.0 | `main.js`, `manifest.json` | 1,146 |
| `task-status-cycler` | Task Status Cycler | 1.0.0 | `main.js`, `manifest.json` | 4,283 |

Total ≈ 17,200 lines across the six. Key facts that shape the recommendation:

- **No build step.** Each `main.js` is hand-written CommonJS
  (`require("obsidian")`, `require("@codemirror/view")`). There is **no
  `package.json`, `tsconfig.json`, esbuild config, sourcemap, or TypeScript
  source** anywhere — the `main.js` is the source of truth, not a compiled
  artifact. This is unusual (the official template uses TS + esbuild) but it
  makes extraction trivial: there is no source/artifact gap to bridge.
- **Already version-controlled, but bundled with personal notes.** The vault is
  a git repo with remote `git@github.com:bbugyi200/bob.git`. Its `.gitignore`
  explicitly tracks `.obsidian/**/*.js`, `*.json`, `*.css`, so all six plugins
  are already committed there. The repo also contains 2,000+ personal markdown
  notes — i.e. these plugins are effectively in a **private** repo today.
- **Git is the sync mechanism, not Obsidian Sync.** There is no
  `.obsidian/sync.json`. Plugins reach other machines via the vault's git
  history. (This relaxes the usual "symlinks break Obsidian Sync / mobile"
  concern, but means any extraction must still get real files into the vault on
  each cloned machine.)
- **All six are enabled** in `.obsidian/community-plugins.json` and all are
  `isDesktopOnly: false`.
- **Loosely coupled, no shared imports.** Plugins are independent; where two
  need the same convention they *deliberately duplicate* it. For example
  `bob-navigation-hotkeys/main.js` comments that it mirrors a minimal subset of
  `bob-ledger-tools` conventions "Duplicated here on purpose so this plugin does
  not reach into another plugin's non-public module internals." So there is no
  shared library forcing them into one repo — but also nothing preventing a
  monorepo from later factoring out shared helpers.
- **A Rust `bob` CLI already exists and is deeply Obsidian-integrated**
  (`bob dataview`, capture, projects, etc.). It is the natural home for a
  `bob obsidian-plugins sync` command.

## Constraints That Drive The Design

1. **Obsidian loads plugins only from `<vault>/.obsidian/plugins/<id>/`.**
   Whatever repo holds the source, the real files (`manifest.json`, `main.js`,
   optional `styles.css`) must end up at that path. A dedicated repo cannot be
   loaded "in place" from outside the vault.
2. **Release tag must equal the manifest version, exactly.** For BRAT and for
   the official store, the release **tag**, **release name**, and the
   **`version` inside the released `manifest.json`** must all match (e.g.
   `1.2.0`, never `v1.2.0`). This is the single biggest constraint on monorepos:
   two plugins in one repo cannot both publish a `1.0.0` release, and
   prefixed tags like `bob-vim-surround-1.2.0` won't match what BRAT/the store
   look for. ([Versions docs], [Submit your plugin], [BRAT dev guide])
3. **The community store is one-repo-per-plugin.** The `obsidian-releases`
   registry maps each plugin id to a single GitHub repo and pulls release
   assets from it. The forum's "Plugin Monorepo" thread confirms Obsidian's
   submission process "fundamentally expects a one-repo-per-plugin model," and
   that monorepo plugins are not available in the Community Plugins list.
   ([Plugin Monorepo thread])
4. **Avoid two sources of truth.** If the plugins stay tracked in the vault repo
   *and* live in the new repo, they will drift. Exactly one repo should own
   them; the other side should be generated/ignored.

## Options

### A. Topology: where the code lives

**A1 — Single monorepo (recommended).** One repo, e.g.
`bob-obsidian-plugins/`, with a folder per plugin at the root:

```
bob-obsidian-plugins/
  README.md                  # index + install instructions
  LICENSE                    # MIT (or your choice)
  .gitignore
  block-id-prompt/
    manifest.json
    main.js
  bob-ledger-tools/
    manifest.json
    main.js
  bob-navigation-hotkeys/
    manifest.json
    main.js
    styles.css
  bob-project-tasks/
    manifest.json
    main.js
  bob-vim-surround/
    manifest.json
    main.js
  task-status-cycler/
    manifest.json
    main.js
```

- *Pros:* one repo to clone/issue-track/CI; matches "a dedicated repo"
  (singular); easy to share conventions and later extract a shared helper
  module; one place to lint/format all six; trivial because there's no build.
- *Cons:* cannot drive BRAT or the community store for more than one plugin
  (tag collision, constraint #2/#3); per-plugin releases need prefixed tags
  which are then "manual download" only.

**A2 — One repo per plugin (six repos).** Mirrors the official model.

- *Pros:* directly compatible with BRAT and the community store; each plugin has
  clean independent tags (`1.2.0`), releases, issues, and README.
- *Cons:* six repos to create and maintain; six clones to wire into the vault;
  heavier for what is mostly personal tooling; harder to share code.

**A3 — Hybrid (monorepo dev + per-plugin publish repo).** Develop in the
monorepo; for any plugin you publish, mirror that one folder to its own
dedicated repo that carries the real releases (the `polyipseity` pattern).

- *Pros:* best of both — unified dev, store/BRAT-compatible publishing.
- *Cons:* most moving parts (a publish/mirror step per published plugin);
  overkill until you actually want to publish.

### B. How the vault gets the files (the "deploy" loop)

Because there is no build, the deployed files are identical to the repo files.
Four ways to bridge repo → `.obsidian/plugins/<id>/`:

- **B1 — `bob` sync command / script (recommended for this vault).** A
  `bob obsidian-plugins sync` (or `make sync` / `justfile`) that copies each
  plugin folder from a local monorepo clone into the vault. Keeps **real files**
  in the vault, so git-sync across machines keeps working and the vault stays
  self-contained. Fits the existing Rust `bob` CLI. Cost: you must re-run it
  after edits (or pair with a watch mode).
- **B2 — Symlinks.** `ln -s <clone>/bob-vim-surround
  <vault>/.obsidian/plugins/bob-vim-surround`. Instant edit loop, no copy step.
  Good on a single desktop. Cost: a symlink committed to the vault is just a
  dangling link on any other machine that lacks the clone; Obsidian on some
  platforms is finicky about symlinked plugin dirs. Since the vault is
  git-synced (not Obsidian-Sync), symlinks do **not** propagate usefully to
  other machines.
- **B3 — Git submodule(s).** Add the dedicated repo as a submodule and copy/link
  from it. Cleanly versions "which plugin commit the vault uses," but nesting a
  submodule beneath the already-tracked `.obsidian/plugins/` is awkward, and you
  still need a copy/link step into the per-id folders.
- **B4 — BRAT install into the vault.** Point BRAT at the dedicated repo's
  releases; it downloads the assets into the vault and auto-updates. Cleanest
  "just keep me current" loop and fully decouples the vault from the source —
  but requires real GitHub Releases (constraint #2), so it only works
  one-plugin-per-repo (topology A2/A3), not from a monorepo.

### C. Keep plain JS vs. adopt a TypeScript build

- **C1 — Stay plain JS (recommended now).** Zero build, source == artifact,
  nothing new to learn, matches today's reality. Lowest friction.
- **C2 — Adopt TS + esbuild (the official template).** Better types,
  refactoring, and it's what store reviewers expect to *see* as source. But it
  introduces a build step and a source/artifact gap you don't have today, and
  none of the six currently need it. Revisit only per-plugin, when one grows
  complex or you decide to publish it. (Plain-JS source is still acceptable to
  the store; TS is a convention, not a requirement.)

## Comparison

| Concern | A1 Monorepo | A2 Per-plugin | A3 Hybrid |
| --- | --- | --- | --- |
| Repos to manage | 1 | 6 | 1 + N published |
| Matches "a dedicated repo" | ✅ | ❌ (six) | ➖ |
| Community store ready | ❌ | ✅ | ✅ (published ones) |
| BRAT auto-update | ❌ (tag clash) | ✅ | ✅ (published ones) |
| Shared code / unified CI | ✅ | ❌ | ✅ |
| Setup effort | Low | Medium | High |
| Best for | Personal use | Public plugins | Personal + selective publish |

## Recommendation

1. **Create one monorepo `bob-obsidian-plugins` (topology A1).** It directly
   satisfies the "a dedicated repo" goal, separates plugin code from personal
   notes, and gives you issues/CI/history/README in one place. Keep each plugin
   in its own root folder named by its manifest `id`.

2. **Stay plain JS (C1).** Don't add a TypeScript/esbuild pipeline yet — there's
   no gap to bridge and no plugin currently warrants it. Add ESLint/Prettier and
   a trivial CI check ("every `main.js` parses, every `manifest.json` is valid
   and its `id` matches its folder") to catch breakage. There are currently no
   tests; that's fine to start.

3. **Make the new repo the single source of truth and stop tracking the plugins
   in the vault (constraint #4).** Add the six folders to the vault's
   `.gitignore`, then deploy via a **`bob obsidian-plugins sync` copy command
   (B1)** so the vault still holds real files for git-sync across machines. If
   you only ever work on one desktop, symlinks (B2) are a fine, lower-effort
   substitute. Either way, the vault no longer commits plugin source.

4. **Releases: per-plugin tags with prefixes** (e.g. `bob-vim-surround/1.2.0`),
   each GitHub Release attaching that plugin's `manifest.json`, `main.js`, and
   `styles.css`. This gives you versioned, downloadable history. Be explicit that
   these prefixed-tag releases are **manual-install only** — they intentionally
   do **not** match what BRAT/the store expect.

5. **Publish selectively, later, by splitting out (A3 only when needed).** If you
   decide to share a plugin publicly, move *that one* into its own dedicated repo
   so tag == version and BRAT/the store work. Good general-purpose candidates:
   **`bob-vim-surround`** and **`block-id-prompt`**. The workflow-specific ones
   (**`bob-ledger-tools`**, **`bob-project-tasks`**,
   **`bob-navigation-hotkeys`**, **`task-status-cycler`**) are tightly coupled to
   Bob conventions (pomodoro ledgers, `#task`, project frontmatter) and are best
   left monorepo-only.

## Suggested Migration Steps

1. **Decide on history.** If you want each plugin's commit history carried over,
   use `git filter-repo` (or `git subtree split -P .obsidian/plugins/<id>`) on a
   clone of the vault to extract each folder, then assemble them into the new
   repo. If history doesn't matter, a clean `git init` + snapshot is simpler and
   avoids dragging vault history along.
2. **Create `bob-obsidian-plugins`** with the folder-per-plugin layout, a
   top-level `README.md` (index + "install via `bob ... sync` / manual
   download"), a per-plugin `README.md`, `LICENSE`, and `.gitignore`.
3. **Add lint/CI** (ESLint + manifest/id validation). Optional: a `versions.json`
   per plugin — only needed if you ever raise a plugin's `minAppVersion` (all are
   `1.8.7` today, so not required yet).
4. **Wire deployment:** implement `bob obsidian-plugins sync` (copy clone →
   vault) or create the symlinks; verify Obsidian still loads all six and they
   stay enabled in `community-plugins.json`.
5. **Cut over the vault:** add the six plugin folders to the vault `.gitignore`,
   `git rm --cached` them from the vault, and commit. From here the monorepo is
   the only source of truth.
6. **Tag initial releases** (`block-id-prompt/1.0.0`, `bob-vim-surround/1.2.0`,
   …) so versions are reproducible.

## Open Questions For Bryan

- **Public or private?** Is the goal mainly to *separate* the code (private repo
  is fine) or to *share/publish* some plugins (then plan the A3 split for those)?
  This is the main fork in the road.
- **One desktop or many machines?** If you only develop on one machine, symlinks
  (B2) are the least-effort deploy; across machines, prefer the `bob` copy-sync
  (B1).
- **Worth a `bob` subcommand?** Given the existing Rust CLI, a first-class
  `bob obsidian-plugins {sync,status,bump}` would make this ergonomic and is the
  natural integration point — but a `Makefile`/`justfile` is a lighter start.
- **Preserve history?** Decides between `git filter-repo`/`subtree` extraction
  vs. a clean snapshot.

## Sources

- [Obsidian sample plugin template (repo layout, build expectations)](https://github.com/obsidianmd/obsidian-sample-plugin)
- [Submit your plugin — Developer Docs (release naming, asset requirements)][Submit your plugin]
- [Versions — Developer Docs (`versions.json`, `minAppVersion`, tag == version)][Versions docs]
- [obsidianmd/obsidian-releases (community plugin registry, one-repo-per-plugin)](https://github.com/obsidianmd/obsidian-releases)
- [Plugin Submission Guidelines (DeepWiki)](https://deepwiki.com/obsidianmd/obsidian-plugin-docs/6.2-plugin-submission-guidelines)
- [BRAT plugin (install/update from GitHub releases)](https://community.obsidian.md/plugins/obsidian42-brat)
- [BRAT Developer Guide (release assets + exact tag/name/version match)][BRAT dev guide]
- [Forum: "Plugin Monorepo" (Obsidian expects one-repo-per-plugin)][Plugin Monorepo thread]
- [polyipseity/obsidian-monorepo (community monorepo example)](https://github.com/polyipseity/obsidian-monorepo)
- [auxvirtua/obsidian (mono-repo of plugins and themes)](https://github.com/minischetti/obsidian)
- [A more streamlined Obsidian plugin dev workflow (symlink/watch)](https://medium.com/@lukasbach/a-more-streamlined-development-workflow-for-obsidian-plugins-2a74b0c57c0f)

[Submit your plugin]: https://docs.obsidian.md/Plugins/Releasing/Submit+your+plugin
[Versions docs]: https://docs.obsidian.md/Reference/Versions
[BRAT dev guide]: https://github.com/TfTHacker/obsidian42-brat/blob/main/BRAT-DEVELOPER-GUIDE.md
[Plugin Monorepo thread]: https://forum.obsidian.md/t/plugin-monorepo/90167
