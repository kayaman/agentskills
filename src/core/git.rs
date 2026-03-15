use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Result};
use tempfile::TempDir;

use crate::console;
use crate::core::config::{SKILL_FILENAME, SKILL_SEARCH_DIRS};
use crate::models::SkillSource;

/// Clone a repo and extract the skill folder(s). Returns (path to skill dir, temp dir handle).
///
/// The caller must keep `TempDir` alive to prevent cleanup.
pub fn fetch_skill(source: &SkillSource) -> Result<(PathBuf, TempDir)> {
    let tmp_dir = TempDir::new()?;
    let repo_dir = tmp_dir.path().join("repo");

    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--single-branch",
            "--branch",
            &source.git_ref,
            &source.repo_url(),
            &repo_dir.display().to_string(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        console::error(&format!("git clone failed: {}", stderr.trim()));
        bail!(
            "Failed to clone {}: {}",
            source.repo_url(),
            stderr.trim()
        );
    }

    let result = locate_skill(&repo_dir, source, tmp_dir.path())?;
    Ok((result, tmp_dir))
}

fn locate_skill(repo_dir: &Path, source: &SkillSource, tmp_base: &Path) -> Result<PathBuf> {
    if !source.skill.is_empty() {
        for search_dir in SKILL_SEARCH_DIRS {
            let candidate = repo_dir.join(search_dir).join(&source.skill);
            if candidate.is_dir() && candidate.join(SKILL_FILENAME).exists() {
                let dest = tmp_base.join(&source.skill);
                copy_dir_recursive(&candidate, &dest)?;
                let _ = fs::remove_dir_all(repo_dir);
                return Ok(dest);
            }
        }
        console::error(&format!(
            "Skill '{}' not found in {}",
            source.skill,
            source.display_name()
        ));
        bail!("Skill '{}' not found in repo", source.skill);
    }

    let mut skills_found: Vec<PathBuf> = Vec::new();
    for search_dir in SKILL_SEARCH_DIRS {
        let search_path = repo_dir.join(search_dir);
        if !search_path.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join(SKILL_FILENAME).exists() {
                    skills_found.push(path);
                }
            }
        }
    }

    if skills_found.is_empty() {
        if repo_dir.join(SKILL_FILENAME).exists() {
            let skill_name = &source.repo;
            let dest = tmp_base.join(skill_name);
            copy_dir_recursive(repo_dir, &dest)?;
            // Remove .git from copied directory
            let _ = fs::remove_dir_all(dest.join(".git"));
            let _ = fs::remove_dir_all(repo_dir);
            return Ok(dest);
        }
        console::error(&format!("No skills found in {}", source.display_name()));
        bail!("No SKILL.md files found in repo");
    }

    skills_found.sort();
    let result_dir = tmp_base.join("_all_skills");
    fs::create_dir_all(&result_dir)?;
    for skill_dir in &skills_found {
        let name = skill_dir.file_name().unwrap();
        copy_dir_recursive(skill_dir, &result_dir.join(name))?;
    }
    let _ = fs::remove_dir_all(repo_dir);
    let names: Vec<_> = skills_found
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    console::info(&format!("Found {} skills: {}", names.len(), names.join(", ")));
    Ok(result_dir)
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
