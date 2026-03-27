use super::skills::{count_files, parse_skill_description, scan_skills};
use std::fs;

fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "skillvault_test_{}_{}", prefix,
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
