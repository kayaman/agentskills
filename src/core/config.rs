use std::env;
use std::path::PathBuf;

pub const LOCKFILE_NAME: &str = ".skill-lock.json";
pub const SKILL_FILENAME: &str = "SKILL.md";
pub const SKILLS_DIR_NAME: &str = "skills";
pub const AGENTS_DIR: &str = ".agents";
pub const SKILL_SEARCH_DIRS: &[&str] = &["skills", "."];

pub fn global_skills_dir() -> PathBuf {
    home_dir().join(AGENTS_DIR).join(SKILLS_DIR_NAME)
}

pub fn project_skills_dir(cwd: Option<&PathBuf>) -> PathBuf {
    let base = cwd
        .cloned()
        .unwrap_or_else(|| env::current_dir().expect("failed to get current directory"));
    base.join(AGENTS_DIR).join(SKILLS_DIR_NAME)
}

fn home_dir() -> PathBuf {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(PathBuf::from)
        .expect("could not determine home directory")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_skills_dir_structure() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        let dir = project_skills_dir(Some(&cwd));
        assert!(dir.ends_with(".agents/skills"));
        assert!(dir.starts_with(tmp.path()));
    }

    #[test]
    fn test_global_skills_dir_structure() {
        let dir = global_skills_dir();
        assert!(dir.ends_with(".agents/skills"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(LOCKFILE_NAME, ".skill-lock.json");
        assert_eq!(SKILL_FILENAME, "SKILL.md");
        assert_eq!(SKILLS_DIR_NAME, "skills");
        assert_eq!(AGENTS_DIR, ".agents");
        assert!(SKILL_SEARCH_DIRS.contains(&"skills"));
    }
}
