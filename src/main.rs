mod application;

mod commands {
    pub mod roots {
        pub mod clone;
        pub mod list;
        pub mod enter;
        pub mod exec;
    }
    pub mod trees {
        pub mod create;
        pub mod list;
    }
}

mod utils {
    pub mod config;
    pub mod path;
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
    Enter {
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

    /// Enter a worktree directory
    #[command(arg_required_else_help = true)]
    Enter {
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
}

fn main() {
    let args = Cli::parse();
    let forest = application::Application::new();

    forest.setup();

    match args.command {
        Commands::Roots(roots_cmd) => match roots_cmd {
            RootsCommands::Clone { repository_address } => forest.roots_clone(repository_address),
            RootsCommands::List => { forest.roots_list() },
            RootsCommands::Enter { root } => { forest.roots_enter(root) },
            RootsCommands::Exec { root, command } => { forest.roots_exec(root, command) },
        },
        Commands::Trees(trees_cmd) => match trees_cmd {
            TreesCommands::Create { root, new_branch_name } => forest.trees_create(root, new_branch_name),
            TreesCommands::List { root } => { forest.trees_list(root) },
            #[allow(unused_variables)]
            TreesCommands::Enter { tree, root } => {},
            #[allow(unused_variables)]
            TreesCommands::Exec { tree, command, root } => {},
            #[allow(unused_variables)]
            TreesCommands::Clean { root } => {},
        },
    }
}
