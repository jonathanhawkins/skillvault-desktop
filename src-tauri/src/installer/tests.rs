use super::{extract_zip, resolve_install_dir, resolve_skills_dir, uninstall_from, unwire_statusline_settings, wire_statusline_settings};
use std::fs;
use std::io::Write;

fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "skillvault_inst_test_{}_{}", prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = fs::remove_dir_all(dir);
}

fn create_zip_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
    let buf = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for (name, content) in files {
        writer.start_file(*name, options).unwrap();
        writer.write_all(content).unwrap();
    }

    writer.finish().unwrap().into_inner()
}

#[test]
fn test_extract_zip() {
    let dir = make_temp_dir("extract");
    let zip_data = create_zip_bytes(&[
        ("SKILL.md", b"# Hello\n"),
        ("scripts/run.sh", b"#!/bin/bash\necho hi\n"),
    ]);

    extract_zip(&zip_data, &dir).unwrap();

    assert!(dir.join("SKILL.md").exists());
    assert_eq!(fs::read_to_string(dir.join("SKILL.md")).unwrap(), "# Hello\n");
    assert!(dir.join("scripts/run.sh").exists());
    assert_eq!(
        fs::read_to_string(dir.join("scripts/run.sh")).unwrap(),
        "#!/bin/bash\necho hi\n"
    );
    cleanup(&dir);
}

#[test]
fn test_extract_zip_path_traversal() {
    let dir = make_temp_dir("traversal");
    let zip_data = create_zip_bytes(&[
        ("good.txt", b"safe"),
        ("../evil.txt", b"malicious"),
        ("sub/../sneaky.txt", b"tricky"),
    ]);

    extract_zip(&zip_data, &dir).unwrap();

    // good.txt should exist
    assert!(dir.join("good.txt").exists());
    // Path traversal entries should be skipped
    assert!(!dir.parent().unwrap().join("evil.txt").exists());
    // sub/../sneaky.txt contains ".." so it should be skipped
    assert!(!dir.join("sneaky.txt").exists());
    cleanup(&dir);
}

#[test]
fn test_extract_zip_absolute_path() {
    let dir = make_temp_dir("absolute");
    let zip_data = create_zip_bytes(&[
        ("good.txt", b"safe"),
        ("/etc/passwd", b"malicious"),
        ("/tmp/evil.txt", b"malicious"),
    ]);

    extract_zip(&zip_data, &dir).unwrap();

    // good.txt should exist
    assert!(dir.join("good.txt").exists());
    // Absolute path entries should be skipped entirely
    assert!(!dir.join("etc/passwd").exists());
    assert!(!dir.join("tmp/evil.txt").exists());
    cleanup(&dir);
}

#[test]
fn test_uninstall_moves_to_trash() {
    // uninstall() uses get_skills_dir() which resolves to ~/.claude/skills/
    // We need to create a real skill there, uninstall it, and check .trash/
    // To avoid touching the real home dir, we test the core logic manually.

    let dir = make_temp_dir("uninstall");
    let skills_dir = dir.join(".claude").join("skills");
    let skill_dir = skills_dir.join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "# Test").unwrap();

    // Simulate uninstall logic (same as the function, but with our custom path)
    let trash_dir = skills_dir.join(".trash");
    fs::create_dir_all(&trash_dir).unwrap();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let backup_name = format!("test-skill-{}", timestamp);
    let backup_path = trash_dir.join(&backup_name);

    fs::rename(&skill_dir, &backup_path).unwrap();

    // Original should be gone
    assert!(!skill_dir.exists());
    // Should be in trash
    assert!(backup_path.exists());
    assert!(backup_path.join("SKILL.md").exists());
    cleanup(&dir);
}

#[test]
fn test_resolve_skills_dir_global() {
    let result = resolve_skills_dir(None).unwrap();
    let home = dirs::home_dir().unwrap();
    assert_eq!(result, home.join(".claude").join("skills"));
}

