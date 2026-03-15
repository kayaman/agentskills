use std::fs;

use anyhow::Result;

use crate::console;
use crate::core::config::{global_skills_dir, project_skills_dir};
use crate::core::lockfile::remove_entry;

pub fn run(name: &str, global_install: bool) -> Result<()> {
    let skills_dir = if global_install {
        global_skills_dir()
    } else {
        project_skills_dir(None)
    };
    let skill_path = skills_dir.join(name);

    if !skill_path.exists() {
        let scope = if global_install { "global" } else { "project" };
        anyhow::bail!("Skill '{name}' not found in {scope} scope.");
    }

    console::info(&format!("Removing skill '{name}'..."));
    fs::remove_dir_all(&skill_path)?;
    let removed = remove_entry(&skills_dir, name)?;

    if removed {
        console::success(&format!("Removed '{name}' and cleaned lockfile."));
    } else {
        console::warning(&format!(
            "Removed '{name}' directory but it was not in the lockfile."
        ));
    }

    Ok(())
}
