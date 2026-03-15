use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};

use crate::console;
use crate::core::config::{global_skills_dir, project_skills_dir};
use crate::core::lockfile::read_lockfile;

pub fn run(global_only: bool, json_output: bool) -> Result<()> {
    let mut rows: Vec<serde_json::Value> = Vec::new();

    if !global_only {
        let project_dir = project_skills_dir(None);
        if project_dir.exists() {
            let lock = read_lockfile(&project_dir);
            for (name, entry) in &lock.skills {
                rows.push(serde_json::json!({
                    "name": name,
                    "source": entry.source,
                    "scope": "project",
                    "installed": if entry.installed_at.len() >= 10 {
                        &entry.installed_at[..10]
                    } else {
                        &entry.installed_at
                    },
                }));
            }
        }
    }

    let global_dir = global_skills_dir();
    if global_dir.exists() {
        let lock = read_lockfile(&global_dir);
        for (name, entry) in &lock.skills {
            rows.push(serde_json::json!({
                "name": name,
                "source": entry.source,
                "scope": "global",
                "installed": if entry.installed_at.len() >= 10 {
                    &entry.installed_at[..10]
                } else {
                    &entry.installed_at
                },
            }));
        }
    }

    if rows.is_empty() {
        console::warning("No skills installed.");
        return Ok(());
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Name", "Source", "Scope", "Installed"]);

    for row in &rows {
        table.add_row(vec![
            row["name"].as_str().unwrap_or(""),
            row["source"].as_str().unwrap_or(""),
            row["scope"].as_str().unwrap_or(""),
            row["installed"].as_str().unwrap_or(""),
        ]);
    }

    println!("{table}");
    console::info(&format!("{} skill(s) installed", rows.len()));
    Ok(())
}