#[test]
fn test_resolve_skills_dir_project() {
    let dir = make_temp_dir("resolve_proj");
    let result = resolve_skills_dir(Some(dir.to_str().unwrap())).unwrap();
    assert_eq!(result, dir.join(".claude").join("skills"));
    assert!(result.exists()); // resolve_skills_dir creates it
    cleanup(&dir);
}

// =====================================================================
// resolve_install_dir tests — items route to correct directories
// =====================================================================

#[test]
fn test_resolve_install_dir_skills() {
    let claude_dir = std::path::Path::new("/tmp/test-claude");
    let (dir, use_name) = resolve_install_dir(claude_dir, "skill");
    assert_eq!(dir, claude_dir.join("skills"));
    assert!(use_name, "Skills should use item name as subdirectory");
}

#[test]
fn test_resolve_install_dir_agents() {
    let claude_dir = std::path::Path::new("/tmp/test-claude");
    let (dir, use_name) = resolve_install_dir(claude_dir, "agent");
    assert_eq!(dir, claude_dir.join("agents"));
    assert!(use_name, "Agents should use item name as subdirectory");
}

#[test]
fn test_resolve_install_dir_teams() {
    let claude_dir = std::path::Path::new("/tmp/test-claude");
    let (dir, use_name) = resolve_install_dir(claude_dir, "team");
    assert_eq!(dir, claude_dir.join("teams"));
    assert!(use_name, "Teams should use item name as subdirectory");
}

#[test]
fn test_resolve_install_dir_rules() {
    let claude_dir = std::path::Path::new("/tmp/test-claude");
    let (dir, use_name) = resolve_install_dir(claude_dir, "rule");
    assert_eq!(dir, claude_dir.join("rules"));
    assert!(use_name, "Rules should use item name as subdirectory");
}

#[test]
fn test_resolve_install_dir_statusline_no_nesting() {
    let claude_dir = std::path::Path::new("/tmp/test-claude");
    let (dir, use_name) = resolve_install_dir(claude_dir, "statusline");
    assert_eq!(dir, claude_dir.join("statusline"));
    assert!(!use_name, "Statusline should NOT nest — files go directly into ~/.claude/statusline/");
}

// =====================================================================
// Packaging tests — verify directory contents are fully included
// =====================================================================

#[test]
fn test_extract_zip_directory_package_no_double_nesting() {
    // Simulates what the installer does: extract a zip with name-prefixed entries
    // into a temp dir, then verify the inner directory has all files
    let dir = make_temp_dir("no_double_nest");

    // Zip has statusline/ prefix on all entries (as package_skills creates)
    let zip_data = create_zip_bytes(&[
        ("statusline/statusline.sh", b"#!/bin/bash\necho hi\n"),
        ("statusline/debug.json", b"{\"debug\": true}"),
        ("statusline/nautilus-context.ts", b"// ts code\n"),
        ("statusline/README.md", b"# Statusline\n"),
    ]);

    // Extract to temp dir (like the manifest-based installer does)
    extract_zip(&zip_data, &dir).unwrap();

    // The inner "statusline/" directory should exist with ALL files
    let inner = dir.join("statusline");
    assert!(inner.is_dir(), "Inner statusline/ directory should exist");
    assert!(inner.join("statusline.sh").exists(), "statusline.sh should exist");
    assert!(inner.join("debug.json").exists(), "debug.json should exist");
    assert!(inner.join("nautilus-context.ts").exists(), "nautilus-context.ts should exist");
    assert!(inner.join("README.md").exists(), "README.md should exist");

    // Verify NO double nesting (no statusline/statusline/ directory)
    assert!(
        !inner.join("statusline").exists(),
        "Should NOT have double-nested statusline/statusline/ directory"
    );

    cleanup(&dir);
}

