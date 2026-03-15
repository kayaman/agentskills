use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;

use crate::models::{SkillSource, SourceType};

pub fn parse_source(raw: &str) -> Result<SkillSource> {
    let raw = raw.trim();

    if raw.starts_with("./") || raw.starts_with("../") || raw.starts_with('/') || Path::new(raw).is_dir()
    {
        let path = Path::new(raw).canonicalize().unwrap_or_else(|_| raw.into());
        let skill = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        return Ok(SkillSource {
            owner: String::new(),
            repo: String::new(),
            skill,
            git_ref: "main".to_string(),
            source_type: SourceType::Local,
            local_path: Some(path),
        });
    }

    let mut input = raw.to_string();
    let mut git_ref = "main".to_string();

    if let Some(pos) = input.rfind('#') {
        git_ref = input[pos + 1..].to_string();
        input = input[..pos].to_string();
    }

    let re = Regex::new(r"^(?:github:)?([^/@]+)/([^/@]+)(?:@(.+))?$")?;
    let caps = re.captures(&input);
    match caps {
        Some(caps) => {
            let owner = caps[1].to_string();
            let repo = caps[2].to_string();
            let skill = caps
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            Ok(SkillSource {
                owner,
                repo,
                skill,
                git_ref,
                source_type: SourceType::Github,
                local_path: None,
            })
        }
        None => {
            bail!(
                "Invalid source format: '{}'. Expected: owner/repo, owner/repo@skill, or a local path.",
                raw
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_repo_at_skill() {
        let result = parse_source("kayaman/skills@semver").unwrap();
        assert_eq!(result.owner, "kayaman");
        assert_eq!(result.repo, "skills");
        assert_eq!(result.skill, "semver");
        assert_eq!(result.git_ref, "main");
        assert!(matches!(result.source_type, SourceType::Github));
    }

    #[test]
    fn test_owner_repo_only() {
        let result = parse_source("vercel-labs/agent-skills").unwrap();
        assert_eq!(result.owner, "vercel-labs");
        assert_eq!(result.repo, "agent-skills");
        assert_eq!(result.skill, "");
        assert!(matches!(result.source_type, SourceType::Github));
    }

    #[test]
    fn test_owner_repo_at_skill_with_ref() {
        let result = parse_source("kayaman/skills@semver#v2").unwrap();
        assert_eq!(result.owner, "kayaman");
        assert_eq!(result.repo, "skills");
        assert_eq!(result.skill, "semver");
        assert_eq!(result.git_ref, "v2");
    }

    #[test]
    fn test_github_prefix() {
        let result = parse_source("github:kayaman/skills@semver").unwrap();
        assert_eq!(result.owner, "kayaman");
        assert_eq!(result.repo, "skills");
        assert_eq!(result.skill, "semver");
    }

    #[test]
    fn test_local_dot_slash() {
        let result = parse_source("./my-skill").unwrap();
        assert!(matches!(result.source_type, SourceType::Local));
    }

    #[test]
    fn test_local_absolute() {
        let result = parse_source("/tmp/my-skill").unwrap();
        assert!(matches!(result.source_type, SourceType::Local));
    }

    #[test]
    fn test_invalid_format() {
        let result = parse_source("not-a-valid-source");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid source format"));
    }

    #[test]
    fn test_whitespace_stripped() {
        let result = parse_source("  kayaman/skills@semver  ").unwrap();
        assert_eq!(result.owner, "kayaman");
        assert_eq!(result.skill, "semver");
    }

    #[test]
    fn test_display_name_full() {
        let result = parse_source("kayaman/skills@semver").unwrap();
        assert_eq!(result.display_name(), "kayaman/skills@semver");
    }

    #[test]
    fn test_display_name_repo_only() {
        let result = parse_source("kayaman/skills").unwrap();
        assert_eq!(result.display_name(), "kayaman/skills");
    }

    #[test]
    fn test_repo_url() {
        let result = parse_source("kayaman/skills@semver").unwrap();
        assert_eq!(result.repo_url(), "https://github.com/kayaman/skills.git");
    }
}
