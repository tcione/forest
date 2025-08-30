# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Communication Guidelines
- Direct, concise communication without unnecessary preamble
- YOU MUST NOT give compliments to the user
- Focus on technical content and problem-solving
- Provide context for design decisions
- The user<>claude intectation must work in the following way:
  - Claude is the navigator, user is the pilot
  - Do not edit files unless explicitly asked to do so

## About the project

Forest is a CLI tool that facilitates git worktree management, following convention over configuration principles. The tool manages repositories in `roots/` directories and worktrees in `trees/` directories with a specific naming pattern: `{repository-name}--{branch-name}`.

## Development Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Run the development version
- `cargo test` - Run tests
- `cargo fmt` - Format code
- `cargo clippy` - Run linter

### Nix Development (if using Nix)
- `nix develop` - Enter development shell with Rust toolchain

## Architecture

### Core Concepts
- **Roots**: Base directories where git repositories are cloned (`roots/`)
- **Trees**: Worktree directories following pattern `{repo-name}--{branch-name}` (`trees/`)
- **Configuration**: Two-level config system (global `config.toml` and project-level `forest.toml`)

### CLI Commands Structure
The tool provides intuitive aliases for git worktree operations:
- `forest plant/clone` - Clone repositories
- `forest grow/create` - Create worktrees with file copying and command execution
- `forest check/list` - List worktrees with filtering
- `forest nurture/goto` - Navigate to worktrees
- `forest trim/clean` - Delete worktrees with confirmation

### Configuration System
- Global configuration in `config.toml` with `[general]` and `[roots.{repo-name}]` sections
- Project-level configuration in `forest.toml`
- Supports file copying patterns and post-creation command execution

## Code Standards

### Error Handling
- Avoid swallowing errors with `let _ = ...` - only use with justification
- Propagate errors using the `?` operator
- Provide meaningful error messages

### Testing Strategy
- Mock API responses for reliable testing
- Use temporary directories for file I/O testing
- Each module should test its own domain logic

### Branch Name Sanitization
- Branch names in folder names must only contain `[A-Za-z0-9\-_]`
- Other characters are replaced with `--`