#[test]
fn test_statusline_install_moves_contents_not_directory() {
    // Simulates the manifest-based install for a statusline:
    // Files should go INTO ~/.claude/statusline/, not create ~/.claude/statusline/statusline/
    let dir = make_temp_dir("sl_install");
    let claude_dir = dir.join(".claude");
    let tmp_dir = dir.join("tmp");

    // Create the extracted zip content (statusline/ directory with files)
    let sl_src = tmp_dir.join("statusline");
    fs::create_dir_all(&sl_src).unwrap();
    fs::write(sl_src.join("statusline.sh"), "#!/bin/bash\necho hi").unwrap();
    fs::write(sl_src.join("debug.json"), "{}").unwrap();
    fs::write(sl_src.join("nautilus-context.ts"), "// code").unwrap();

    // Resolve install dir for statusline type
    let (dest_dir, use_name_subdir) = resolve_install_dir(&claude_dir, "statusline");
    assert!(!use_name_subdir, "Statusline should not use name subdir");
    fs::create_dir_all(&dest_dir).unwrap();

    // Simulate the installer: move contents INTO dest_dir (not the directory itself)
    if let Ok(entries) = fs::read_dir(&sl_src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dest_path = dest_dir.join(entry.file_name());
            fs::rename(&src_path, &dest_path).unwrap();
        }
    }

    // Verify files are directly in ~/.claude/statusline/, not nested
    assert!(dest_dir.join("statusline.sh").exists(), "statusline.sh should be in ~/.claude/statusline/");
    assert!(dest_dir.join("debug.json").exists(), "debug.json should be in ~/.claude/statusline/");
    assert!(dest_dir.join("nautilus-context.ts").exists(), "nautilus-context.ts should be in ~/.claude/statusline/");

    // Verify NO double nesting
    assert!(
        !dest_dir.join("statusline").exists(),
        "Should NOT have nested statusline/statusline/"
    );

    cleanup(&dir);
}

#[test]
fn test_skill_install_uses_name_subdir() {
    // Skills SHOULD create a subdirectory: ~/.claude/skills/<name>/
    let dir = make_temp_dir("skill_install");
    let claude_dir = dir.join(".claude");

    let (dest_dir, use_name_subdir) = resolve_install_dir(&claude_dir, "skill");
    assert!(use_name_subdir, "Skills should use name subdir");

    let item_dest = dest_dir.join("my-skill");
    fs::create_dir_all(&item_dest).unwrap();
    fs::write(item_dest.join("SKILL.md"), "# Test").unwrap();

    assert!(item_dest.join("SKILL.md").exists());
    assert_eq!(
        item_dest.to_string_lossy(),
        claude_dir.join("skills").join("my-skill").to_string_lossy()
    );

    cleanup(&dir);
}

#[test]
fn test_agent_install_uses_name_subdir() {
    let dir = make_temp_dir("agent_install");
    let claude_dir = dir.join(".claude");

    let (dest_dir, use_name_subdir) = resolve_install_dir(&claude_dir, "agent");
    assert!(use_name_subdir);
    assert_eq!(dest_dir, claude_dir.join("agents"));

    cleanup(&dir);
}

// =====================================================================
// wire_statusline_settings tests
// =====================================================================

