// Copyright 2024 tcione
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod application;
mod trees;
mod roots;
mod config;

mod utils {
    pub mod path;
    pub mod git;
    pub mod exec;
    pub mod cli_ui;
}

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "forest")]
#[command(about = "Convention-over-configuration CLI tool to manager git worktrees", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage git repositories in roots/
    #[command(subcommand)]
    Roots(RootsCommands),

    /// Manage worktrees in trees/
    #[command(subcommand)]
    Trees(TreesCommands),
}

#[derive(Debug, Subcommand)]
enum RootsCommands {
    /// Clone git repository inside roots/
    #[command(arg_required_else_help = true)]
    Clone {
        /// The git repository you want to clone. Use the same address you'd use for "git clone"
        repository_address: String,
    },

    /// List all repositories
    List,

    /// Enter a repository directory
    #[command(arg_required_else_help = true)]
    Path {
        /// Repository name
        root: String,
    },

    /// Execute command in repository directory
    #[command(arg_required_else_help = true)]
    Exec {
        /// Repository name
        root: String,
        /// Command to execute
        command: String,
    },
}

#[derive(Debug, Subcommand)]
enum TreesCommands {
    /// Create a worktree for the repo inside trees/
    #[command(arg_required_else_help = true)]
    Create {
        /// The repository name (same as the name of the folder in your system)
        root: String,
        /// Name for your new branch. Follow normal git conventions
        new_branch_name: String,
    },

    /// List all worktrees
    List {
        /// Filter by root (repository name)
        #[arg(long)]
        root: Option<String>,
    },

    /// Path to a worktree directory
    #[command(arg_required_else_help = true)]
    Path {
        /// The repository name (same as the name of the folder in your system)
        root: String,
        /// Branch you'll enter
        tree: String,
    },

    /// Execute command in worktree directory
    #[command(arg_required_else_help = true)]
    Exec {
        /// The repository name (same as the name of the folder in your system)
        root: String,
        /// Branch you'll enter
        tree: String,
        /// Command to execute
        command: String,
    },

    /// Clean up worktrees interactively
    Clean {
        /// Filter by root
        #[arg(long)]
        root: Option<String>,
    },

    /// Execute command in worktree directory
    #[command(arg_required_else_help = true)]
    Delete {
        /// The repository name (same as the name of the folder in your system)
        root: String,
        /// Branch you'll enter
        tree: String,
    },
}

fn main() {
    let args = Cli::parse();
    let forest = application::Application::new();

    forest.setup();

    match args.command {
        Commands::Roots(roots_cmd) => match roots_cmd {
            RootsCommands::Clone { repository_address } => forest.roots_clone(repository_address),
            RootsCommands::Exec { root, command } => forest.roots_exec(root, command),
            RootsCommands::List => forest.roots_list(),
            RootsCommands::Path { root } => forest.roots_path(root),
        },
        Commands::Trees(trees_cmd) => match trees_cmd {
            TreesCommands::Clean { root } => forest.trees_clean(root),
            TreesCommands::Create { root, new_branch_name } => forest.trees_create(root, new_branch_name),
            TreesCommands::Delete { root, tree } => forest.trees_delete(root, tree),
            TreesCommands::Exec { root, tree, command } => forest.trees_exec(root, tree, command),
            TreesCommands::List { root } => forest.trees_list(root),
            TreesCommands::Path { tree, root } => forest.trees_path(root, tree),
        },
    }
}
