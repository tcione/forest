# forest

A CLI tool to facilitate git worktrees usage.

## Next
- Refactor for types
- Extract git logic (esp. default branch)
- trees::exec
- trees::enter
- trees::clean
- Nice output

## General

- Easy approach, following convention over configuration
- Nice and cute interface (offers more obvious aliases for commands)

## Conventions
- Repositories exist in `roots/`
- Worktrees exist in `trees/`
- Worktree folders follow this pattern "{repository-name}--{branch-name}"
- When translated to folders, branch names might only contain "[A-Za-z0-9\-_]". Any character outside of these gets replaced by "--"
- Project configuration can happen at two places: 1. the global configuration file; 2. at the project level

## Configuration
```config.toml
[general]
base_dir = "~/Projects"
copy = ["./.envrc", ".env"]
exec = []

[roots.{repo-name]]
copy = [".envrc", ".env", "some-other-file"]
exec = [
    "npm install"
]
```

## Project configuration
```forest.toml
copy = [".envrc", ".env", "some-other-file"]
exec = [
    "npm install"
]
```

## CLI

`forest (plant|clone) <repository-address>`
- clones repository into the roots directory

`forest (grow|create) <repository-name> <branch-name>`
- creates a worktree in `trees/`
- copies defined files
- executes defined commands
- If there's no configuration for that repo, give a warning

`forest (check|list) [--root=<repository-shorthand>]`
- list all worktrees in the format "[<root>]  <branch name>"
- if `--root` is given, it filters by it

`forest (nurture|goto) [--root=<repository-shorthand>] [--cmd=<command, default: "cd">]` [<root:branch>]
- `--cmd` defines a command that's run again the full path for tree. Example where cmd is "cd": `cd <base-path>/trees/<worktree-folder>`
- `cd` is the default value for `--cmd`
- `<root:branch>` is optional, if not given, the command offers a list of worktrees
- if `--root` is given, the list is filtered by `<root>`

`forest (trim|clean) [--root=<repo>]`
- Lists all worktrees
- if `--root` is given, the list is filtered by `<root>`
- Allows user to select the ones they want to delete
- Deletes after asking confirmation

## Rust standards

- Swallowing errors using `let _ = ...` must be avoided. Only use with justification
- Propagate errors with `?` operator
- Testing:
  - Mock API responses for reliable testing
  - File I/O testing with temporary directories
  - Each module tests its own domain logic
