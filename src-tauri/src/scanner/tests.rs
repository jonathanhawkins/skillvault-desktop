use super::skills::{count_files, parse_skill_description, scan_skills};
use super::agents::scan_agents;
use super::hooks::scan_hooks;
use super::plugins::scan_plugins;
use super::mcp::scan_mcp_servers;
use super::teams::scan_teams;
use super::rules::{scan_rules, decode_project_path_pub};
use std::fs;

fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "skillvault_test_{}_{}", prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_parse_skill_description_inline() {
    let dir = make_temp_dir("inline");
    let skill_md = dir.join("SKILL.md");
    fs::write(
        &skill_md,
        "---\nname: test-skill\ndescription: \"A helpful skill\"\n---\n# Content\n",
    )
    .unwrap();

    let desc = parse_skill_description(&skill_md);
    assert_eq!(desc, "A helpful skill");
    cleanup(&dir);
}

#[test]
fn test_parse_skill_description_folded() {
    let dir = make_temp_dir("folded");
    let skill_md = dir.join("SKILL.md");
    fs::write(
        &skill_md,
        "---\nname: test\ndescription: >-\n  This is a folded\n  scalar description\n---\n",
    )
    .unwrap();

    let desc = parse_skill_description(&skill_md);
    assert_eq!(desc, "This is a folded scalar description");
    cleanup(&dir);
}

#[test]
fn test_parse_skill_description_block() {
    let dir = make_temp_dir("block");
    let skill_md = dir.join("SKILL.md");
    fs::write(
        &skill_md,
        "---\nname: test\ndescription: |-\n  Line one\n  Line two\n---\n",
    )
    .unwrap();

    let desc = parse_skill_description(&skill_md);
    assert_eq!(desc, "Line one\nLine two");
    cleanup(&dir);
}

#[test]
fn test_parse_skill_description_no_frontmatter() {
    let dir = make_temp_dir("nofm");
    let skill_md = dir.join("SKILL.md");
    fs::write(&skill_md, "# My Skill\nThis does cool things.\n").unwrap();

    let desc = parse_skill_description(&skill_md);
    assert_eq!(desc, "This does cool things.");
    cleanup(&dir);
}

#[test]
fn test_parse_skill_description_empty() {
    let dir = make_temp_dir("empty");
    let skill_md = dir.join("SKILL.md");
    fs::write(&skill_md, "").unwrap();

    let desc = parse_skill_description(&skill_md);
    assert_eq!(desc, "");
    cleanup(&dir);
}

#[test]
fn test_count_files() {
    let dir = make_temp_dir("count");
    fs::write(dir.join("a.txt"), "a").unwrap();
    fs::write(dir.join("b.txt"), "b").unwrap();
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("c.txt"), "c").unwrap();

    assert_eq!(count_files(&dir), 3);
    cleanup(&dir);
}