#[test]
fn test_wire_statusline_creates_settings_entry() {
    let dir = make_temp_dir("wire_sl_new");
    let claude_dir = dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Create a statusline directory with a script
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash\necho hi").unwrap();

    // No settings.json exists yet
    assert!(!claude_dir.join("settings.json").exists());

    wire_statusline_settings(&claude_dir, &sl_dir);

    // settings.json should now exist with statusLine config
    let settings_path = claude_dir.join("settings.json");
    assert!(settings_path.exists(), "settings.json should be created");

    let content = fs::read_to_string(&settings_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(json["statusLine"]["type"], "command");
    let cmd = json["statusLine"]["command"].as_str().unwrap();
    assert!(cmd.starts_with("bash "), "Command should start with 'bash '");
    assert!(cmd.contains("statusline.sh"), "Command should reference statusline.sh");

    cleanup(&dir);
}

#[test]
fn test_wire_statusline_preserves_existing_settings() {
    let dir = make_temp_dir("wire_sl_existing");
    let claude_dir = dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Create existing settings.json with other config
    let existing = r#"{
  "permissions": { "allow": ["Bash(git *)"] },
  "model": "opus"
}"#;
    fs::write(claude_dir.join("settings.json"), existing).unwrap();

    // Create statusline
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash\necho hi").unwrap();

    wire_statusline_settings(&claude_dir, &sl_dir);

    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // statusLine should be added
    assert_eq!(json["statusLine"]["type"], "command");

    // Existing settings should be preserved
    assert_eq!(json["model"], "opus");
    assert!(json["permissions"]["allow"].is_array());

    cleanup(&dir);
}

#[test]
fn test_wire_statusline_includes_type_field() {
    // Regression test: settings.json must have "type": "command" for Claude Code to use it
    let dir = make_temp_dir("wire_sl_type");
    let claude_dir = dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash\necho hi").unwrap();

    wire_statusline_settings(&claude_dir, &sl_dir);

    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(
        json["statusLine"]["type"].is_string(),
        "statusLine must have 'type' field — Claude Code requires it"
    );
    assert_eq!(
        json["statusLine"]["type"], "command",
        "statusLine.type must be 'command'"
    );

    cleanup(&dir);
}

// =====================================================================
// unwire_statusline_settings tests
// =====================================================================

#[test]
fn test_unwire_removes_our_statusline_from_settings() {
    let dir = make_temp_dir("unwire_ours");
    let claude_dir = dir.join(".claude");
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash").unwrap();

    // Settings with our statusline
    let settings = format!(
        r#"{{"model":"opus","statusLine":{{"type":"command","command":"bash {}"}}}}"#,
        sl_dir.join("statusline.sh").to_string_lossy()
    );
    fs::write(claude_dir.join("settings.json"), &settings).unwrap();

    unwire_statusline_settings(&claude_dir, &sl_dir);

    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // statusLine should be removed
    assert!(json.get("statusLine").is_none(), "statusLine should be removed");
    // Other settings preserved
    assert_eq!(json["model"], "opus", "Other settings should be preserved");

    cleanup(&dir);
}

#[test]
fn test_unwire_preserves_custom_statusline() {
    let dir = make_temp_dir("unwire_custom");
    let claude_dir = dir.join(".claude");
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();

    // Settings with a DIFFERENT statusline (user's own, not ours)
    let settings = r#"{"statusLine":{"type":"command","command":"bash /some/other/statusline.sh"}}"#;
    fs::write(claude_dir.join("settings.json"), settings).unwrap();

    unwire_statusline_settings(&claude_dir, &sl_dir);

    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // statusLine should NOT be removed — it's someone else's
    assert!(
        json.get("statusLine").is_some(),
        "Should NOT remove a statusline that doesn't point to our directory"
    );

    cleanup(&dir);
}

// =====================================================================
// Type-aware uninstall tests
// =====================================================================

#[test]
fn test_uninstall_skill_moves_to_trash() {
    let dir = make_temp_dir("uninst_skill");
    let claude_dir = dir.join(".claude");
    let skills_dir = claude_dir.join("skills");
    let skill = skills_dir.join("my-skill");
    fs::create_dir_all(&skill).unwrap();
    fs::write(skill.join("SKILL.md"), "# Test").unwrap();

    assert!(skill.exists());
    uninstall_from("my-skill", &claude_dir).unwrap();
    assert!(!skill.exists(), "Skill should be removed");

    // Should be in trash
    let trash = skills_dir.join(".trash");
    assert!(trash.exists(), "Trash dir should exist");
    let trash_entries: Vec<_> = fs::read_dir(&trash).unwrap().flatten().collect();
    assert_eq!(trash_entries.len(), 1, "Should have one entry in trash");
    assert!(trash_entries[0].file_name().to_string_lossy().contains("skill-my-skill"));

    cleanup(&dir);
}

