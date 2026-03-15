use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Github,
    Local,
}

impl SourceType {
    pub fn as_str(&self) -> &str {
        match self {
            SourceType::Github => "github",
            SourceType::Local => "local",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillSource {
    pub owner: String,
    pub repo: String,
    pub skill: String,
    pub git_ref: String,
    pub source_type: SourceType,
    pub local_path: Option<PathBuf>,
}

impl SkillSource {
    pub fn display_name(&self) -> String {
        if self.source_type == SourceType::Local {
            return self
                .local_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
        }
        if !self.skill.is_empty() {
            format!("{}/{}@{}", self.owner, self.repo, self.skill)
        } else {
            format!("{}/{}", self.owner, self.repo)
        }
    }

    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.repo)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLockEntry {
    pub source: String,
    pub source_type: String,
    pub source_url: String,
    pub skill_path: String,
    pub skill_folder_hash: String,
    pub installed_at: String,
    pub updated_at: String,
}

impl SkillLockEntry {
    pub fn create(source: &SkillSource, skill_path: &str, folder_hash: &str) -> Self {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let src = if !source.owner.is_empty() {
            format!("{}/{}", source.owner, source.repo)
        } else {
            source
                .local_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        };
        let source_url = if source.source_type == SourceType::Github {
            source.repo_url()
        } else {
            String::new()
        };
        Self {
            source: src,
            source_type: source.source_type.as_str().to_string(),
            source_url,
            skill_path: skill_path.to_string(),
            skill_folder_hash: folder_hash.to_string(),
            installed_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLock {
    pub version: u32,
    pub skills: BTreeMap<String, SkillLockEntry>,
}

impl Default for SkillLock {
    fn default() -> Self {
        Self {
            version: 3,
            skills: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AgentInfo {
    pub name: String,
    pub skills_dir: String,
    pub marker: String,
}

impl AgentInfo {
    #[allow(dead_code)]
    pub fn display(&self) -> String {
        format!("{} ({})", self.name, self.skills_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_as_str() {
        assert_eq!(SourceType::Github.as_str(), "github");
        assert_eq!(SourceType::Local.as_str(), "local");
    }

    #[test]
    fn test_display_name_with_skill() {
        let s = SkillSource {
            owner: "kayaman".to_string(),
            repo: "skills".to_string(),
            skill: "semver".to_string(),
            git_ref: "main".to_string(),
            source_type: SourceType::Github,
            local_path: None,
        };
        assert_eq!(s.display_name(), "kayaman/skills@semver");
    }

    #[test]
    fn test_display_name_without_skill() {
        let s = SkillSource {
            owner: "kayaman".to_string(),
            repo: "skills".to_string(),
            skill: String::new(),
            git_ref: "main".to_string(),
            source_type: SourceType::Github,
            local_path: None,
        };
        assert_eq!(s.display_name(), "kayaman/skills");
    }

    #[test]
    fn test_display_name_local() {
        let s = SkillSource {
            owner: String::new(),
            repo: String::new(),
            skill: "my-skill".to_string(),
            git_ref: "main".to_string(),
            source_type: SourceType::Local,
            local_path: Some(PathBuf::from("/tmp/my-skill")),
        };
        assert_eq!(s.display_name(), "/tmp/my-skill");
    }

    #[test]
    fn test_repo_url() {
        let s = SkillSource {
            owner: "kayaman".to_string(),
            repo: "skills".to_string(),
            skill: String::new(),
            git_ref: "main".to_string(),
            source_type: SourceType::Github,
            local_path: None,
        };
        assert_eq!(s.repo_url(), "https://github.com/kayaman/skills.git");
    }

    #[test]
    fn test_skill_lock_entry_create_github() {
        let source = SkillSource {
            owner: "kayaman".to_string(),
            repo: "skills".to_string(),
            skill: "semver".to_string(),
            git_ref: "main".to_string(),
            source_type: SourceType::Github,
            local_path: None,
        };
        let entry = SkillLockEntry::create(&source, "skills/semver/SKILL.md", "abc123");
        assert_eq!(entry.source, "kayaman/skills");
        assert_eq!(entry.source_type, "github");
        assert_eq!(entry.source_url, "https://github.com/kayaman/skills.git");
        assert_eq!(entry.skill_path, "skills/semver/SKILL.md");
        assert_eq!(entry.skill_folder_hash, "abc123");
        assert!(!entry.installed_at.is_empty());
        assert!(!entry.updated_at.is_empty());
    }

    #[test]
    fn test_skill_lock_entry_create_local() {
        let source = SkillSource {
            owner: String::new(),
            repo: String::new(),
            skill: "my-skill".to_string(),
            git_ref: "main".to_string(),
            source_type: SourceType::Local,
            local_path: Some(PathBuf::from("/tmp/my-skill")),
        };
        let entry = SkillLockEntry::create(&source, "skills/my-skill/SKILL.md", "def456");
        assert_eq!(entry.source_type, "local");
        assert_eq!(entry.source_url, "");
        assert_eq!(entry.skill_folder_hash, "def456");
    }

    #[test]
    fn test_skill_lock_default() {
        let lock = SkillLock::default();
        assert_eq!(lock.version, 3);
        assert!(lock.skills.is_empty());
    }

    #[test]
    fn test_agent_info_display() {
        let agent = AgentInfo {
            name: "Claude Code".to_string(),
            skills_dir: ".claude/".to_string(),
            marker: ".claude".to_string(),
        };
        assert_eq!(agent.display(), "Claude Code (.claude/)");
    }
}
