use std::collections::HashSet;

use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};

use crate::console;

pub fn run(query: &str, limit: usize) -> Result<()> {
    let search_query = format!("SKILL.md in:path {query}");
    let url = format!(
        "https://api.github.com/search/code?q={}&per_page={}",
        urlencoding(&search_query),
        limit.min(30)
    );

    console::info(&format!("Searching GitHub for skills matching '{query}'..."));

    let resp = ureq::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "skillz-cli")
        .call();

    let resp = match resp {
        Ok(r) => r,
        Err(e) => {
            anyhow::bail!("GitHub API request failed: {e}");
        }
    };

    let status = resp.status();
    if status == 403 {
        console::warning("GitHub API rate limit reached. Try again later or set GITHUB_TOKEN.");
        std::process::exit(1);
    }

    let body: String = resp.into_body().read_to_string()?;

    if status != 200 {
        let preview = if body.len() > 200 { &body[..200] } else { &body };
        anyhow::bail!("GitHub API returned {status}: {preview}");
    }

    let data: serde_json::Value = serde_json::from_str(&body)?;
    let items = data["items"].as_array();

    let items = match items {
        Some(items) if !items.is_empty() => items,
        _ => {
            console::warning(&format!("No skills found matching '{query}'."));
            return Ok(());
        }
    };

    let mut seen = HashSet::new();
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Install with", "Repository", "Path"]);

    let mut row_count = 0;

    for item in items.iter().take(limit) {
        let repo_full = item["repository"]["full_name"]
            .as_str()
            .unwrap_or_default();
        let path = item["path"].as_str().unwrap_or_default();

        let parts: Vec<&str> = path.split('/').collect();
        let mut skill_name = "";
        for (i, part) in parts.iter().enumerate() {
            if *part == "SKILL.md" && i > 0 {
                skill_name = parts[i - 1];
                break;
            }
        }

        if skill_name.is_empty() {
            continue;
        }
        let key = format!("{repo_full}@{skill_name}");
        if !seen.insert(key) {
            continue;
        }

        let install_cmd = format!("skillz add {repo_full}@{skill_name}");
        let dir_path = parts[..parts.len().saturating_sub(1)].join("/");
        table.add_row(vec![&install_cmd, repo_full, &dir_path]);
        row_count += 1;
    }

    if row_count == 0 {
        console::warning("Found results but couldn't extract skill names.");
        return Ok(());
    }

    println!("{table}");
    console::info("Install with: skillz add <source>");
    Ok(())
}

fn urlencoding(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
