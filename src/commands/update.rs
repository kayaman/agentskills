use anyhow::Result;

use crate::console;
use crate::core::config::{global_skills_dir, project_skills_dir};
use crate::core::git::fetch_skill;
use crate::core::hash::hash_directory;
use crate::core::installer::install_skill;
use crate::core::lockfile::read_lockfile;
use crate::core::source_parser::parse_source;

pub fn run(name: Option<&str>, global_install: bool) -> Result<()> {
    let skills_dir = if global_install {
        global_skills_dir()
    } else {
        project_skills_dir(None)
    };
    let lock = read_lockfile(&skills_dir);

    if lock.skills.is_empty() {
        console::warning("No skills installed.");
        return Ok(());
    }

    if let Some(name) = name {
        if !lock.skills.contains_key(name) {
            anyhow::bail!("Skill '{name}' not found in lockfile.");
        }
    }

    let targets: Vec<_> = if let Some(name) = name {
        lock.skills
            .iter()
            .filter(|(k, _)| k.as_str() == name)
            .collect()
    } else {
        lock.skills.iter().collect()
    };

    let mut updated_count = 0;
    for (skill_name, entry) in targets {
        if entry.source_type != "github" {
            console::info(&format!("Skipping '{skill_name}' (local source)"));
            continue;
        }

        console::info(&format!("Checking '{skill_name}'..."));
        let source_str = format!("{}@{}", entry.source, skill_name);
        match parse_source(&source_str).and_then(|source| {
            let (tmp_path, _tmp_dir) = fetch_skill(&source)?;
            let new_hash = hash_directory(&tmp_path);

            if new_hash != entry.skill_folder_hash {
                console::info(&format!("'{skill_name}' has changes, updating..."));
                install_skill(&tmp_path, &source, global_install, None)?;
                Ok(true)
            } else {
                console::info(&format!("'{skill_name}' is up to date."));
                Ok(false)
            }
        }) {
            Ok(true) => updated_count += 1,
            Ok(false) => {}
            Err(e) => console::warning(&format!("Failed to update '{skill_name}': {e}")),
        }
    }

    if updated_count > 0 {
        console::success(&format!("Updated {updated_count} skill(s)."));
    } else {
        console::info("All skills are up to date.");
    }

    Ok(())
}
