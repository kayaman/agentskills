use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::console;
use crate::core::config::{SKILL_FILENAME, global_skills_dir, project_skills_dir};
use crate::core::hash::hash_directory;
use crate::core::lockfile::add_entry;
use crate::models::{SkillLockEntry, SkillSource};

/// Install skill(s) from a fetched directory.
///
/// If skill_dir contains multiple subdirectories with SKILL.md, installs all of them.
/// Returns list of installed skill names.
pub fn install_skill(
    skill_dir: &Path,
    source: &SkillSource,
    global_install: bool,
    cwd: Option<&PathBuf>,
) -> Result<Vec<String>> {
    let target_base = if global_install {
        global_skills_dir()
    } else {
        project_skills_dir(cwd)
    };
    fs::create_dir_all(&target_base)?;

    let mut installed: Vec<String> = Vec::new();

    if skill_dir.join(SKILL_FILENAME).exists() {
        install_single(skill_dir, &target_base, source)?;
        if let Some(name) = skill_dir.file_name() {
            installed.push(name.to_string_lossy().to_string());
        }
    } else {
        let mut entries: Vec<_> = fs::read_dir(skill_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_dir() && e.path().join(SKILL_FILENAME).exists()
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            install_single(&path, &target_base, source)?;
            installed.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    if installed.is_empty() {
        console::warning("No skills found to install.");
    }

    Ok(installed)
}

fn install_single(skill_dir: &Path, target_base: &Path, source: &SkillSource) -> Result<()> {
    let name = skill_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let dest = target_base.join(&name);

    if dest.exists() {
        console::info(&format!("Updating existing skill '{name}'..."));
        fs::remove_dir_all(&dest)?;
    } else {
        console::info(&format!("Installing skill '{name}'..."));
    }

    copy_dir_recursive(skill_dir, &dest)?;

    let folder_hash = hash_directory(&dest);
    let skill_path = format!("skills/{name}/{SKILL_FILENAME}");
    let entry = SkillLockEntry::create(source, &skill_path, &folder_hash);
    add_entry(target_base, &name, entry)?;

    let scope = if target_base
        .display()
        .to_string()
        .contains("home")
    {
        "global"
    } else {
        "project"
    };
    console::success(&format!("Installed '{name}' ({scope}) → {}", dest.display()));
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use crate::core::source_parser::parse_source;

    fn make_skill(parent: &Path, name: &str) -> PathBuf {
        let dir = parent.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {name}\n---\n\n# {name}\n"),
        )
        .unwrap();
        dir
    }

    #[test]
    fn test_install_single_skill() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let skill_dir = make_skill(src.path(), "my-skill");
        let source = parse_source("owner/repo@my-skill").unwrap();

        let installed =
            install_skill(&skill_dir, &source, false, Some(&dst.path().to_path_buf())).unwrap();

        assert_eq!(installed, vec!["my-skill"]);
        assert!(dst.path().join(".agents/skills/my-skill/SKILL.md").exists());
    }

    #[test]
    fn test_install_multiple_skills() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        make_skill(src.path(), "skill-a");
        make_skill(src.path(), "skill-b");
        let source = parse_source("owner/repo").unwrap();

        let mut installed =
            install_skill(src.path(), &source, false, Some(&dst.path().to_path_buf())).unwrap();
        installed.sort();

        assert_eq!(installed, vec!["skill-a", "skill-b"]);
        assert!(dst.path().join(".agents/skills/skill-a/SKILL.md").exists());
        assert!(dst.path().join(".agents/skills/skill-b/SKILL.md").exists());
    }

    #[test]
    fn test_install_empty_dir_returns_empty() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let source = parse_source("owner/repo").unwrap();

        let installed =
            install_skill(src.path(), &source, false, Some(&dst.path().to_path_buf())).unwrap();

        assert!(installed.is_empty());
    }

    #[test]
    fn test_reinstall_updates_content() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let skill_dir = make_skill(src.path(), "update-me");
        let source = parse_source("owner/repo@update-me").unwrap();
        let cwd = dst.path().to_path_buf();

        install_skill(&skill_dir, &source, false, Some(&cwd)).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: update-me\n---\n\n# Updated\n",
        )
        .unwrap();
        install_skill(&skill_dir, &source, false, Some(&cwd)).unwrap();

        let content = fs::read_to_string(
            dst.path().join(".agents/skills/update-me/SKILL.md"),
        )
        .unwrap();
        assert!(content.contains("Updated"));
    }

    #[test]
    fn test_lockfile_written_after_install() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let skill_dir = make_skill(src.path(), "locked");
        let source = parse_source("owner/repo@locked").unwrap();

        install_skill(&skill_dir, &source, false, Some(&dst.path().to_path_buf())).unwrap();

        let lockfile = dst.path().join(".agents/.skill-lock.json");
        assert!(lockfile.exists());
        let data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();
        assert!(data["skills"]["locked"].is_object());
    }
}
