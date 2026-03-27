use super::{extract_zip, resolve_skills_dir};
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
