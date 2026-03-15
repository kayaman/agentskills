use std::env;
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::console;
use crate::core::config::SKILL_FILENAME;

const TEMPLATE: &str = r#"---
name: {name}
description: Brief description of what this skill does and when to use it.
---

# {title}

## When to Use

Use this skill when...

## Instructions

1. Step one
2. Step two
3. Step three
"#;

pub fn run(name: Option<&str>, dir: &Path) -> Result<()> {
    let name = name
        .map(|n| n.to_string())
        .unwrap_or_else(|| {
            env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "my-skill".to_string())
        });

    let skill_dir = if dir == Path::new(".") {
        Path::new(&name).to_path_buf()
    } else {
        dir.join(&name)
    };
    fs::create_dir_all(&skill_dir)?;

    let skill_file = skill_dir.join(SKILL_FILENAME);
    if skill_file.exists() {
        anyhow::bail!(
            "{} already exists. Use a different name or directory.",
            skill_file.display()
        );
    }

    let title = name
        .replace('-', " ")
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let content = TEMPLATE
        .replace("{name}", &name)
        .replace("{title}", &title);
    fs::write(&skill_file, content)?;
    fs::create_dir_all(skill_dir.join("references"))?;

    console::success(&format!("Created {}", skill_file.display()));
    console::success(&format!(
        "Edit {} to add your skill instructions.",
        skill_file.display()
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_creates_skill_md() {
        let tmp = TempDir::new().unwrap();
        run(Some("my-skill"), tmp.path()).unwrap();
        assert!(tmp.path().join("my-skill/SKILL.md").exists());
    }

    #[test]
    fn test_creates_references_dir() {
        let tmp = TempDir::new().unwrap();
        run(Some("my-skill"), tmp.path()).unwrap();
        assert!(tmp.path().join("my-skill/references").is_dir());
    }

    #[test]
    fn test_skill_md_has_correct_name() {
        let tmp = TempDir::new().unwrap();
        run(Some("my-skill"), tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join("my-skill/SKILL.md")).unwrap();
        assert!(content.contains("name: my-skill"));
    }

    #[test]
    fn test_title_case_from_kebab() {
        let tmp = TempDir::new().unwrap();
        run(Some("react-testing-library"), tmp.path()).unwrap();
        let content =
            fs::read_to_string(tmp.path().join("react-testing-library/SKILL.md")).unwrap();
        assert!(content.contains("# React Testing Library"));
    }

    #[test]
    fn test_title_case_from_snake() {
        let tmp = TempDir::new().unwrap();
        run(Some("my_skill_name"), tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join("my_skill_name/SKILL.md")).unwrap();
        assert!(content.contains("# My Skill Name"));
    }

    #[test]
    fn test_fails_if_already_exists() {
        let tmp = TempDir::new().unwrap();
        run(Some("my-skill"), tmp.path()).unwrap();
        let result = run(Some("my-skill"), tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_custom_output_dir() {
        let tmp = TempDir::new().unwrap();
        let custom = tmp.path().join("output");
        run(Some("my-skill"), &custom).unwrap();
        assert!(custom.join("my-skill/SKILL.md").exists());
    }

    #[test]
    fn test_template_has_sections() {
        let tmp = TempDir::new().unwrap();
        run(Some("demo"), tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join("demo/SKILL.md")).unwrap();
        assert!(content.contains("## When to Use"));
        assert!(content.contains("## Instructions"));
    }
}
