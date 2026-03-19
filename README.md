# agentskills

Agent skills package manager — install, manage, and discover AI agent skills from GitHub.

[![crates.io](https://img.shields.io/crates/v/agentskills.svg)](https://crates.io/crates/agentskills)
[![license](https://img.shields.io/crates/l/agentskills.svg)](LICENSE)
[![codecov](https://codecov.io/gh/kayaman/agentskills/graph/badge.svg)](https://codecov.io/gh/kayaman/agentskills)
[![CI](https://github.com/kayaman/agentskills/actions/workflows/ci.yml/badge.svg)](https://github.com/kayaman/agentskills/actions/workflows/ci.yml)

## Install

```bash
cargo install agentskills
```

## Skills

For a curated collection of ready-to-use skills, see [kayaman/skills](https://github.com/kayaman/skills).

## Commands

### `agentskills add <source>`

Install a skill from GitHub or a local path.

```bash
# Install a specific skill from a repo
agentskills add kayaman/skills@semver

# Install all skills from a repo
agentskills add kayaman/skills

# Install globally (to ~/.agents/skills/)
agentskills add kayaman/skills@semver --global

# Install from a local directory
agentskills add ./my-local-skill

# Skip confirmation
agentskills add kayaman/skills@semver --yes
```

### `agentskills list`

Show installed skills (project + global).

```bash
agentskills list           # Table output
agentskills list --json    # JSON output
agentskills list --global  # Global only
agentskills ls             # Alias
```

### `agentskills remove <name>`

Uninstall a skill and clean the lockfile.

```bash
agentskills remove semver
agentskills rm semver --global  # Remove from global
```

### `agentskills update [name]`

Check for upstream changes and re-fetch updated skills.

```bash
agentskills update          # Update all skills
agentskills update semver   # Update a specific skill
```

### `agentskills init [name]`

Scaffold a new `SKILL.md` template.

```bash
agentskills init my-skill
agentskills init my-skill --dir ./skills
```

### `agentskills find <query>`

Search GitHub for repos containing agent skills.

```bash
agentskills find "react testing"
agentskills find deploy --limit 5
```

## Skill format

A skill is a directory containing a `SKILL.md` file with YAML frontmatter:

```markdown
---
name: my-skill
description: What this skill does and when to use it.
---

# My Skill

Instructions for the AI agent...
```

Skills can be organized in a GitHub repo in either layout:

**Option A — under `skills/` subdirectory:**
```
my-repo/
└── skills/
    ├── skill-a/
    │   └── SKILL.md
    └── skill-b/
        ├── SKILL.md
        └── references/
            └── extra-context.md
```

**Option B — at repo root:**
```
my-repo/
├── skill-a/
│   └── SKILL.md
└── skill-b/
    ├── SKILL.md
    └── references/
        └── extra-context.md
```

Both layouts are supported. The installer searches `skills/` first, then the repo root.

## Lockfile

Installed skills are tracked in `.skill-lock.json` (v3 format). Skills installed
by compatible tools sharing this format are visible to each other.

## Supported agents

agentskills detects and works with: Cursor, Claude Code, Codex, GitHub Copilot,
Amp, Cline, Continue, and Kiro.

## Development

```bash
cargo build          # Build
cargo test           # Run tests
cargo run -- --help  # Run locally
```

## License

Apache-2.0
