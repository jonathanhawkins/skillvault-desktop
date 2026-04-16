use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkill {
    pub name: String,
    pub description: String,
    pub path: String,
    pub file_count: u32,
    pub has_scripts: bool,
    pub has_subagents: bool,
    pub has_references: bool,
    pub has_statusline: bool,
    pub source: SkillSource,
    pub package_id: Option<String>,
    pub installed_version: Option<String>,
    pub project: Option<String>,  // None = global, Some("patina") = project-scoped
    /// True when this skill's local files have been modified after the last install/publish sync.
    /// Computed by comparing max file mtime vs SkillvaultMeta.synced_at (fallback: installed_at).
    /// Always false for SkillSource::Local (nothing to sync to).
    #[serde(default)]
    pub has_local_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    Skillvault,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAgent {
    pub name: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub event: String,
    pub matcher: Option<String>,
    pub hook_type: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub name: String,
    pub marketplace: String,
    pub version: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub server_type: String,
    pub url: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub description: Option<String>,
    pub member_count: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub path: String,
    pub project_path: Option<String>,
    pub size_bytes: u64,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statusline {
    pub name: String,
    pub path: String,
    pub language: String,  // "bash", "python", "javascript", "typescript"
    pub size_bytes: u64,
    pub preview: String,
    /// True when this statusline was installed/published from SkillVault and the local
    /// files have been modified since the last sync. False for single-file statuslines
    /// and packages without .skillvault-meta.json.
    #[serde(default)]
    pub has_local_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub model: Option<String>,
    pub trusted_projects: Vec<String>,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexRule {
    pub name: String,
    pub path: String,
    pub preview: String,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSkill {
    pub name: String,
    pub path: String,
    pub description: String,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAgent {
    pub name: String,
    pub project: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalState {
    pub skills: Vec<LocalSkill>,
    pub agents: Vec<LocalAgent>,
    pub hooks: Vec<Hook>,
    pub plugins: Vec<InstalledPlugin>,
    pub mcp_servers: Vec<McpServer>,
    pub teams: Vec<Team>,
    pub rules: Vec<Rule>,
    pub statuslines: Vec<Statusline>,
    pub claude_dir: String,
    pub codex_config: Option<CodexConfig>,
    pub codex_rules: Vec<CodexRule>,
    pub codex_skills: Vec<CodexSkill>,
    pub codex_agents: Vec<CodexAgent>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub auth_token: Option<String>,
    pub username: Option<String>,
    pub local_state: Option<LocalState>,
    pub codex_plugins_cache: Option<Vec<MarketplacePlugin>>,
}

// SkillVault API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Package {
    pub id: String,
    pub author_id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub category: String,
    pub tags: Option<String>,
    pub pricing_type: String,
    pub price_cents: i32,
    pub download_count: i32,
    pub star_count: i32,
    pub current_version: String,
    pub compat_claude_code: Option<i32>,
    pub compat_cursor: Option<i32>,
    pub compat_codex: Option<i32>,
    pub compat_copilot: Option<i32>,
    pub compat_gemini: Option<i32>,
    pub compat_claude_ai: Option<i32>,
    pub compat_other: Option<String>,
    pub author_display_name: Option<String>,
    pub author_avatar_url: Option<String>,
    // Full detail fields (nullable in list results)
    pub license: Option<String>,
    pub repo_url: Option<String>,
    pub homepage_url: Option<String>,
    pub has_skills: Option<i32>,
    pub has_agents: Option<i32>,
    pub has_hooks: Option<i32>,
    pub has_docs: Option<i32>,
    pub has_rules: Option<i32>,
    pub has_claude_md: Option<i32>,
    pub has_settings_json: Option<i32>,
    pub has_statusline: Option<i32>,
    pub has_commands: Option<i32>,
    pub has_scripts: Option<i32>,
    pub review_count: Option<i32>,
    pub avg_rating: Option<f64>,
    pub current_version_size: Option<i32>,
    pub download_count_7d: Option<i32>,
    pub published_at: Option<String>,
    pub is_published: Option<i32>,
    pub is_featured: Option<i32>,
    pub is_flagged: Option<i32>,
    pub skill_names: Option<String>,
    pub agent_names: Option<String>,
    pub command_names: Option<String>,
    pub security_reviewed: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for Package {
    fn default() -> Self {
        Self {
            id: String::new(),
            author_id: String::new(),
            name: String::new(),
            display_name: None,
            tagline: None,
            description: None,
            category: String::new(),
            tags: None,
            pricing_type: String::from("free"),
            price_cents: 0,
            download_count: 0,
            star_count: 0,
            current_version: String::from("0.0.0"),
            compat_claude_code: None,
            compat_cursor: None,
            compat_codex: None,
            compat_copilot: None,
            compat_gemini: None,
            compat_claude_ai: None,
            compat_other: None,
            author_display_name: None,
            author_avatar_url: None,
            license: None,
            repo_url: None,
            homepage_url: None,
            has_skills: None,
            has_agents: None,
            has_hooks: None,
            has_docs: None,
            has_rules: None,
            has_claude_md: None,
            has_settings_json: None,
            has_statusline: None,
            has_commands: None,
            has_scripts: None,
            review_count: None,
            avg_rating: None,
            current_version_size: None,
            download_count_7d: None,
            published_at: None,
            is_published: None,
            is_featured: None,
            is_flagged: None,
            skill_names: None,
            agent_names: None,
            command_names: None,
            security_reviewed: None,
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSearchResult {
    pub packages: Vec<Package>,
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub page: i32,
    #[serde(default)]
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesResult {
    pub categories: Vec<CategoryCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStats {
    pub total_packages: i32,
    pub total_downloads: i32,
    pub total_authors: i32,
    pub total_reviews: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub encoded_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub name: String,
    pub path: String,
    pub description: String,
    pub skill_md_content: String,
    pub files: Vec<SkillFile>,
    pub source: String,
    pub package_id: Option<String>,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub source: String,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub installed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDetail {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub source: String,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub installed_at: Option<String>,
    pub install_path: Option<String>,
    pub readme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillvaultMeta {
    pub source: String,
    pub package_id: String,
    pub version: String,
    pub installed_at: String,
    /// Timestamp of the last sync (install or successful publish).
    /// When absent on legacy files, fall back to installed_at.
    #[serde(default)]
    pub synced_at: String,
    pub auto_update: bool,
}

// Optimizer types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OptimizationProfile {
    #[serde(default = "default_thinking_tokens")]
    pub max_thinking_tokens: u32,
    #[serde(default = "default_autocompact")]
    pub autocompact_pct: u32,
    #[serde(default = "default_true")]
    pub disable_adaptive_thinking: bool,
    #[serde(default = "default_true")]
    pub always_thinking_enabled: bool,
    pub auto_background_tasks: bool,
    pub no_flicker: bool,
    pub skip_permissions: bool,
    pub use_tmux: bool,
    pub experimental_agent_teams: bool,
    #[serde(default)]
    pub task_list_id: String,
    #[serde(default)]
    pub extra_cli_args: String,
    /// Model alias or full ID (e.g. "opus", "sonnet", "claude-opus-4-7"). Empty = no override.
    #[serde(default)]
    pub model: String,
    /// Effort level: "low", "medium", "high", "max", "auto". Empty = no override.
    #[serde(default)]
    pub effort_level: String,
}

fn default_thinking_tokens() -> u32 { 50000 }
fn default_autocompact() -> u32 { 40 }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStatus {
    pub always_thinking_enabled: bool,
    pub disable_adaptive_thinking: Option<String>,
    pub max_thinking_tokens: Option<String>,
    pub autocompact_pct_override: Option<String>,
    pub optimization_score: u8,
    pub settings_json_exists: bool,
    pub shell_profile_path: String,
    pub shell_block_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTerminal {
    pub name: String,
    pub app_path: String,
    pub icon_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectWithLaunchScript {
    pub name: String,
    pub path: String,
    pub encoded_name: String,
    pub has_launch_script: bool,
    pub launch_script_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagedSkill {
    pub name: String,
    pub description: String,
    pub zip_base64: String,
    pub file_count: u32,
    pub size_bytes: u64,
    pub skill_names: Vec<String>,
}
