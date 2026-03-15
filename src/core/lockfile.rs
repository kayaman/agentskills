use std::fs;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;

use crate::core::config::LOCKFILE_NAME;
use crate::models::{SkillLock, SkillLockEntry};

fn lockfile_path(base_dir: &Path) -> std::path::PathBuf {
    let parent = if base_dir.file_name().is_some_and(|n| n == "skills") {
        base_dir.parent().unwrap_or(base_dir)
    } else {
        base_dir
    };
    parent.join(LOCKFILE_NAME)
}

pub fn read_lockfile(base_dir: &Path) -> SkillLock {
    let path = lockfile_path(base_dir);
    if !path.exists() {
        return SkillLock::default();
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return SkillLock::default(),
    };
    match serde_json::from_str(&content) {
        Ok(lock) => lock,
        Err(_) => SkillLock::default(),
    }
}

pub fn write_lockfile(base_dir: &Path, lock: &SkillLock) -> Result<()> {
    let path = lockfile_path(base_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(lock)?;
    fs::write(&path, format!("{json}\n"))?;
    Ok(())
}

pub fn add_entry(base_dir: &Path, name: &str, mut entry: SkillLockEntry) -> Result<()> {
    let mut lock = read_lockfile(base_dir);
    if let Some(existing) = lock.skills.get(name) {
        entry.installed_at = existing.installed_at.clone();
        entry.updated_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    }
    lock.skills.insert(name.to_string(), entry);
    write_lockfile(base_dir, &lock)
}

pub fn remove_entry(base_dir: &Path, name: &str) -> Result<bool> {
    let mut lock = read_lockfile(base_dir);
    if lock.skills.remove(name).is_none() {
        return Ok(false);
    }
    write_lockfile(base_dir, &lock)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_missing_file_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let lock = read_lockfile(&tmp.path().join("skills"));
        assert_eq!(lock.version, 3);
        assert!(lock.skills.is_empty());
    }

    #[test]
    fn test_reads_existing() {
        let tmp = TempDir::new().unwrap();
        let lockfile = tmp.path().join(".skill-lock.json");
        let data = serde_json::json!({
            "version": 3,
            "skills": {
                "semver": {
                    "source": "kayaman/skills",
                    "sourceType": "github",
                    "sourceUrl": "https://github.com/kayaman/skills.git",
                    "skillPath": "skills/semver/SKILL.md",
                    "skillFolderHash": "abc123",
                    "installedAt": "2026-01-01T00:00:00.000Z",
                    "updatedAt": "2026-01-01T00:00:00.000Z"
                }
            }
        });
        fs::write(&lockfile, serde_json::to_string(&data).unwrap()).unwrap();

        let lock = read_lockfile(&tmp.path().join("skills"));
        assert!(lock.skills.contains_key("semver"));
        assert_eq!(lock.skills["semver"].source, "kayaman/skills");
        assert_eq!(lock.skills["semver"].skill_folder_hash, "abc123");
    }

    #[test]
    fn test_creates_file() {
        let tmp = TempDir::new().unwrap();
        let mut lock = SkillLock::default();
        lock.skills.insert(
            "test".to_string(),
            SkillLockEntry {
                source: "test/repo".to_string(),
                source_type: "github".to_string(),
                source_url: "https://github.com/test/repo.git".to_string(),
                skill_path: "skills/test/SKILL.md".to_string(),
                skill_folder_hash: "def456".to_string(),
                installed_at: "2026-01-01T00:00:00.000Z".to_string(),
                updated_at: "2026-01-01T00:00:00.000Z".to_string(),
            },
        );
        write_lockfile(&tmp.path().join("skills"), &lock).unwrap();

        let lockfile = tmp.path().join(".skill-lock.json");
        assert!(lockfile.exists());
        let data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();
        assert_eq!(data["version"], 3);
        assert_eq!(data["skills"]["test"]["source"], "test/repo");
        assert_eq!(data["skills"]["test"]["skillFolderHash"], "def456");
    }

    #[test]
    fn test_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mut lock = SkillLock::default();
        lock.skills.insert(
            "roundtrip".to_string(),
            SkillLockEntry {
                source: "a/b".to_string(),
                source_type: "github".to_string(),
                source_url: "https://github.com/a/b.git".to_string(),
                skill_path: "skills/roundtrip/SKILL.md".to_string(),
                skill_folder_hash: "xyz".to_string(),
                installed_at: "2026-02-01T00:00:00.000Z".to_string(),
                updated_at: "2026-02-01T00:00:00.000Z".to_string(),
            },
        );
        write_lockfile(&tmp.path().join("skills"), &lock).unwrap();
        let loaded = read_lockfile(&tmp.path().join("skills"));
        assert_eq!(loaded.skills["roundtrip"].source, "a/b");
        assert_eq!(loaded.skills["roundtrip"].skill_folder_hash, "xyz");
    }

    #[test]
    fn test_adds_new_entry() {
        let tmp = TempDir::new().unwrap();
        let entry = SkillLockEntry {
            source: "test/repo".to_string(),
            source_type: "github".to_string(),
            source_url: String::new(),
            skill_path: "skills/new/SKILL.md".to_string(),
            skill_folder_hash: "hash1".to_string(),
            installed_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        };
        add_entry(&tmp.path().join("skills"), "new", entry).unwrap();
        let lock = read_lockfile(&tmp.path().join("skills"));
        assert!(lock.skills.contains_key("new"));
    }

    #[test]
    fn test_updates_existing_preserves_installed_at() {
        let tmp = TempDir::new().unwrap();
        let first = SkillLockEntry {
            source: "test/repo".to_string(),
            source_type: "github".to_string(),
            source_url: String::new(),
            skill_path: "skills/existing/SKILL.md".to_string(),
            skill_folder_hash: "hash1".to_string(),
            installed_at: "2025-01-01T00:00:00.000Z".to_string(),
            updated_at: "2025-01-01T00:00:00.000Z".to_string(),
        };
        add_entry(&tmp.path().join("skills"), "existing", first).unwrap();

        let second = SkillLockEntry {
            source: "test/repo".to_string(),
            source_type: "github".to_string(),
            source_url: String::new(),
            skill_path: "skills/existing/SKILL.md".to_string(),
            skill_folder_hash: "hash2".to_string(),
            installed_at: "2026-06-01T00:00:00.000Z".to_string(),
            updated_at: "2026-06-01T00:00:00.000Z".to_string(),
        };
        add_entry(&tmp.path().join("skills"), "existing", second).unwrap();

        let lock = read_lockfile(&tmp.path().join("skills"));
        assert_eq!(
            lock.skills["existing"].installed_at,
            "2025-01-01T00:00:00.000Z"
        );
        assert_eq!(lock.skills["existing"].skill_folder_hash, "hash2");
    }

    #[test]
    fn test_removes_existing() {
        let tmp = TempDir::new().unwrap();
        let entry = SkillLockEntry {
            source: "test/repo".to_string(),
            source_type: "github".to_string(),
            source_url: String::new(),
            skill_path: "skills/gone/SKILL.md".to_string(),
            skill_folder_hash: "hash1".to_string(),
            installed_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        };
        add_entry(&tmp.path().join("skills"), "gone", entry).unwrap();
        assert!(remove_entry(&tmp.path().join("skills"), "gone").unwrap());
        let lock = read_lockfile(&tmp.path().join("skills"));
        assert!(!lock.skills.contains_key("gone"));
    }

    #[test]
    fn test_returns_false_for_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(!remove_entry(&tmp.path().join("skills"), "nope").unwrap());
    }
}
