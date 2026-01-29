# PRD: Grove & Forest Split

## Overview

Split the Forest project into two binaries:
- **Grove**: Repository-scoped worktree manager (bare repo based, unopinionated)
- **Forest**: Global orchestrator that builds on Grove (opinionated, convention-driven)

---

## Grove

### Philosophy
- Works at single repository level
- Uses bare repos (worktrees are the only working copies)
- No assumptions about user workflows
- Configurable via `.grove.toml` in repo

### Commands

| Command | Description |
|---------|-------------|
| `grove init [url]` | Initialize bare repo (convert existing OR clone from url). Always creates main tree. |
| `grove create <branch>` | Create worktree branching from latest remote main. Also creates main tree if missing. |
| `grove delete <branch>` | Delete worktree AND the branch |
| `grove list` | List all worktrees |
| `grove switch <branch>` | Print path to worktree (for shell integration) |
| `grove merge <source> <target>` | Fetch latest, merge source into target. Use `.` for current branch/tree. |

### Merge Examples (Grove)
```bash
grove merge . main      # merge current branch into main
grove merge main .      # merge main into current branch (update branch)
grove merge feature-x main   # merge feature-x into main
```

### Shell Integration

Shell function (recommended):
```bash
# ~/.bashrc or ~/.zshrc
gw() { cd "$(grove switch "$1")"; }
```

### Configuration (`.grove.toml`)

```toml
[grove]
trees = "./trees"           # default, user can override
main_branch = "main"        # default "main", user sets if repo uses "master" or other

[grove.forest]              # Forest-specific fields (set by Forest)
managed = true
trees = "~/trees/my-project--"   # supersedes grove.trees when managed = true

[hooks]
post_create = ["cp .env.example .env"]

[copy]
paths = [                   # files AND directories
  ".env.local",
  "config/local/",
]
```

### Tree Naming (Grove)
- Sanitized branch names: `[A-Za-z0-9\-_]`, other chars replaced with `--`
- No repo prefix (just `feature-x/`, not `repo--feature-x/`)

### Bare Repo Structure
```
my-project/                  # bare repo (no working files)
├── .grove.toml
├── HEAD, objects/, refs/    # git internals
└── trees/                   # default location
    ├── main/
    ├── feature-x/
    └── bugfix-y/
```

---

## Forest

### Philosophy
- Global tool, works from anywhere
- Enforces conventions: `roots/` for repos, `trees/` for worktrees
- Builds on Grove (library dependency)
- Flat API with root as argument

### Commands

| Command | Description |
|---------|-------------|
| `forest add <url>` | Clone repo as bare into `roots/`, init Grove with `managed = true` |
| `forest list [root]` | List all roots and all trees. If root provided, filters trees to that root. |
| `forest create <root> <branch>` | Wraps `grove create` for specified root |
| `forest delete <root> [branch]` | If branch: delete tree+branch. If only root: delete root and all its trees. Requires confirmation. |
| `forest switch <root> <branch>` | Wraps `grove switch`, prints path |
| `forest merge <root> <source> <target>` | Wraps `grove merge`. Source and target are explicit (no `.` shorthand). |

### Merge Examples (Forest)
```bash
forest merge my-project feature-x main   # merge feature-x into main
forest merge my-project main feature-x   # merge main into feature-x
```

### Configuration (`~/.config/forest/config.toml`)

```toml
[general]
roots_dir = "~/roots"
trees_dir = "~/trees"

# Per-root overrides (optional)
[roots.my-project]
main_branch = "master"      # override if different from "main"
post_create = ["make setup"]
```

### Tree Naming (Forest)
- Pattern: `{repo}--{branch}` (e.g., `my-project--feature-x`)
- Sanitized branch names same as Grove

### Directory Structure (Forest-managed)
```
~/roots/
├── my-project/              # bare repo
│   └── .grove.toml          # managed=true, trees="~/trees/my-project--"
└── other-repo/

~/trees/
├── my-project--main/
├── my-project--feature-x/
└── other-repo--main/
```

### Managed by Forest Behavior
When `grove.forest.managed = true`:
- Grove still works, but `grove.forest.trees` path supersedes `grove.trees`
- User can run `grove list`, `grove switch`, etc. from the bare repo

---

## Crate Structure

```
forest/
├── Cargo.toml               # workspace
├── crates/
│   ├── grove/
│   │   ├── Cargo.toml       # [lib] + [[bin]]
│   │   └── src/
│   │       ├── lib.rs       # library exports
│   │       ├── main.rs      # CLI binary
│   │       ├── config.rs
│   │       ├── bare.rs      # bare repo operations
│   │       └── worktree.rs
│   └── forest/
│       ├── Cargo.toml       # depends on grove
│       └── src/
│           ├── main.rs
│           ├── config.rs
│           └── roots.rs
```

---

## Implementation Order

1. **Phase 1**: Grove standalone (new crate)
   - `init`, `create`, `delete`, `list`, `switch`, `merge`
   - `.grove.toml` config
   - Bare repo support

2. **Phase 2**: Refactor Forest to use Grove
   - Add Grove as dependency
   - Flatten API
   - Add `grove.forest` config fields
   - Migrate existing roots logic
