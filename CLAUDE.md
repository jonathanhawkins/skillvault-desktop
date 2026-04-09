# SkillVault Desktop — Claude Code Instructions

## What is this project?

SkillVault Desktop is an open-source macOS companion app for the SkillVault marketplace (skillvault.md). A "mod manager" for AI coding skills. Browse, install, uninstall, publish, and manage SKILL.md packages locally. Supports both Claude Code and Codex (OpenAI).

## Tech stack

- **Framework:** Tauri 2.0 (Rust backend + WebKit frontend)
- **Backend:** Rust — file scanning, zip extraction, HTTP client, local config auth
- **Frontend:** Vanilla TypeScript + Vite (no frameworks)
- **Design:** Dark theme CSS (Geist + Geist Mono fonts)
- **Token storage:** `~/.skillvault/config.json` (NOT keychain)

## Project structure

```
skillvault-desktop/
├── src-tauri/
│   └── src/
│       ├── scanner/              # Reads ~/.claude/ and ~/.codex/
│       │   ├── skills.rs         # Claude skills (global + project-scoped)
│       │   ├── agents.rs         # Claude agents
│       │   ├── hooks.rs          # Hooks from settings.json
│       │   ├── plugins.rs        # Installed plugins registry
│       │   ├── mcp.rs            # MCP servers from settings.json
│       │   ├── teams.rs          # Team configs
│       │   ├── rules.rs          # CLAUDE.md rules + path decoding
│       │   ├── codex.rs          # Codex config, rules, skills, agents
│       │   ├── tests.rs          # 36 scanner tests
│       │   └── codex_tests.rs    # 11 Codex tests
│       ├── installer/            # Downloads + extracts packages
│       │   ├── mod.rs            # Install, uninstall, multi-skill extraction
│       │   └── tests.rs          # 6 installer tests
│       ├── api/
│       │   ├── client.rs         # HTTP client for skillvault.md (30s timeout)
│       │   ├── auth.rs           # Token storage (~/.skillvault/config.json)
│       │   └── tests.rs          # 3 API tests
│       ├── commands.rs           # 23 Tauri IPC commands + 8 plugin tests
│       ├── state.rs              # All data types
│       ├── lib.rs                # App entry + command registration
│       ├── watcher.rs            # File watcher (disabled in bundled builds)
│       └── main.rs
├── src/                          # TypeScript frontend
│   ├── styles/
│   │   ├── tokens.css            # Design tokens (colors, spacing, type scale)
│   │   ├── base.css              # Reset, typography, layout
│   │   ├── components.css        # Reusable component styles
│   │   └── views.css             # View-specific styles
│   ├── lib/
│   │   ├── api.ts                # 23 Tauri invoke wrappers
│   │   ├── router.ts             # Navigation + keyboard shortcuts + history
│   │   ├── state.ts              # Global app state
│   │   ├── types.ts              # All TypeScript interfaces
│   │   └── utils.ts              # Shared esc(), formatBytes(), formatNum()
│   ├── components/
│   │   ├── sidebar.ts            # Navigation sidebar
│   │   ├── package-card.ts       # SkillVault package cards
│   │   └── toast.ts              # Toast notifications
│   └── views/
│       ├── installed.ts          # My Skills — all local assets grouped by platform
│       ├── browse.ts             # Marketplace search with compat filter
│       ├── recent.ts             # New packages
│       ├── trending.ts           # Trending packages
│       ├── detail.ts             # Package detail + install location chooser
│       ├── skill-detail.ts       # Local skill detail (SKILL.md rendered)
│       ├── file-detail.ts        # Generic file viewer (agents, rules, configs)
│       ├── plugins.ts            # 355 plugins browser (119 Claude + 236 Codex)
│       ├── plugin-detail.ts      # Plugin detail + install/uninstall + README
│       ├── publish.ts            # Multi-skill publish wizard (3 steps)
│       └── settings.ts           # Account connection + keyboard shortcuts
├── docs/SKILLVAULT-DESKTOP-PRD.md
├── README.md
├── CONTRIBUTING.md
└── CLAUDE.md                     # This file
```

## Commands

```bash
npm run tauri dev                              # Dev mode (hot reload)
npm run tauri build -- --debug --bundles app   # Debug .app bundle
npm run tauri build                            # Production build (DMG)
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1  # 67 tests
npx tsc --noEmit                               # TypeScript type check
```

## Release & Deploy Process (MUST follow every time)

After building a new version:

1. **Bump version** in all 3 files: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
2. **Production build:** `npm run tauri build` — produces DMG at `src-tauri/target/release/bundle/dmg/`
3. **Commit & push** the version bump + code changes
4. **Create GitHub release with the DMG attached:**
   ```bash
   gh release create v0.X.Y \
     --title "SkillVault Desktop v0.X.Y" \
     --notes "Release notes here" \
     "src-tauri/target/release/bundle/dmg/SkillVault Desktop_0.X.Y_aarch64.dmg"
   ```
5. **Verify** the release is live: `gh release view v0.X.Y`

