mod config;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::{Config, load_config};
use grove::GroveRepo;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "forest")]
#[command(about = "A CLI tool for managing multiple git repositories with worktrees")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new repository (clones as bare repo with grove)
    #[command(arg_required_else_help = true)]
    Add {
        /// Git repository URL to clone
        url: String,
    },

    /// List roots and trees
    List {
        /// Filter by root name (shows only trees for that root)
        root: Option<String>,
    },

    /// Create a new worktree
    #[command(arg_required_else_help = true)]
    Create {
        /// Root name (repository)
        root: String,
        /// Branch name for the new worktree
        branch: String,
    },

    /// Delete a root or tree (requires confirmation)
    #[command(arg_required_else_help = true)]
    Delete {
        /// Root name (repository)
        root: String,
        /// Branch name (if omitted, deletes the entire root)
        branch: Option<String>,
    },

    /// Print path to a worktree (for shell integration)
    #[command(arg_required_else_help = true)]
    Switch {
        /// Root name (repository)
        root: String,
        /// Branch name
        branch: String,
    },

    /// Merge one branch into another
    #[command(arg_required_else_help = true)]
    Merge {
        /// Root name (repository)
        root: String,
        /// Source branch
        source: String,
        /// Target branch
        target: String,
    },
}

struct Forest {
    roots_dir: PathBuf,
    trees_dir: PathBuf,
    config: Config,
}

impl Forest {
    fn new() -> Result<Self> {
        let config_dir = config::config_dir()?;
        let config = load_config(&config_dir)?;

        let roots_dir = PathBuf::from(&config.general.roots_dir);
        let trees_dir = PathBuf::from(&config.general.trees_dir);

        // Ensure directories exist
        std::fs::create_dir_all(&roots_dir)?;
        std::fs::create_dir_all(&trees_dir)?;

        Ok(Self {
            roots_dir,
            trees_dir,
            config,
        })
    }

    /// Get path to a root's bare repo
    fn root_path(&self, root: &str) -> PathBuf {
        self.roots_dir.join(root)
    }

    /// Get the tree directory prefix for a root (e.g., ~/trees/my-repo--)
    fn tree_prefix(&self, root: &str) -> String {
        self.trees_dir
            .join(format!("{}--", root))
            .to_string_lossy()
            .to_string()
    }

    /// Open a grove repo for a specific root
    fn open_grove(&self, root: &str) -> Result<GroveRepo> {
        let root_path = self.root_path(root);
        if !root_path.exists() {
            anyhow::bail!("Root '{}' not found at {}", root, root_path.display());
        }
        GroveRepo::open(&root_path).with_context(|| format!("Failed to open root '{}'", root))
    }

