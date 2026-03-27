#[cfg(test)]
mod tests {
    use crate::scanner::codex::*;
    use std::fs;
    use std::path::Path;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("svd_codex_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_extract_toml_string_value() {
        assert_eq!(
            extract_toml_string_value("model = \"gpt-5.4\""),
            Some("gpt-5.4".to_string())
        );
        assert_eq!(
            extract_toml_string_value("model = 'gpt-5.4'"),
            Some("gpt-5.4".to_string())
        );
        assert_eq!(extract_toml_string_value("no_equals_here"), None);
        assert_eq!(extract_toml_string_value("empty = \"\""), None);
    }

    #[test]
    fn test_scan_codex_config_parses_model_and_projects() {
        let dir = temp_dir("config");
        let config = r#"
notify = ["/some/hook.sh"]
model = "gpt-5.4"

[projects."/Users/bone/dev/games/patina"]
trust_level = "trusted"

[projects."/Users/bone/dev/web-apps/skill-vault"]
trust_level = "trusted"
"#;
        fs::write(dir.join("config.toml"), config).unwrap();

        let result = scan_codex_config(&dir);
        assert!(result.is_some());
        let cfg = result.unwrap();
        assert_eq!(cfg.model, Some("gpt-5.4".to_string()));
        assert_eq!(cfg.trusted_projects.len(), 2);
        assert!(cfg.trusted_projects.iter().any(|p: &String| p.contains("patina")));
        assert!(cfg.trusted_projects.iter().any(|p: &String| p.contains("skill-vault")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_config_missing_file() {
        let dir = temp_dir("config_missing");
        let result = scan_codex_config(&dir);
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_rules() {
        let dir = temp_dir("rules");
        fs::write(dir.join("ubs.md"), "## UBS Quick Reference\nSome content here").unwrap();
        fs::write(dir.join(".hidden.md"), "hidden rule").unwrap();

        let rules = scan_codex_rules(&dir, None);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "ubs");
        assert!(rules[0].preview.contains("UBS Quick Reference"));
        assert!(rules[0].project.is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_rules_with_project() {
        let dir = temp_dir("rules_proj");
        fs::write(dir.join("test.md"), "test rule content").unwrap();

        let rules = scan_codex_rules(&dir, Some("patina"));
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].project, Some("patina".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_skills_directory_with_skill_md() {
        let dir = temp_dir("skills_dir");
        let skill_dir = dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\ndescription: A test skill\n---\nContent").unwrap();

        let skills = scan_codex_skills(&dir, None);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");
        // Path should point to SKILL.md inside the directory
        assert!(skills[0].path.ends_with("SKILL.md"), "Path should point to SKILL.md, got: {}", skills[0].path);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_skills_single_file() {
        let dir = temp_dir("skills_file");
        fs::write(dir.join("quick-skill.md"), "A simple skill").unwrap();

        let skills = scan_codex_skills(&dir, None);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "quick-skill");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_skills_empty() {
        let dir = temp_dir("skills_empty");
        let skills = scan_codex_skills(&dir, None);
        assert!(skills.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_agents_filters_artifacts() {
        let dir = temp_dir("agents");
        fs::create_dir_all(&dir).unwrap();
        // Real agent
        fs::write(dir.join("my-agent.md"), "agent definition").unwrap();
        // Artifacts that should be filtered
        fs::write(dir.join("AzureIsland.last-assignment"), "stale data").unwrap();
        fs::write(dir.join("debug.log"), "log data").unwrap();
        fs::write(dir.join("state.json"), "{}").unwrap();
        fs::write(dir.join("temp.tmp"), "temp").unwrap();
        // Hidden file
        fs::write(dir.join(".hidden"), "hidden").unwrap();

        let agents = scan_codex_agents(&dir, Some("patina"));
        assert_eq!(agents.len(), 1, "Should only find my-agent, got: {:?}", agents.iter().map(|a| &a.name).collect::<Vec<_>>());
        assert_eq!(agents[0].name, "my-agent");
        assert_eq!(agents[0].project, Some("patina".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_agents_empty_dir() {
        let dir = temp_dir("agents_empty");
        let agents = scan_codex_agents(&dir, None);
        assert!(agents.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_codex_agents_only_artifacts() {
        let dir = temp_dir("agents_only_artifacts");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("StormyLynx.last-assignment"), "data").unwrap();
        fs::write(dir.join("FrostyForest.last-assignment"), "data").unwrap();
        fs::write(dir.join("notices.json"), "{}").unwrap();

        let agents = scan_codex_agents(&dir, None);
        assert!(agents.is_empty(), "All artifacts should be filtered, got: {:?}", agents.iter().map(|a| &a.name).collect::<Vec<_>>());

        let _ = fs::remove_dir_all(&dir);
    }
}
