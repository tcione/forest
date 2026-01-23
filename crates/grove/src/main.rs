use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "grove")]
#[command(about = "A repository-scoped git worktree manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a bare repo (convert existing or clone from url)
    Init {
        /// URL to clone from (if not provided, converts current repo to bare)
        url: Option<String>,
    },
    /// Create a new worktree from the latest remote main branch
    Create {
        /// Branch name for the new worktree
        branch: String,
    },
    /// Delete a worktree and its branch
    Delete {
        /// Branch name of the worktree to delete
        branch: String,
    },
    /// List all worktrees
    List,
    /// Print the path to a worktree (for shell integration)
    Switch {
        /// Branch name of the worktree
        branch: String,
    },
    /// Merge one branch into another
    Merge {
        /// Source branch (use "." for current branch)
        source: String,
        /// Target branch (use "." for current branch)
        target: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { url } => {
            println!("grove init: {:?}", url);
            todo!("Implement init")
        }
        Commands::Create { branch } => {
            println!("grove create: {}", branch);
            todo!("Implement create")
        }
        Commands::Delete { branch } => {
            println!("grove delete: {}", branch);
            todo!("Implement delete")
        }
        Commands::List => {
            println!("grove list");
            todo!("Implement list")
        }
        Commands::Switch { branch } => {
            println!("grove switch: {}", branch);
            todo!("Implement switch")
        }
        Commands::Merge { source, target } => {
            println!("grove merge: {} -> {}", source, target);
            todo!("Implement merge")
        }
    }
}