    /// List all roots
    fn list_roots(&self) -> Result<Vec<String>> {
        let mut roots = Vec::new();
        if self.roots_dir.exists() {
            for entry in std::fs::read_dir(&self.roots_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    roots.push(name.to_string());
                }
            }
        }
        roots.sort();
        Ok(roots)
    }

    /// Get per-root config (copy/exec settings)
    fn root_config(&self, root: &str) -> (Vec<String>, Vec<String>) {
        if let Some(root_cfg) = self.config.roots.get(root) {
            (root_cfg.copy.clone(), root_cfg.exec.clone())
        } else {
            (
                self.config.general.copy.clone(),
                self.config.general.exec.clone(),
            )
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let forest = Forest::new()?;

    match cli.command {
        Commands::Add { url } => cmd_add(&forest, &url),
        Commands::List { root } => cmd_list(&forest, root.as_deref()),
        Commands::Create { root, branch } => cmd_create(&forest, &root, &branch),
        Commands::Delete { root, branch } => cmd_delete(&forest, &root, branch.as_deref()),
        Commands::Switch { root, branch } => cmd_switch(&forest, &root, &branch),
        Commands::Merge {
            root,
            source,
            target,
        } => cmd_merge(&forest, &root, &source, &target),
    }
}

/// Add a new repository
fn cmd_add(forest: &Forest, url: &str) -> Result<()> {
    // Extract repo name from URL
    let repo_name = extract_repo_name(url)?;
    let root_path = forest.root_path(&repo_name);

    if root_path.exists() {
        anyhow::bail!(
            "Root '{}' already exists at {}",
            repo_name,
            root_path.display()
        );
    }

    println!("Cloning {} as bare repository...", url);

    // Clone as bare using grove
    grove::git::clone_bare(url, &root_path)?;

    // Get config for this root
    let (copy_paths, exec_commands) = forest.root_config(&repo_name);

    // Create grove config with forest settings
    let mut grove_config = grove::GroveConfig::default();
    grove_config.grove.main_branch =
        grove::git::get_default_branch(&root_path).unwrap_or_else(|_| "main".to_string());

    // Set forest-managed settings
    grove_config.grove.forest = Some(grove::config::ForestSettings {
        managed: true,
        trees: Some(PathBuf::from(forest.tree_prefix(&repo_name))),
    });

    // Set copy and hooks
    grove_config.copy.paths = copy_paths;
    grove_config.hooks.post_create = exec_commands;

    // Save grove config
    grove_config.save(&root_path.join(".grove.toml"))?;

    // Create trees directory and main worktree
    let main_branch = &grove_config.grove.main_branch;
    let main_tree_path = forest.trees_dir.join(format!(
        "{}--{}",
        repo_name,
        grove::worktree::sanitize_branch_name(main_branch)
    ));

    std::fs::create_dir_all(&forest.trees_dir)?;
    grove::git::add_worktree(&root_path, &main_tree_path, main_branch)?;

    println!("Added root: {}", repo_name);
    println!("Root path: {}", root_path.display());
    println!("Main tree: {}", main_tree_path.display());

    Ok(())
}

/// List roots and trees
fn cmd_list(forest: &Forest, filter_root: Option<&str>) -> Result<()> {
    let roots = forest.list_roots()?;

    if roots.is_empty() {
        println!("No roots found. Use 'forest add <url>' to add a repository.");
        return Ok(());
    }

    for root in &roots {
        // Skip if filtering and doesn't match
        if let Some(filter) = filter_root
            && root != filter
        {
            continue;
        }

        // Print root
        println!("{}:", root);

        // List trees for this root
        match forest.open_grove(root) {
            Ok(repo) => {
                if let Ok(worktrees) = grove::list(&repo) {
                    for wt in worktrees {
                        let marker = if wt.is_main { " *" } else { "" };
                        println!("  {}{}\t{}", wt.branch, marker, wt.path.display());
                    }
                }
            }
            Err(_) => {
                println!("  (unable to read worktrees)");
            }
        }
    }

    // If filter was specified but not found
    if let Some(filter) = filter_root
        && !roots.contains(&filter.to_string())
    {
        anyhow::bail!("Root '{}' not found", filter);
    }

    Ok(())
}

/// Create a new worktree
fn cmd_create(forest: &Forest, root: &str, branch: &str) -> Result<()> {
    let repo = forest.open_grove(root)?;
    let result = grove::create(&repo, branch)?;

    println!("Created worktree: {}", result.worktree_path.display());
    println!("Branch: {}", result.branch);

    Ok(())
}

/// Delete a root or tree
fn cmd_delete(forest: &Forest, root: &str, branch: Option<&str>) -> Result<()> {
    match branch {
        Some(branch) => {
            // Delete a specific tree
            let repo = forest.open_grove(root)?;

            // Confirm deletion
            print!("Delete worktree '{}' in root '{}'? [y/N] ", branch, root);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("Cancelled.");
                return Ok(());
            }

            grove::delete(&repo, branch)?;
            println!("Deleted worktree and branch: {}", branch);
        }
        None => {
            // Delete the entire root
            let root_path = forest.root_path(root);
            if !root_path.exists() {
                anyhow::bail!("Root '{}' not found", root);
            }

            // Find all trees for this root
            let prefix = format!("{}--", root);
            let mut tree_paths: Vec<PathBuf> = Vec::new();
            if forest.trees_dir.exists() {
                for entry in std::fs::read_dir(&forest.trees_dir)? {
                    let entry = entry?;
                    if let Some(name) = entry.file_name().to_str()
                        && name.starts_with(&prefix)
                    {
                        tree_paths.push(entry.path());
                    }
                }
            }

            // Confirm deletion
            println!("This will delete:");
            println!("  Root: {}", root_path.display());
            for tree in &tree_paths {
                println!("  Tree: {}", tree.display());
            }
            print!("Continue? [y/N] ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("Cancelled.");
                return Ok(());
            }

            // Delete trees first
            for tree in &tree_paths {
                std::fs::remove_dir_all(tree)?;
                println!("Deleted tree: {}", tree.display());
            }

            // Delete root
            std::fs::remove_dir_all(&root_path)?;
            println!("Deleted root: {}", root);
        }
    }

    Ok(())
}

/// Switch to a worktree (print path)
fn cmd_switch(forest: &Forest, root: &str, branch: &str) -> Result<()> {
    let repo = forest.open_grove(root)?;
    let path = grove::switch(&repo, branch)?;
    println!("{}", path.display());
    Ok(())
}

/// Merge branches
fn cmd_merge(forest: &Forest, root: &str, source: &str, target: &str) -> Result<()> {
    let repo = forest.open_grove(root)?;
    let result = grove::merge(&repo, source, target)?;

    println!(
        "Merged '{}' into '{}' ({})",
        result.source,
        result.target,
        result.target_worktree.display()
    );

    Ok(())
}

/// Extract repository name from URL
fn extract_repo_name(url: &str) -> Result<String> {
    let url = url.trim_end_matches('/');
    let url = url.strip_suffix(".git").unwrap_or(url);

    url.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not extract repository name from URL: {}", url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git").unwrap(),
            "repo"
        );
    }
}
