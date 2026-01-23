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
            let repo = grove::GroveRepo::discover()?;
            let result = grove::create(&repo, &branch)?;
            println!("Created worktree: {}", result.worktree_path.display());
            println!("Branch: {}", result.branch);
            Ok(())
        }
        Commands::Delete { branch } => {
            let repo = grove::GroveRepo::discover()?;
            grove::delete(&repo, &branch)?;
            println!("Deleted worktree and branch: {}", branch);
            Ok(())
        }
        Commands::List => {
            let repo = grove::GroveRepo::discover()?;
            let worktrees = grove::list(&repo)?;

            if worktrees.is_empty() {
                println!("No worktrees found");
            } else {
                for wt in worktrees {
                    let marker = if wt.is_main { " *" } else { "" };
                    println!("{}{}\t{}", wt.branch, marker, wt.path.display());
                }
            }
            Ok(())
        }
        Commands::Switch { branch } => {
            let repo = grove::GroveRepo::discover()?;
            let path = grove::switch(&repo, &branch)?;
            // Print just the path for shell integration: cd $(grove switch branch)
            println!("{}", path.display());
            Ok(())
        }
        Commands::Merge { source, target } => {
            let repo = grove::GroveRepo::discover()?;
            let result = grove::merge(&repo, &source, &target)?;
            println!(
                "Merged '{}' into '{}' ({})",
                result.source,
                result.target,
                result.target_worktree.display()
            );
            Ok(())
        }
    }
}
