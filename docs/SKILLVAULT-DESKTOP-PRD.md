# SkillVault Desktop — Product Requirements Document

## Overview

SkillVault Desktop is an open-source macOS companion app for the [SkillVault](https://skillvault.md) marketplace. It functions as a "mod manager" for AI coding skills — enabling users to browse, install, uninstall, update, and publish SKILL.md packages for Claude Code and other AI coding tools, all through a native desktop GUI.

Think CurseForge/Vortex for game mods, but for Claude Code skills, agents, hooks, rules, and configurations.

## Problem Statement

Currently, managing Claude Code skills requires:
1. Manually downloading zip files from skillvault.md
2. Extracting them to `~/.claude/skills/` by hand
3. No visibility into what's installed or whether updates exist
4. No easy way to move skills between projects or machines
5. Publishing requires CLI knowledge and manual packaging

Users need a visual tool that makes skill management as simple as installing a game mod.

## Target Users

- **Developers** who use Claude Code and want to extend it with community skills
- **Skill creators** who want to package and publish their work to SkillVault
- **Teams** who need to standardize Claude Code configurations across members

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Framework | Tauri 2.0 | Cross-platform, small bundle (5-15MB), native window chrome |
| Backend | Rust | Fast file scanning, safe zip extraction, config file auth |
| Frontend | Vanilla TypeScript + Vite | No framework overhead, matches SkillVault's philosophy |
| Design | CSS (ported from SkillVault) | Dark theme, Geist fonts, consistent marketplace aesthetic |
| Auth | Clerk via SkillVault API | `svt_` API tokens stored in `~/.skillvault/config.json` |

## Architecture

```
┌─────────────────────────────────────────────┐
│                  Tauri Shell                 │
│  ┌───────────────────────────────────────┐  │
│  │        Frontend (WebKit/WebView)      │  │
│  │  HTML + CSS + TypeScript (Vite)       │  │
│  │                                       │  │
│  │  Views: Installed | Browse | Detail   │  │
│  │         Settings | Publish            │  │
│  └──────────────┬────────────────────────┘  │
│                 │ Tauri IPC (invoke)         │
│  ┌──────────────┴────────────────────────┐  │
│  │          Rust Backend                  │  │
│  │                                       │  │
│  │  Scanner ─── reads ~/.claude/          │  │
│  │  API Client ─ talks to skillvault.md   │  │
│  │  Installer ── downloads + extracts     │  │
│  │  Publisher ── packages + uploads       │  │
│  │  Auth ─────── ~/.skillvault/config.json │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
         │                        │
         ▼                        ▼
   ~/.claude/              skillvault.md
   (local files)           (REST API)
```

## Claude Code Local File System

The app scans and manages these locations:

| Asset | Path | Format |
|-------|------|--------|
| Skills | `~/.claude/skills/*/SKILL.md` | Directory with SKILL.md, references/, scripts/, subagents/ |
| Agents | `~/.claude/agents/*.md` | Markdown agent definitions |
| Hooks | `~/.claude/settings.json` → `hooks` | JSON event → command mappings |
| Plugins | `~/.claude/plugins/installed_plugins.json` | JSON plugin registry |
| Teams | `~/.claude/teams/*/config.json` | Team config + inbox files |
| Projects | `~/.claude/projects/*/` | URL-encoded path directories |
| MCP Servers | `~/.claude/settings.json` → `mcpServers` | JSON server definitions |

Project-level files: `CLAUDE.md`, `AGENTS.md`, `.claude/` directories in repos.

### Codex (OpenAI) File System

The app also scans Codex directories:

| Asset | Path | Format |
|-------|------|--------|
| Config | `~/.codex/config.json` | JSON (model, trusted projects) |
| Rules | `~/.codex/instructions.md`, project `.codex/instructions.md` | Markdown |
| Skills | `~/.codex/skills/*/SKILL.md` | Directory with SKILL.md |
| Agents | `~/.codex/agents/*.md`, project `.codex/agents/*.md` | Markdown agent definitions |

## SkillVault API Integration

### Public Endpoints (no auth)
- `GET /api/packages` — Search (q, category, tags, compat, pricing, sort, page, limit)
- `GET /api/packages/:author/:name` — Package details
- `GET /api/packages/:author/:name/download` — Download zip
- `GET /api/categories` — Category list
- `GET /api/trending` — Top 20 trending
- `GET /api/stats` — Platform statistics
- `GET /api/authors` — Author list

### Authenticated Endpoints (svt_ token)
- `GET /api/me` — User profile
- `POST /api/packages` — Create package
- `PUT /api/packages/:author/:name/upload` — Upload version
- `POST /api/packages/:author/:name/star` — Toggle star
- `POST /api/packages/:author/:name/reviews` — Submit review

## Data Models

### LocalSkill (scanned from ~/.claude/skills/)
```typescript
interface LocalSkill {
  name: string;                    // Directory name
  description: string;             // From SKILL.md frontmatter
  path: string;                    // Absolute path
  file_count: number;              // Total files in directory
  has_scripts: boolean;            // Has scripts/ subdirectory
  has_subagents: boolean;          // Has subagents/ subdirectory
  has_references: boolean;         // Has references/ subdirectory
  source: 'skillvault' | 'local'; // Where it came from
  package_id: string | null;       // "author/name" if from SkillVault
  installed_version: string | null; // Semver if from SkillVault
  update_available: string | null;  // Newer version if exists
}
```

### SkillVault Meta File (.skillvault-meta.json)
Written alongside installed skills to track their origin:
```json
{
  "source": "skillvault",
  "package_id": "author/name",
  "version": "1.2.0",
  "installed_at": "2026-03-26T...",
  "auto_update": true
}
```

### CodexConfig (scanned from ~/.codex/)
```typescript
interface CodexConfig {
  model: string | null;
  trusted_projects: string[];
  config_path: string;
}
```

### CodexRule
```typescript
interface CodexRule {
  name: string;
  path: string;
  preview: string;
  project: string | null;         // null = global
}
```

### CodexSkill
```typescript
interface CodexSkill {
  name: string;
  path: string;
  description: string;
  project: string | null;
}
```

### CodexAgent
```typescript
interface CodexAgent {
  name: string;
  project: string | null;
  path: string;
}
```

### MarketplacePlugin (unified plugin browser)
```typescript
interface MarketplacePlugin {
  name: string;
  description: string;
  category: string | null;
  author_name: string | null;
  author_url: string | null;
  homepage: string | null;
  keywords: string[];
  source: string;                  // "claude-code" | "codex"
  is_installed: boolean;
  installed_version: string | null;
  installed_at: string | null;
}
```

### PluginDetail
```typescript
interface PluginDetail extends MarketplacePlugin {
  install_path: string | null;
  readme: string | null;
}
```

### SkillDetail (local skill deep view)
```typescript
interface SkillDetail {
  name: string;
  path: string;
  description: string;
  skill_md_content: string;
  files: SkillFile[];
  source: string;
  package_id: string | null;
  installed_version: string | null;
}
```

### Package (from API — mirrors server types)
```typescript
interface Package {
  id: string;                      // "author/name"
  author_id: string;
  name: string;
  display_name: string | null;
  tagline: string | null;
  description: string | null;
  category: string;
  tags: string | null;             // JSON array
  pricing_type: 'free' | 'paid' | 'freemium';
  price_cents: number;
  download_count: number;
  star_count: number;
  current_version: string;
  compat_claude_code: number;
  compat_cursor: number;
  compat_codex: number;
  compat_copilot: number;
  compat_gemini: number;
}
```

## Core User Flows

### 1. Install a Skill
1. User browses marketplace or searches
2. Clicks package → sees detail view
3. Clicks "Install"
4. Rust downloads zip from API
5. Checks for name conflicts in `~/.claude/skills/`
6. Extracts to `~/.claude/skills/<name>/`
7. Writes `.skillvault-meta.json`
8. Shows success toast, refreshes local view

### 2. Uninstall a Skill
1. User right-clicks skill in "My Skills" → Uninstall
2. Confirmation dialog
3. Skill moved to `~/.claude/skills/.trash/<name>-<timestamp>/`
4. Recoverable for 7 days, then auto-cleaned

### 3. Update a Skill
1. Background task checks for updates every 30 minutes
2. Badge appears on skills with newer versions
3. User clicks "Update" → new version downloaded and extracted
4. Old version backed up to .trash

### 4. Authenticate
User creates `svt_` token on skillvault.md → pastes into Settings → token stored in `~/.skillvault/config.json`.
**Phase 2**: Deep-link OAuth via `skillvault://auth?token=...`

### 5. Publish a Skill
1. User navigates to "Publish" view
2. Selects one or more local skills to package
3. App validates SKILL.md format for each skill
4. Creates zip package (supports multi-skill packages)
5. Uploads via API with auth token

## Views / Screens

### Sidebar (persistent navigation)
- SkillVault logo + "Desktop" label
- MY SKILLS — local installed count badge
- BROWSE — marketplace search
- TRENDING — trending packages
- PLUGINS — 355 plugins (Claude Code + Codex)
- PUBLISH — package and publish skills
- RECENT — recent activity feed
- Divider
- SETTINGS — auth + preferences
- Bottom: user avatar + name (if signed in) or "Sign In"
- Keyboard shortcuts: Cmd+1-7 for views

### My Skills (default view)
- Grid of locally installed skills
- Each card: name, description, file count, source badge (SkillVault/Local)
- Green "update available" badge
- Context menu: Open in Finder, Uninstall, View on SkillVault
- Collapsible sections: Agents, Hooks, Plugins, Rules, MCP Servers, Teams
- Codex section: Codex config, rules, skills, agents

### Browse
- Search bar with filter chips: Category, Compatibility, Pricing, Sort
- Package card grid (same design as skillvault.md)
- Infinite scroll pagination

### Package Detail
- Hero: name, author, version, markdown description
- Install / Update / Installed button (global or project-scoped)
- Tabs: Overview, Files, Reviews, Versions
- Stats sidebar: compatibility dots, license, repo link, downloads, stars

### Plugins
- Unified browser for 119 Claude Code plugins + 236 Codex plugins
- Platform filter tabs (All / Claude Code / Codex)
- Search and category filtering
- Install/uninstall directly from the list
- Click through to Plugin Detail view

### Plugin Detail
- Plugin name, description, author, homepage link
- Keywords/tags
- Source badge (Claude Code / Codex)
- Install/Uninstall button
- README content (when available)

### Skill Detail
- Full SKILL.md content rendered
- File tree browser for all files in the skill directory
- Click through to File Detail for individual files
- Source and version info

### File Detail
- Read-only file content viewer
- Syntax-highlighted code display
- File path and size metadata

### Publish
- Select local skills to package
- Multi-skill package support
- Automatic SKILL.md detection and validation
- Package preview before upload
- Direct upload to SkillVault marketplace

### Recent
- Activity feed of recent installs, uninstalls, and updates
- Timestamps and action details

### Settings
- API token input (masked, stored in `~/.skillvault/config.json`)
- Claude Code path override (default: ~/.claude/)
- Scan interval preference
- About / version info

## Design System

Ported directly from SkillVault web:

```css
:root {
  --bg-primary: #020202;
  --bg-secondary: #101010;
  --bg-card: #1f1d1c;
  --text-primary: #eeeeee;
  --text-secondary: #a49d9a;
  --text-muted: #8a8380;
  --text-faint: #4d4947;
  --accent: #ee6018;
  --accent-hover: #ef6f2e;
  --border: #2e2c2b;
  --border-hover: #3d3a39;
  --success: #22c55e;
  --warning: #eab308;
  --error: #ef4444;
}
```

**Fonts**: Geist (400, 500, 600) + Geist Mono (400)
**Theme**: Dark only (MVP)
**Window**: 1200x800 default, 900x600 minimum

## Server-Side Changes Required

1. **CORS headers** — Add `Access-Control-Allow-Origin` to API JSON responses (desktop requests from `tauri://localhost`)
2. **`POST /api/packages/batch-check`** — Batch version check endpoint (avoids N+1)
3. **`/desktop-auth` page** (Phase 2) — SSR page for deep-link OAuth

## MVP Scope

### Built (complete)
- Tauri 2.0 project with Vite + TypeScript
- Rust scanner for skills, agents, hooks, rules, plugins, MCP servers, teams
- Codex scanner for config, rules, skills, agents
- "My Skills" view with local skill cards
- Marketplace browser (search, filter, trending)
- Package detail view
- One-click install/uninstall (global and project-scoped)
- Plugin browser (355 plugins: 119 Claude Code + 236 Codex) with platform filters
- Plugin install/uninstall
- Plugin detail view
- Skill detail view with file browser
- File detail view (read file contents)
- Multi-skill package publishing flow
- Recent activity view
- Update detection
- Manual svt_ token auth (stored in `~/.skillvault/config.json`)
- macOS DMG build
- File watcher for live ~/.claude/ changes
- Keyboard shortcuts (Cmd+1-7, Cmd+[/], Cmd+F, Cmd+R)
- 67 Rust tests

### Out (Phase 2+)
- Deep-link OAuth
- Star/review from desktop
- Windows/Linux builds
- Auto-update (Tauri updater)
- System tray / menu bar mode

## Rust Crate Dependencies

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2.6.0"       # Native file/directory dialogs
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "multipart"] }
tokio = { version = "1", features = ["full"] }
zip = "2"
notify = "7"              # File watcher
dirs = "6"                # Home directory resolution
base64 = "0.22"           # Encoding for package uploads
```

## Distribution

- **macOS**: DMG installer, Apple notarized
- **GitHub Releases**: Primary distribution channel
- **License**: MIT
- **Open source**: Full source on GitHub
