use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        /// Target directory (defaults to repo name for clone, current dir for convert)
        #[arg(short, long)]
        target: Option<PathBuf>,
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
        Commands::Init { url, target } => {
            let result = grove::init(url.as_deref(), target.as_deref())?;
            println!(
                "Initialized grove repository at: {}",
                result.repo_path.display()
            );
            println!("Main branch: {}", result.main_branch);
            println!("Main tree: {}", result.main_tree_path.display());
            Ok(())
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