#[test]
fn test_uninstall_agent_moves_to_trash() {
    let dir = make_temp_dir("uninst_agent");
    let claude_dir = dir.join(".claude");
    let agents_dir = claude_dir.join("agents");
    let agent = agents_dir.join("reviewer");
    fs::create_dir_all(&agent).unwrap();
    fs::write(agent.join("reviewer.md"), "# Agent").unwrap();

    assert!(agent.exists());
    uninstall_from("reviewer", &claude_dir).unwrap();
    assert!(!agent.exists(), "Agent should be removed");

    let trash = claude_dir.join("skills").join(".trash");
    let entries: Vec<_> = fs::read_dir(&trash).unwrap().flatten().collect();
    assert!(entries[0].file_name().to_string_lossy().contains("agent-reviewer"));

    cleanup(&dir);
}

#[test]
fn test_uninstall_team_moves_to_trash() {
    let dir = make_temp_dir("uninst_team");
    let claude_dir = dir.join(".claude");
    let teams_dir = claude_dir.join("teams");
    let team = teams_dir.join("my-team");
    fs::create_dir_all(&team).unwrap();
    fs::write(team.join("config.json"), "{}").unwrap();

    uninstall_from("my-team", &claude_dir).unwrap();
    assert!(!team.exists(), "Team should be removed");

    let trash = claude_dir.join("skills").join(".trash");
    let entries: Vec<_> = fs::read_dir(&trash).unwrap().flatten().collect();
    assert!(entries[0].file_name().to_string_lossy().contains("team-my-team"));

    cleanup(&dir);
}

#[test]
fn test_uninstall_rule_moves_to_trash() {
    let dir = make_temp_dir("uninst_rule");
    let claude_dir = dir.join(".claude");
    let rules_dir = claude_dir.join("rules");
    let rule = rules_dir.join("testing");
    fs::create_dir_all(&rule).unwrap();
    fs::write(rule.join("testing.md"), "# Rules").unwrap();

    uninstall_from("testing", &claude_dir).unwrap();
    assert!(!rule.exists(), "Rule should be removed");

    cleanup(&dir);
}

#[test]
fn test_uninstall_statusline_with_meta_removes_and_unwires() {
    let dir = make_temp_dir("uninst_sl");
    let claude_dir = dir.join(".claude");
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash\necho hi").unwrap();

    // Must have .skillvault-meta.json to be uninstallable
    fs::write(sl_dir.join(".skillvault-meta.json"), r#"{"source":"skillvault","package_id":"test/sl","version":"1.0.0","installed_at":"now","auto_update":false}"#).unwrap();

    // Wire up settings.json
    let settings = format!(
        r#"{{"model":"opus","statusLine":{{"type":"command","command":"bash {}"}}}}"#,
        sl_dir.join("statusline.sh").to_string_lossy()
    );
    fs::write(claude_dir.join("settings.json"), &settings).unwrap();

    uninstall_from("statusline", &claude_dir).unwrap();

    // Directory should be gone
    assert!(!sl_dir.exists(), "Statusline dir should be removed");

    // settings.json should have statusLine removed
    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("statusLine").is_none(), "statusLine should be removed from settings");
    assert_eq!(json["model"], "opus", "Other settings preserved");

    cleanup(&dir);
}

#[test]
fn test_uninstall_statusline_without_meta_refuses() {
    let dir = make_temp_dir("uninst_sl_nope");
    let claude_dir = dir.join(".claude");
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash\necho hi").unwrap();
    // NO .skillvault-meta.json — user's own statusline

    let result = uninstall_from("statusline", &claude_dir);
    assert!(result.is_err(), "Should refuse to remove user's own statusline");
    assert!(result.unwrap_err().contains("not installed by SkillVault"));

    // Directory should still exist
    assert!(sl_dir.exists(), "User's statusline should NOT be removed");

    cleanup(&dir);
}

