// Local state from Rust scanner
export interface LocalState {
  skills: LocalSkill[];
  agents: LocalAgent[];
  hooks: Hook[];
  plugins: InstalledPlugin[];
  mcp_servers: McpServer[];
  teams: Team[];
  rules: Rule[];
  statuslines: Statusline[];
  claude_dir: string;
  codex_config: CodexConfig | null;
  codex_rules: CodexRule[];
  codex_skills: CodexSkill[];
  codex_agents: CodexAgent[];
}

export interface LocalSkill {
  name: string;
  description: string;
  path: string;
  file_count: number;
  has_scripts: boolean;
  has_subagents: boolean;
  has_references: boolean;
  has_statusline: boolean;
  source: 'skillvault' | 'local';
  package_id: string | null;
  installed_version: string | null;
  project: string | null;  // null = global, "patina" = project-scoped
}

export interface LocalAgent {
  name: string;
  description: string;
  path: string;
}

export interface Hook {
  event: string;
  matcher: string | null;
  hook_type: string;
  command: string;
}

export interface InstalledPlugin {
  name: string;
  marketplace: string;
  version: string;
  scope: string;
}

export interface McpServer {
  name: string;
  server_type: string;
  url: string | null;
  command: string | null;
}

export interface Team {
  name: string;
  description: string | null;
  member_count: number;
  path: string;
}

export interface Rule {
  name: string;
  path: string;
  project_path: string | null;
  size_bytes: number;
  preview: string;
}

export interface Statusline {
  name: string;
  path: string;
  language: string;
  size_bytes: number;
  preview: string;
}

export interface CodexConfig {
  model: string | null;
  trusted_projects: string[];
  config_path: string;
}

export interface CodexRule {
  name: string;
  path: string;
  preview: string;
  project: string | null;
}

export interface CodexSkill {
  name: string;
  path: string;
  description: string;
  project: string | null;
}

export interface CodexAgent {
  name: string;
  project: string | null;
  path: string;
}

// SkillVault API types
export interface Package {
  id: string;
  author_id: string;
  name: string;
  display_name: string | null;
  tagline: string | null;
  description: string | null;
  category: string;
  tags: string | null;
  pricing_type: string;
  price_cents: number;
  download_count: number;
  star_count: number;
  current_version: string;
  compat_claude_code: number | null;
  compat_cursor: number | null;
  compat_codex: number | null;
  compat_copilot: number | null;
  compat_gemini: number | null;
  author_display_name: string | null;
  author_avatar_url: string | null;
  license: string | null;
  repo_url: string | null;
  homepage_url: string | null;
  has_skills: number | null;
  has_agents: number | null;
  has_hooks: number | null;
  review_count: number | null;
  avg_rating: number | null;
  current_version_size: number | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface PackageSearchResult {
  packages: Package[];
  total: number;
  page: number;
  limit: number;
}

export interface CategoryCount {
  category: string;
  count: number;
}

export interface PlatformStats {
  total_packages: number;
  total_downloads: number;
  total_authors: number;
  total_reviews: number;
}

export interface UpdateInfo {
  skill_name: string;
  package_id: string;
  installed_version: string;
  latest_version: string;
}

export interface AuthStatus {
  authenticated: boolean;
  username: string | null;
}

export interface SkillDetail {
  name: string;
  path: string;
  description: string;
  skill_md_content: string;
  files: SkillFile[];
  source: string;
  package_id: string | null;
  installed_version: string | null;
}

export interface SkillFile {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
}

export interface MarketplacePlugin {
  name: string;
  description: string;
  category: string | null;
  author_name: string | null;
  author_url: string | null;
  homepage: string | null;
  keywords: string[];
  source: string;
  is_installed: boolean;
  installed_version: string | null;
  installed_at: string | null;
}

export interface PluginDetail {
  name: string;
  description: string;
  category: string | null;
  author_name: string | null;
  author_url: string | null;
  homepage: string | null;
  keywords: string[];
  source: string;
  is_installed: boolean;
  installed_version: string | null;
  installed_at: string | null;
  install_path: string | null;
  readme: string | null;
}

export interface PackagedSkill {
  name: string;
  description: string;
  zip_base64: string;
  file_count: number;
  size_bytes: number;
  skill_names: string[];
}

export type ViewName = 'installed' | 'browse' | 'recent' | 'trending' | 'detail' | 'skill-detail' | 'file-detail' | 'settings' | 'plugins' | 'plugin-detail' | 'publish' | 'edit-package';
