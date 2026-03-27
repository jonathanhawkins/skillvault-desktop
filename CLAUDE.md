# SkillVault Desktop — Claude Code Instructions

## What is this project?

SkillVault Desktop is an open-source macOS companion app for the SkillVault marketplace (skillvault.md). It's a "mod manager" for AI coding skills — browse, install, uninstall, update, and publish SKILL.md packages for Claude Code.

## Tech stack

- **Framework:** Tauri 2.0 (Rust backend + WebKit frontend)
- **Backend:** Rust (file scanning, zip extraction, HTTP client, local config)
- **Frontend:** Vanilla TypeScript + Vite (no frameworks)
- **Design:** Dark theme CSS ported from SkillVault web (Geist fonts)

## Project structure

```
skillvault-desktop/
├── src-tauri/          # Rust backend
│   └── src/
│       ├── scanner/    # Reads ~/.claude/ (skills, agents, hooks, plugins)
│       ├── installer/  # Downloads + extracts packages from API
│       ├── api/        # HTTP client for skillvault.md + local config auth
│       ├── commands.rs # Tauri IPC command handlers
│       └── state.rs    # App state + data types
├── src/                # TypeScript frontend
│   ├── styles/         # CSS (tokens, base, components, views)
│   ├── lib/            # API wrappers, state, router, types
│   ├── components/     # Sidebar, package cards, toast
│   └── views/          # Installed, Browse, Trending, Detail, Settings
└── docs/               # PRD
```

## Commands

```bash
npm run dev                              # Start Vite dev server (frontend only)
npm run tauri dev                        # Start full Tauri app in dev mode (frontend + Rust backend)
npm run tauri build -- --debug --bundles app  # Build debug .app bundle (for testing icons, bundled behavior)
npm run tauri build                      # Build production app (DMG)
cargo test --manifest-path src-tauri/Cargo.toml  # Run Rust tests (16 tests)
npx tsc --noEmit                         # TypeScript type check
```

## Rules

1. **No frontend frameworks.** Vanilla TypeScript, HTML template strings. Same philosophy as SkillVault web.
2. **Dark theme only** for MVP. All colors via CSS custom properties in tokens.css.
3. **Geist + Geist Mono** fonts. Match the SkillVault web aesthetic.
4. **All file I/O in Rust.** Frontend communicates via Tauri IPC (`invoke`).
5. **API tokens in ~/.skillvault/config.json.** The svt_ token only accesses SkillVault's API, not GitHub. Stored as plain JSON (same pattern as ~/.npmrc, ~/.svrc).
6. **Security: validate zip paths.** Skip entries with `..` path traversal.
7. **Never use `confirm()` or `alert()` in frontend code.** Tauri's WebKit blocks on browser dialogs and it freezes/crashes the app. Use inline confirmation UI instead (e.g., two-click confirm buttons, inline "Remove? [Yes] [No]").
8. **Claude Code path encoding.** Project directories under `~/.claude/projects/` use `-` to replace `/` (e.g., `-Users-bone-dev-web-apps-skill-vault`). Simple `.replace('-', '/')` breaks paths with hyphens in directory names (like `web-apps`, `skill-vault`). Always use the smart recursive `decode_project_path` function in `scanner/rules.rs` that tries all possible segment joins and validates against the filesystem.

## macOS Icon Rules (CRITICAL)

1. **Always use `cargo tauri icon <source.png>` to generate icons.** Never manually create or resize icon PNGs — Tauri's `generate_context!()` macro embeds icons at compile time and validates that pixel dimensions match filenames exactly. Wrong dimensions = compile panic.
2. **Source icon must be 1024x1024 PNG with NO alpha channel.** Use `sips -g hasAlpha icon.png` to verify. If alpha exists, the icon gets a white background in the macOS dock.
3. **macOS does NOT auto-apply squircle mask to app icons.** Unlike iOS, you must bake rounded corners into the icon image itself if you want them. The standard macOS corner radius is ~22.37% of the icon size.
4. **Dev mode (`cargo tauri dev`) does NOT show your custom icon in the dock.** It shows a generic terminal icon. To test the real icon, build an .app bundle: `npm run tauri build -- --debug --bundles app` then `open src-tauri/target/debug/bundle/macos/SkillVault\ Desktop.app`.
5. **After changing icons, do a clean build.** `cargo clean --manifest-path src-tauri/Cargo.toml` then rebuild. macOS also caches dock icons — restart Dock with `killall Dock` if the icon doesn't update.
6. **Never overwrite icon files with Swift-generated retina PNGs.** Swift's `NSImage` renders at 2x on retina displays — a "512x512" image actually contains 1024x1024 pixels. Tauri expects filenames to match actual pixel counts.

## .setup() and Background Threads

- The `.setup()` closure in `lib.rs` runs during `did_finish_launching`. If it panics, the entire app crashes with `panic_cannot_unwind` in the `tao` event loop — this is unrecoverable.
- Background threads spawned in `.setup()` (file watcher, auto-update) must never panic. Wrap them in `catch_unwind` or handle all errors gracefully.
- If the bundled app crashes on launch but dev mode works, check `.setup()` first — the bundled environment differs from dev mode.
- Currently `.setup()` is disabled to avoid crashes. Re-enable carefully when ready.