#[test]
fn test_uninstall_nonexistent_returns_error() {
    let dir = make_temp_dir("uninst_404");
    let claude_dir = dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let result = uninstall_from("does-not-exist", &claude_dir);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));

    cleanup(&dir);
}

#[test]
fn test_uninstall_statusline_preserves_custom_settings() {
    // If someone else's statusline is in settings.json, uninstalling ours
    // should NOT touch their config
    let dir = make_temp_dir("uninst_sl_custom");
    let claude_dir = dir.join(".claude");
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash").unwrap();
    fs::write(sl_dir.join(".skillvault-meta.json"), r#"{"source":"skillvault","package_id":"test/sl","version":"1.0.0","installed_at":"now","auto_update":false}"#).unwrap();

    // Settings point to a DIFFERENT statusline (user's custom one)
    let settings = r#"{"statusLine":{"type":"command","command":"bash /custom/statusline.sh"}}"#;
    fs::write(claude_dir.join("settings.json"), settings).unwrap();

    uninstall_from("statusline", &claude_dir).unwrap();

    // Directory removed (it's ours via meta)
    assert!(!sl_dir.exists());

    // But settings.json statusLine should be PRESERVED (it's someone else's script)
    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        json.get("statusLine").is_some(),
        "Should NOT remove custom statusline from settings"
    );

    cleanup(&dir);
}

// =====================================================================
// Critical fix regression tests
// =====================================================================

#[test]
fn test_wire_statusline_does_not_corrupt_invalid_json() {
    // If settings.json has invalid JSON, wire_statusline_settings must NOT
    // replace it with an empty object — that would lose all settings.
    let dir = make_temp_dir("wire_invalid");
    let claude_dir = dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash").unwrap();

    // Write invalid JSON (trailing comma)
    let invalid = r#"{"model": "opus", "permissions": {"allow": ["Bash(git *)"]},}"#;
    fs::write(claude_dir.join("settings.json"), invalid).unwrap();

    wire_statusline_settings(&claude_dir, &sl_dir);

    // The original invalid content should be PRESERVED (not replaced with {})
    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    assert!(
        content.contains("opus"),
        "Invalid settings.json should NOT be overwritten. Got: {}",
        content
    );
    // statusLine should NOT have been added (since we couldn't parse)
    assert!(
        !content.contains("statusLine"),
        "Should not inject statusLine into unparseable settings.json"
    );

    cleanup(&dir);
}

#[test]
fn test_unwire_uses_path_match_not_substring() {
    // unwire_statusline_settings must use proper path matching, not substring.
    // "/home/user/.claude/statusline" should NOT match "/home/user/.claude/statusline-backup"
    let dir = make_temp_dir("unwire_path");
    let claude_dir = dir.join(".claude");
    let sl_dir = claude_dir.join("statusline");
    fs::create_dir_all(&sl_dir).unwrap();
    fs::write(sl_dir.join("statusline.sh"), "#!/bin/bash").unwrap();

    // Settings point to a DIFFERENT directory that happens to contain "statusline" as a substring
    let other_dir = claude_dir.join("statusline-backup");
    fs::create_dir_all(&other_dir).unwrap();
    fs::write(other_dir.join("statusline.sh"), "#!/bin/bash").unwrap();

    let settings = format!(
        r#"{{"statusLine":{{"type":"command","command":"bash {}"}}}}"#,
        other_dir.join("statusline.sh").to_string_lossy()
    );
    fs::write(claude_dir.join("settings.json"), &settings).unwrap();

    // Unwire with the shorter path — should NOT match the longer path
    unwire_statusline_settings(&claude_dir, &sl_dir);

    let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        json.get("statusLine").is_some(),
        "Should NOT remove statusLine pointing to a different directory (statusline-backup)"
    );

    cleanup(&dir);
}
