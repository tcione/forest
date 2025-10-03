# forest

The CLI tool endorsed by your local Forest Preservation Club™ for managing [(git) worktrees](https://git-scm.com/docs/git-worktree).

**:warning: This is still beta software. Core functionality likely won't change, but the feature set is not yet complete and might require fundamental changes depending on what I discover. Please check the "roadmap" section to see what's planned for the future.**

## But really, what is this?

A CLI tool to make working with [git worktrees](https://git-scm.com/docs/git-worktree) easier by establishing a few conventions and abstracting away some git commands.

## Then, what are the conventions?

- Repositories are called `roots`
- Worktrees are called `trees`
- `roots` live under `{base_dir}/roots`
- `trees` live under `{base_dir}/trees`
- All trees exist under the `trees` folder, regardless of `root`
- `tree` folders are named after the following pattern: `{root}--{branch-name}`
- When creating the `tree` folder name, `root` and `branch-name` get normalized by replacing any character different from `[A-Za-z0-9\-_]` by `--`
- `forest tree create <root> <branch>` will always branch out from the latest version of `root`'s default branch. Meaning whenever `create` is invoked, `git pull origin <default-branch` is performed in the `root` directory

## Nice! How can I install this?

### Nix flake

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    forest.url = "github:tcione/forest";
  };

  outputs = { self, nixpkgs, forest }: {
    # Add to your system packages
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [ forest.packages.x86_64-linux.forest ];
        })
      ];
    };
  };
}
```

Or with Home Manager:
```nix
# flake.nix
{
  inputs = {
    # ...
    forest.url = "path:/home/tortoise/Projects/src/forest";
  };

  outputs = { self, nixpkgs, home-manager, darwin, ...}@inputs:
  let
    system = "x86_64-linux";
  in {
    # ...
    nixosConfigurations.{user} = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        {
          nixpkgs.overlays = [
            (
              final: prev: {
               forest = inputs.forest.packages.${system}.default;
             }
            )
          ];
        }
        # ...
        home-manager.nixosModules.home-manager {
          # ...
          home-manager.users.{user}.imports = [
            # ...
            inputs.forest.homeManagerModules.${system}.default
            ./home.nix
          ];
        }
      ];
    };
  };
}
```

```nix
# in home.nix
programs.forest = {
  enable = true;
  settings = {
    general = {
      baseDir = "${config.home.homeDirectory}/Projects";
      copy = [".env" ".envrc"];
      exec = [];
    };
  };
};
```

### Homebrew

```bash
brew install tcione/tap/forest
```

### Other

1. Download the latest binary for your system under [releases](https://github.com/tcione/forest/releases)
2. Place somewhere visible in your `PATH`

## Now how can I use it?

1. Clone a repo: `forest roots clone <repo address>`
2. Create a tree: `forest trees create <root/repo-name> <branch-name>`
3. Use git normally inside `tree` and `root` (just avoid leaving the default branch in `root`)

## What does future look like? (roadmap)
0.10.1 - Current version

- [x] Proper documentation
- [x] Test homebrew setup
- [ ] 0.11.0: "fogo" bash setup (command that takes user to tree or root)
- [ ] 0.12.0: Fuzzy selection in "path", "exec" and "create"
- [ ] 0.13.0: CLI completions
- [ ] 0.14.0: Allow user to create local repos via `forest roots`
- [ ] 0.15.0: Allow for local repo configs using `forest.toml` at the `root`'s folder
- [ ] 0.15.1: Prepare repo for 1.0.0 by tidying up codebase, ensure consistency and double-checking standards. Check if there's anything else to take care before 1.0.0
...
- [ ] 1.0.0: "Formal" release of 1.0.0, no features here

## Configuration

If you have no configuration set, one is created for you when using the tool for the first time.

The configuration file is stored in one of the following places:
- **Linux**: `~/.config/forest/config.toml`
- **macOS**: `~/Library/Application Support/forest/config.toml`

And it looks like this:
```config.toml
[general]
# Dir where /roots and /trees will be stored
base_dir = "~/Projects"

# Files to be copied from root to tree on creation
copy = ["./.envrc", ".env"]

# CLI commands to be called when creating a tree
exec = []

[roots.{repo-name}]
copy = [".envrc", ".env", "some-other-file"]
exec = [
    "npm install"
]
```

## CLI

### Main Command

```
$ forest -h
A CLI tool to make working with easier by establishing a few conventions and abstracting away some git commands.

Usage: forest <COMMAND>

Commands:
  roots  Manage git repositories in roots/
  trees  Manage worktrees in trees/
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Roots Commands

```
$ forest roots -h
Manage git repositories in roots/

Usage: forest roots <COMMAND>

Commands:
  clone  Clone git repository inside roots/
  list   List all roots
  path   Show full path to a specific root
  exec   Execute a command against a root. Similar to entering root dir and inputting <command>
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Trees Commands

```
$ forest trees -h
Manage worktrees in trees/

Usage: forest trees <COMMAND>

Commands:
  create  Create a worktree for the repo inside trees/
  list    List all worktrees
  path    Path to a worktree directory
  exec    Execute a command against a tree. Similar to entering tree dir and inputting <command>
  clean   Clean up worktrees interactively
  delete  Execute command in worktree directory
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## License

Forest is dual-licensed under either:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

This means you can choose either license when using this code. See the license files for details.
