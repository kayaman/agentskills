use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

use crate::models::AgentInfo;

const AGENT_MARKERS: &[(&str, &str, &str)] = &[
    (".cursor", "Cursor", ".cursor/rules/"),
    (".claude", "Claude Code", ".claude/"),
    (".codex", "Codex", ".codex/"),
    (".github/copilot", "GitHub Copilot", ".github/copilot/"),
    (".amp", "Amp", ".amp/"),
    (".cline", "Cline", ".cline/"),
    (".continue", "Continue", ".continue/"),
    (".kiro", "Kiro", ".kiro/"),
];

pub fn detect_agents(cwd: Option<&PathBuf>) -> Vec<AgentInfo> {
    let current = cwd
        .cloned()
        .unwrap_or_else(|| env::current_dir().unwrap_or_default());
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default();

    let search_dirs = [current, home];
    let mut found = Vec::new();
    let mut seen = HashSet::new();

    for base in &search_dirs {
        for &(marker, name, skills_dir) in AGENT_MARKERS {
            if base.join(marker).exists() && seen.insert(name.to_string()) {
                found.push(AgentInfo {
                    name: name.to_string(),
                    skills_dir: skills_dir.to_string(),
                    marker: marker.to_string(),
                });
            }
        }
    }

    found
}

pub fn detect_primary_agent(cwd: Option<&PathBuf>) -> Option<AgentInfo> {
    detect_agents(cwd).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_dir_has_no_project_agents() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        let agents = detect_agents(Some(&cwd));
        let in_cwd: Vec<_> = agents
            .iter()
            .filter(|a| cwd.join(&a.marker).exists())
            .collect();
        assert!(in_cwd.is_empty());
    }

    #[test]
    fn test_detects_claude() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        fs::create_dir_all(cwd.join(".claude")).unwrap();
        let agents = detect_agents(Some(&cwd));
        assert!(agents.iter().any(|a| a.name == "Claude Code"));
    }

    #[test]
    fn test_detects_cursor() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        fs::create_dir_all(cwd.join(".cursor")).unwrap();
        let agents = detect_agents(Some(&cwd));
        assert!(agents.iter().any(|a| a.name == "Cursor"));
    }

    #[test]
    fn test_detects_multiple_agents() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        fs::create_dir_all(cwd.join(".claude")).unwrap();
        fs::create_dir_all(cwd.join(".cursor")).unwrap();
        let agents = detect_agents(Some(&cwd));
        assert!(agents.iter().any(|a| a.name == "Claude Code"));
        assert!(agents.iter().any(|a| a.name == "Cursor"));
    }

    #[test]
    fn test_no_duplicate_agents() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        fs::create_dir_all(cwd.join(".claude")).unwrap();
        let agents = detect_agents(Some(&cwd));
        let count = agents.iter().filter(|a| a.name == "Claude Code").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_primary_is_cursor_when_present() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        // .cursor appears first in AGENT_MARKERS
        fs::create_dir_all(cwd.join(".cursor")).unwrap();
        let primary = detect_primary_agent(Some(&cwd));
        assert!(primary.is_some());
        assert_eq!(primary.unwrap().name, "Cursor");
    }

    #[test]
    fn test_agent_info_fields() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        fs::create_dir_all(cwd.join(".claude")).unwrap();
        let agents = detect_agents(Some(&cwd));
        let claude = agents.iter().find(|a| a.name == "Claude Code").unwrap();
        assert_eq!(claude.marker, ".claude");
        assert_eq!(claude.skills_dir, ".claude/");
    }
}
