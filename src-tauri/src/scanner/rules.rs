use crate::state::Rule;
use std::fs;
use std::path::Path;

/// Find CLAUDE.md files in project directories and globally
pub fn scan_rules(claude_dir: &Path) -> Result<Vec<Rule>, String> {
    let mut rules = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // Check global CLAUDE.md
    let global_claude_md = claude_dir.join("CLAUDE.md");
    if global_claude_md.exists() {
        if let Some(rule) = read_rule("global", &global_claude_md, None) {
            seen_paths.insert(global_claude_md.to_string_lossy().to_string());
            rules.push(rule);
        }
    }

    // Check each project directory in ~/.claude/projects/*/
    // Use a smarter path decoder that handles hyphens in directory names
    let projects_dir = claude_dir.join("projects");
    if projects_dir.exists() {
        if let Ok(entries) = fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let encoded_name = entry.file_name().to_string_lossy().to_string();

                // Decode the encoded path by trying to find the actual directory
                if let Some(decoded) = decode_project_path(&encoded_name) {
                    let project_root = Path::new(&decoded);
                    let project_claude_md = project_root.join("CLAUDE.md");

                    if project_claude_md.exists() {
                        let key = project_claude_md.to_string_lossy().to_string();
                        if seen_paths.contains(&key) {
                            continue;
                        }
                        seen_paths.insert(key);

                        let display_name = project_root
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| encoded_name.clone());

                        if let Some(rule) = read_rule(
                            &display_name,
                            &project_claude_md,
                            Some(decoded),
                        ) {
                            rules.push(rule);
                        }
                    }
                }
            }
        }
    }

    rules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(rules)
}

/// Decode an encoded project directory name back to a real path.
/// The encoding replaces `/` with `-`, so `-Users-bone-dev-web-apps-skill-vault`
/// could be `/Users/bone/dev/web-apps/skill-vault` or `/Users/bone/dev/web/apps/...`.
/// We try all possible splits and return the first that exists on disk.
/// Public wrapper for use from commands.rs
pub fn decode_project_path_pub(encoded: &str) -> Option<String> {
    decode_project_path(encoded)
}

fn decode_project_path(encoded: &str) -> Option<String> {
    // Strip leading `-` → the segments
    let stripped = if encoded.starts_with('-') {
        &encoded[1..]
    } else {
        encoded
    };

    let segments: Vec<&str> = stripped.split('-').collect();

    // Try to reconstruct the path by greedily joining segments
    // and checking which combinations correspond to real directories
    if let Some(path) = reconstruct_path(&segments, 0, String::from("/")) {
        return Some(path);
    }

    // Fallback: simple replacement (works for paths without hyphens)
    let simple = format!("/{}", stripped.replace('-', "/"));
    if Path::new(&simple).exists() {
        return Some(simple);
    }

    None
}

/// Recursively try joining segments with `/` or `-` to find a valid path
fn reconstruct_path(segments: &[&str], idx: usize, current: String) -> Option<String> {
    if idx >= segments.len() {
        if Path::new(&current).exists() {
            return Some(current);
        }
        return None;
    }

    // Try adding the next segment with `/`
    let with_slash = if current == "/" {
        format!("/{}", segments[idx])
    } else {
        format!("{}/{}", current, segments[idx])
    };

    // Try consuming more segments joined with `-` (greedy)
    // Try longest match first for better results
    for end in (idx + 1..=segments.len()).rev() {
        let joined = segments[idx..end].join("-");
        let candidate = if current == "/" {
            format!("/{}", joined)
        } else {
            format!("{}/{}", current, joined)
        };

        if end == segments.len() {
            // This is the final segment group — check if path exists
            if Path::new(&candidate).exists() {
                return Some(candidate);
            }
        } else if Path::new(&candidate).is_dir() {
            // This is an intermediate directory — recurse
            if let Some(result) = reconstruct_path(segments, end, candidate) {
                return Some(result);
            }
        }
    }

    None
}

fn read_rule(name: &str, path: &Path, project_path: Option<String>) -> Option<Rule> {
    let metadata = fs::metadata(path).ok()?;
    let size_bytes = metadata.len();

    let content = fs::read_to_string(path).ok()?;
    let preview: String = content.chars().take(200).collect();

    Some(Rule {
        name: name.to_string(),
        path: path.to_string_lossy().to_string(),
        project_path,
        size_bytes,
        preview,
    })
}
