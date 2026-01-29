mod cli;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
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
        /// Default branch name
        #[arg(short, long)]
        branch: Option<String>,
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
        /// Delete source worktree after successful merge
        #[arg(short, long)]
        delete: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { url, target, branch } => {
            let default_branch = match branch {
                Some(b) => b,
                None => prompt_default_branch()?,
            };

            let result = grove::init(url.as_deref(), target.as_deref(), &default_branch)?;
            println!("{}", cli::success("Initialized grove repository"));
            println!("  {} {}", cli::context("Path:"), result.repo_path.display());
            println!("  {} {}", cli::context("Default branch:"), cli::highlight(&result.default_branch));
            println!("  {} {}", cli::context("Worktree:"), result.default_tree_path.display());
            Ok(())
        }
        Commands::Create { branch } => {
            let repo = grove::GroveRepo::discover()?;
            let result = grove::create(&repo, &branch)?;
            println!("{}", cli::success("Created worktree"));
            println!("  {} {}", cli::context("Branch:"), cli::highlight(&result.branch));
            println!("  {} {}", cli::context("Path:"), result.worktree_path.display());
            Ok(())
        }
        Commands::Delete { branch } => {
            let repo = grove::GroveRepo::discover()?;
            grove::delete(&repo, &branch)?;
            println!("{}", cli::success(&format!("Deleted worktree: {}", branch)));
            Ok(())
        }
        Commands::List => {
            let repo = grove::GroveRepo::discover()?;
            let worktrees = grove::list(&repo)?;

            if worktrees.is_empty() {
                println!("{}", cli::warn("No worktrees found"));
            } else {
                for wt in worktrees {
                    println!("{}", cli::branch_with_path(&wt.branch, &wt.path, wt.is_default));
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
        Commands::Merge { source, target, delete } => {
            let repo = grove::GroveRepo::discover()?;
            let result = grove::merge(&repo, &source, &target)?;
            println!("{}", cli::success(&format!("Merged '{}' into '{}'", result.source, result.target)));
            println!("  {} {}", cli::context("Worktree:"), result.target_worktree.display());

            if delete {
                if result.source == repo.default_branch() {
                    println!("{}", cli::warn("Cannot delete the default branch worktree"));
                } else {
                    grove::delete(&repo, &result.source)?;
                    println!("{}", cli::success(&format!("Deleted source worktree: {}", result.source)));
                }
            }

            Ok(())
        }
    }
}

fn prompt_default_branch() -> Result<String> {
    print!("{} ", cli::highlight("Default branch name:"));
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let branch = input.trim();

    if branch.is_empty() {
        anyhow::bail!("Default branch name cannot be empty");
    }

    Ok(branch.to_string())
}
