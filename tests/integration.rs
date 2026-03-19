//! Integration tests for agentskills CLI.
//!
//! Verifies install from repos with root-level skill layout (skills at repo root, not under skills/).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn make_skill_at_root(parent: &std::path::Path, name: &str) {
    let dir = parent.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("SKILL.md"),
        format!("---\nname: {name}\n---\n\n# {name}\n"),
    )
    .unwrap();
}

#[test]
fn install_from_local_repo_with_root_level_skills() {
    let repo = TempDir::new().unwrap();
    make_skill_at_root(repo.path(), "root-skill-a");
    make_skill_at_root(repo.path(), "root-skill-b");

    let cwd = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("agentskills").unwrap();
    cmd.current_dir(cwd.path())
        .arg("add")
        .arg(repo.path().display().to_string())
        .arg("--yes");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Installed 'root-skill-a'"))
        .stdout(predicate::str::contains("Installed 'root-skill-b'"));

    assert!(cwd.path().join(".agents/skills/root-skill-a/SKILL.md").exists());
    assert!(cwd.path().join(".agents/skills/root-skill-b/SKILL.md").exists());
}
