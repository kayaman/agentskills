use anyhow::Result;
use dialoguer::Confirm;

use crate::console;
use crate::core::agents::detect_primary_agent;
use crate::core::git::fetch_skill;
use crate::core::installer::install_skill;
use crate::core::source_parser::parse_source;
use crate::models::SourceType;

pub fn run(source: &str, global_install: bool, yes: bool) -> Result<()> {
    let parsed = parse_source(source)?;

    if let Some(agent) = detect_primary_agent(None) {
        console::info(&format!("Detected agent: {}", agent.name));
    }

    let scope = if global_install {
        "globally (~/.agents/skills/)"
    } else {
        "to project (.agents/skills/)"
    };
    console::info(&format!("Installing {} {scope}", parsed.display_name()));

    if !yes {
        let confirm = Confirm::new()
            .with_prompt("Proceed?")
            .default(true)
            .interact()?;
        if !confirm {
            return Ok(());
        }
    }

    if parsed.source_type == SourceType::Local {
        let local_path = parsed.local_path.as_ref().unwrap();
        if !local_path.is_dir() {
            anyhow::bail!("Local path does not exist: {}", local_path.display());
        }
        let installed = install_skill(local_path, &parsed, global_install, None)?;
        if !installed.is_empty() {
            console::success(&format!(
                "Installed {} skill(s): {}",
                installed.len(),
                installed.join(", ")
            ));
        }
    } else {
        let (tmp_path, _tmp_dir) = fetch_skill(&parsed)?;
        let installed = install_skill(&tmp_path, &parsed, global_install, None)?;
        if !installed.is_empty() {
            console::success(&format!(
                "Installed {} skill(s): {}",
                installed.len(),
                installed.join(", ")
            ));
        }
        // _tmp_dir drops here, cleaning up automatically
    }

    Ok(())
}