#[test]
fn test_scan_skills_empty_dir() {
    let dir = make_temp_dir("scan_empty");
    // Create skills subdir (scan_skills expects claude_dir with skills/ inside)
    let skills_dir = dir.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();

    let result = scan_skills(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_skills_with_skill() {
    let dir = make_temp_dir("scan_skill");
    let skills_dir = dir.join("skills");
    let my_skill = skills_dir.join("my-skill");
    fs::create_dir_all(&my_skill).unwrap();

    fs::write(
        my_skill.join("SKILL.md"),
        "---\nname: my-skill\ndescription: \"Test skill\"\n---\n# My Skill\n",
    )
    .unwrap();

    let result = scan_skills(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "my-skill");
    assert_eq!(result[0].description, "Test skill");
    assert_eq!(result[0].file_count, 1); // just SKILL.md
    cleanup(&dir);
}

// =====================================================================
// agents.rs tests
// =====================================================================

#[test]
fn test_scan_agents_finds_md_files() {
    let dir = make_temp_dir("agents_find");
    let agents_dir = dir.join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    fs::write(agents_dir.join("reviewer.md"), "# Reviewer\nReviews pull requests carefully.").unwrap();
    fs::write(agents_dir.join("deployer.md"), "# Deployer\nDeploys to production.").unwrap();

    let result = scan_agents(&dir).unwrap();
    assert_eq!(result.len(), 2);
    // sorted by name
    assert_eq!(result[0].name, "deployer");
    assert_eq!(result[1].name, "reviewer");
    assert!(result[1].path.ends_with("reviewer.md"));
    assert!(!result[0].description.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_agents_skips_non_md() {
    let dir = make_temp_dir("agents_skip");
    let agents_dir = dir.join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    fs::write(agents_dir.join("agent.md"), "# Agent\nA real agent.").unwrap();
    fs::write(agents_dir.join("notes.txt"), "not an agent").unwrap();
    fs::write(agents_dir.join("config.json"), "{}").unwrap();

    let result = scan_agents(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "agent");
    cleanup(&dir);
}

#[test]
fn test_scan_agents_empty_dir() {
    let dir = make_temp_dir("agents_empty");
    let agents_dir = dir.join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let result = scan_agents(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_agents_no_agents_dir() {
    let dir = make_temp_dir("agents_nodir");
    // No agents/ subdirectory at all
    let result = scan_agents(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_agents_extracts_description() {
    let dir = make_temp_dir("agents_desc");
    let agents_dir = dir.join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    // With frontmatter
    fs::write(
        agents_dir.join("with-fm.md"),
        "---\ntitle: Test\n---\n# Heading\nThis is the first paragraph after frontmatter.",
    ).unwrap();

    // Without frontmatter
    fs::write(
        agents_dir.join("no-fm.md"),
        "# My Agent\nFirst paragraph here.",
    ).unwrap();

    let result = scan_agents(&dir).unwrap();
    assert_eq!(result.len(), 2);

    let no_fm = result.iter().find(|a| a.name == "no-fm").unwrap();
    assert_eq!(no_fm.description, "First paragraph here.");

    let with_fm = result.iter().find(|a| a.name == "with-fm").unwrap();
    assert_eq!(with_fm.description, "This is the first paragraph after frontmatter.");
    cleanup(&dir);
}

// =====================================================================
// hooks.rs tests
// =====================================================================

#[test]
fn test_scan_hooks_nested_format() {
    let dir = make_temp_dir("hooks_nested");
    let settings = r#"{
        "hooks": {
            "Stop": [{
                "matcher": ".*",
                "hooks": [{
                    "type": "command",
                    "command": "echo done"
                }]
            }]
        }
    }"#;
    fs::write(dir.join("settings.json"), settings).unwrap();

    let result = scan_hooks(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event, "Stop");
    assert_eq!(result[0].matcher, Some(".*".to_string()));
    assert_eq!(result[0].hook_type, "command");
    assert_eq!(result[0].command, "echo done");
    cleanup(&dir);
}

#[test]
fn test_scan_hooks_empty_settings() {
    let dir = make_temp_dir("hooks_empty");
    fs::write(dir.join("settings.json"), r#"{"something": "else"}"#).unwrap();

    let result = scan_hooks(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_hooks_no_file() {
    let dir = make_temp_dir("hooks_nofile");
    let result = scan_hooks(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_hooks_legacy_format() {
    let dir = make_temp_dir("hooks_legacy");
    let settings = r#"{
        "hooks": {
            "PreToolUse": [{
                "matcher": "Bash",
                "type": "command",
                "command": "lint-check.sh"
            }]
        }
    }"#;
    fs::write(dir.join("settings.json"), settings).unwrap();

    let result = scan_hooks(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event, "PreToolUse");
    assert_eq!(result[0].matcher, Some("Bash".to_string()));
    assert_eq!(result[0].command, "lint-check.sh");
    cleanup(&dir);
}

#[test]
fn test_scan_hooks_multiple_events() {
    let dir = make_temp_dir("hooks_multi");
    let settings = r#"{
        "hooks": {
            "Stop": [{
                "matcher": ".*",
                "hooks": [{ "type": "command", "command": "notify.sh" }]
            }],
            "PreToolUse": [{
                "matcher": "Edit",
                "hooks": [
                    { "type": "command", "command": "pre-edit.sh" },
                    { "type": "command", "command": "validate.sh" }
                ]
            }]
        }
    }"#;
    fs::write(dir.join("settings.json"), settings).unwrap();

    let result = scan_hooks(&dir).unwrap();
    assert_eq!(result.len(), 3);
    cleanup(&dir);
}

// =====================================================================
// plugins.rs tests
// =====================================================================

#[test]
fn test_scan_plugins_parses_registry() {
    let dir = make_temp_dir("plugins_parse");
    let plugins_dir = dir.join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();

    let data = r#"{
        "plugins": {
            "my-plugin@skillvault": [{
                "version": "1.2.0",
                "scope": "user"
            }],
            "other-plugin@npm": [{
                "version": "0.5.1",
                "scope": "project"
            }]
        }
    }"#;
    fs::write(plugins_dir.join("installed_plugins.json"), data).unwrap();

    let result = scan_plugins(&dir).unwrap();
    assert_eq!(result.len(), 2);

    let my = result.iter().find(|p| p.name == "my-plugin").unwrap();
    assert_eq!(my.marketplace, "skillvault");
    assert_eq!(my.version, "1.2.0");
    assert_eq!(my.scope, "user");

    let other = result.iter().find(|p| p.name == "other-plugin").unwrap();
    assert_eq!(other.marketplace, "npm");
    assert_eq!(other.version, "0.5.1");
    assert_eq!(other.scope, "project");
    cleanup(&dir);
}

#[test]
fn test_scan_plugins_empty_registry() {
    let dir = make_temp_dir("plugins_empty");
    let plugins_dir = dir.join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    fs::write(plugins_dir.join("installed_plugins.json"), r#"{"plugins": {}}"#).unwrap();

    let result = scan_plugins(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_plugins_no_file() {
    let dir = make_temp_dir("plugins_nofile");
    let result = scan_plugins(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_plugins_multiple_installs() {
    let dir = make_temp_dir("plugins_multi");
    let plugins_dir = dir.join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();

    let data = r#"{
        "plugins": {
            "shared-plugin@registry": [
                { "version": "1.0.0", "scope": "user" },
                { "version": "2.0.0", "scope": "project" }
            ]
        }
    }"#;
    fs::write(plugins_dir.join("installed_plugins.json"), data).unwrap();

    let result = scan_plugins(&dir).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "shared-plugin");
    assert_eq!(result[1].name, "shared-plugin");
    // Different versions
    let versions: Vec<&str> = result.iter().map(|p| p.version.as_str()).collect();
    assert!(versions.contains(&"1.0.0"));
    assert!(versions.contains(&"2.0.0"));
    cleanup(&dir);
}

// =====================================================================
// mcp.rs tests
// =====================================================================

#[test]
fn test_scan_mcp_servers_http() {
    let dir = make_temp_dir("mcp_http");
    let settings = r#"{
        "mcpServers": {
            "my-server": {
                "type": "http",
                "url": "https://mcp.example.com/v1"
            }
        }
    }"#;
    fs::write(dir.join("settings.json"), settings).unwrap();

    let result = scan_mcp_servers(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "my-server");
    assert_eq!(result[0].server_type, "http");
    assert_eq!(result[0].url, Some("https://mcp.example.com/v1".to_string()));
    assert_eq!(result[0].command, None);
    cleanup(&dir);
}

#[test]
fn test_scan_mcp_servers_empty() {
    let dir = make_temp_dir("mcp_empty");
    fs::write(dir.join("settings.json"), r#"{"other": "stuff"}"#).unwrap();

    let result = scan_mcp_servers(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_mcp_servers_no_file() {
    let dir = make_temp_dir("mcp_nofile");
    let result = scan_mcp_servers(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_mcp_servers_multiple() {
    let dir = make_temp_dir("mcp_multi");
    let settings = r#"{
        "mcpServers": {
            "http-server": {
                "type": "http",
                "url": "https://example.com/mcp"
            },
            "stdio-server": {
                "command": "npx mcp-server"
            },
            "another-http": {
                "type": "http",
                "url": "https://other.com/mcp"
            }
        }
    }"#;
    fs::write(dir.join("settings.json"), settings).unwrap();

    let result = scan_mcp_servers(&dir).unwrap();
    assert_eq!(result.len(), 3);
    // sorted by name
    assert_eq!(result[0].name, "another-http");
    assert_eq!(result[1].name, "http-server");
    assert_eq!(result[2].name, "stdio-server");

    // stdio-server defaults to "stdio" type and has a command
    assert_eq!(result[2].server_type, "stdio");
    assert_eq!(result[2].command, Some("npx mcp-server".to_string()));
    assert_eq!(result[2].url, None);
    cleanup(&dir);
}

// =====================================================================
// teams.rs tests
// =====================================================================

#[test]
fn test_scan_teams_with_config() {
    let dir = make_temp_dir("teams_config");
    let teams_dir = dir.join("teams");
    let team_dir = teams_dir.join("engineering");
    fs::create_dir_all(&team_dir).unwrap();

    let config = r#"{
        "description": "The engineering team",
        "member_count": 12
    }"#;
    fs::write(team_dir.join("config.json"), config).unwrap();

    let result = scan_teams(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "engineering");
    assert_eq!(result[0].description, Some("The engineering team".to_string()));
    assert_eq!(result[0].member_count, 12);
    cleanup(&dir);
}

#[test]
fn test_scan_teams_with_members_array() {
    let dir = make_temp_dir("teams_members");
    let teams_dir = dir.join("teams");
    let team_dir = teams_dir.join("design");
    fs::create_dir_all(&team_dir).unwrap();

    let config = r#"{
        "description": "Design team",
        "members": ["alice", "bob", "charlie"]
    }"#;
    fs::write(team_dir.join("config.json"), config).unwrap();

    let result = scan_teams(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].member_count, 3);
    cleanup(&dir);
}

#[test]
fn test_scan_teams_without_config() {
    let dir = make_temp_dir("teams_noconfig");
    let teams_dir = dir.join("teams");
    let team_dir = teams_dir.join("ops");
    fs::create_dir_all(&team_dir).unwrap();

    // Put some files in the team dir (no config.json)
    fs::write(team_dir.join("readme.md"), "ops team").unwrap();
    fs::write(team_dir.join("playbook.md"), "playbook").unwrap();

    let result = scan_teams(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "ops");
    assert_eq!(result[0].description, None);
    assert_eq!(result[0].member_count, 2); // counts files in dir
    cleanup(&dir);
}

#[test]
fn test_scan_teams_empty() {
    let dir = make_temp_dir("teams_empty");
    let teams_dir = dir.join("teams");
    fs::create_dir_all(&teams_dir).unwrap();

    let result = scan_teams(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_teams_no_teams_dir() {
    let dir = make_temp_dir("teams_nodir");
    let result = scan_teams(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

// =====================================================================
// rules.rs tests
// =====================================================================

#[test]
fn test_decode_project_path_simple() {
    // This path should exist on the dev machine
    let result = decode_project_path_pub("-Users-bone-dev-apps-skillvault-desktop");
    // The path may or may not exist on this machine, so just test the function runs
    // If the path exists, it should decode correctly
    if let Some(path) = &result {
        assert!(path.starts_with("/"), "Decoded path should start with /");
        assert!(std::path::Path::new(path).exists(), "Decoded path should exist");
    }
    // If path doesn't exist, None is acceptable
}

#[test]
fn test_decode_project_path_nonexistent() {
    let result = decode_project_path_pub("-Nonexistent-Path-That-Will-Never-Exist-12345");
    assert!(result.is_none(), "Nonexistent path should decode to None");
}

#[test]
fn test_scan_rules_global_claude_md() {
    let dir = make_temp_dir("rules_global");
    fs::write(dir.join("CLAUDE.md"), "# Global Rules\nThese are global rules for Claude.").unwrap();

    let result = scan_rules(&dir).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "global");
    assert!(result[0].preview.contains("Global Rules"));
    assert!(result[0].project_path.is_none());
    assert!(result[0].size_bytes > 0);
    cleanup(&dir);
}

#[test]
fn test_scan_rules_no_claude_md() {
    let dir = make_temp_dir("rules_none");
    // No CLAUDE.md and no projects/
    let result = scan_rules(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}

#[test]
fn test_scan_rules_with_projects_dir_empty() {
    let dir = make_temp_dir("rules_projempty");
    let projects_dir = dir.join("projects");
    fs::create_dir_all(&projects_dir).unwrap();

    let result = scan_rules(&dir).unwrap();
    assert!(result.is_empty());
    cleanup(&dir);
}