**Never skip step 4.** The DMG must be uploaded to GitHub Releases so users can download it. A local build without a release means nobody gets the update.

## Features built

- **Local scanner:** skills (global + per-project), agents, hooks, plugins, MCP servers, teams, rules
- **Codex scanner:** config.toml, rules, skills, orchestrator agents
- **SkillVault marketplace:** browse, search, filter by category/compatibility, new, trending
- **355 plugins browser** (119 Claude Code + 236 Codex from GitHub)
- **Plugin install/uninstall** (Claude via CLI, Codex via GitHub download)
- **One-click skill install** with project location chooser + "Choose Directory..."
- **Multi-skill publish wizard** (select multiple skills, bundle, upload)
- **Smart installer:** detects multi-skill packages, extracts to individual directories
- **Markdown rendering** in package/skill/plugin detail views
- **Platform filter** (Claude Code / Codex / All) on My Skills and Plugins
- **Keyboard shortcuts** (Cmd+1-7, Cmd+[/], Cmd+F, Cmd+R, Esc)
- **Navigation history** with back/forward
- **3-step account connection flow** (no keychain — uses `~/.skillvault/config.json`)

## Rules

1. **No frontend frameworks.** Vanilla TypeScript, HTML template strings. Same philosophy as SkillVault web.
2. **Dark theme only.** All colors via CSS custom properties in `tokens.css`.
3. **Geist + Geist Mono** fonts. Match the SkillVault web aesthetic.
4. **All file I/O in Rust.** Frontend communicates via Tauri IPC (`invoke`).
5. **API tokens in `~/.skillvault/config.json`.** NOT keychain. The `svt_` token only accesses SkillVault's API, not GitHub. Stored as plain JSON (same pattern as `~/.npmrc`).
6. **Security: validate zip paths.** Use `is_absolute` + `starts_with` checks — skip entries with `..` path traversal.
7. **Never use `confirm()` or `alert()` in frontend code.** Tauri's WebKit blocks on browser dialogs and freezes/crashes the app. Use inline confirmation UI instead (e.g., two-click confirm buttons, inline "Remove? [Yes] [No]").
8. **Claude Code path encoding.** Project directories under `~/.claude/projects/` use `-` to replace `/` (e.g., `-Users-bone-dev-web-apps-skill-vault`). Simple `.replace('-', '/')` breaks paths with hyphens in directory names (like `web-apps`). Always use the smart recursive `decode_project_path` function in `scanner/rules.rs` that tries all possible segment joins and validates against the filesystem.
9. **Validate names with `validate_name()` regex** before any filesystem or CLI use.

## macOS Icon Rules (CRITICAL)

1. **Always use `cargo tauri icon <source.png>` to generate icons.** Never manually create or resize icon PNGs — Tauri's `generate_context!()` macro embeds icons at compile time and validates that pixel dimensions match filenames exactly. Wrong dimensions = compile panic.
2. **Source icon must be 1024x1024 PNG with NO alpha channel.** Use `sips -g hasAlpha icon.png` to verify. If alpha exists, the icon gets a white background in the macOS dock.
3. **macOS does NOT auto-apply squircle mask to app icons.** Unlike iOS, you must bake rounded corners into the icon image itself. The standard macOS corner radius is ~22.37% of the icon size.
4. **Dev mode (`cargo tauri dev`) does NOT show your custom icon in the dock.** It shows a generic terminal icon. To test the real icon, build an .app bundle: `npm run tauri build -- --debug --bundles app` then `open src-tauri/target/debug/bundle/macos/SkillVault\ Desktop.app`.
5. **After changing icons, do a clean build.** `cargo clean --manifest-path src-tauri/Cargo.toml` then rebuild. macOS also caches dock icons — restart Dock with `killall Dock` if the icon doesn't update.
6. **Never overwrite icon files with Swift-generated retina PNGs.** Swift's `NSImage` renders at 2x on retina displays — a "512x512" image actually contains 1024x1024 pixels. Tauri expects filenames to match actual pixel counts.

## .setup() and Background Threads

- The `.setup()` closure in `lib.rs` runs during `did_finish_launching`. If it panics, the entire app crashes with `panic_cannot_unwind` in the `tao` event loop — this is unrecoverable.
- Background threads spawned in `.setup()` (file watcher, auto-update) must never panic. Wrap them in `catch_unwind` or handle all errors gracefully.
- If the bundled app crashes on launch but dev mode works, check `.setup()` first — the bundled environment differs from dev mode.
- Currently `.setup()` is disabled to avoid crashes. Re-enable carefully when ready.

## Testing

67 tests across all modules:

| Module | Tests |
|--------|-------|
| Scanner | 36 (skills, agents, hooks, plugins, MCP, teams, rules) |
| Codex scanner | 11 |
| Commands | 8 (plugin install/uninstall) |
| Installer | 6 |
| API client | 3 |
| Auth | 3 |

Run with `--test-threads=1` to avoid parallel test interference (tests share filesystem state).
