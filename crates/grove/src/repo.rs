//! Grove repository discovery and context

use crate::config::GroveConfig;
use crate::git;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// A Grove repository context
pub struct GroveRepo {
    /// Path to the bare repository
    pub path: PathBuf,
    /// Grove configuration
    pub config: GroveConfig,
}

impl GroveRepo {
    /// Open a grove repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;

        if !git::is_bare_repo(&path) {
            anyhow::bail!("Not a bare git repository: {}", path.display());
        }

        let config_path = path.join(".grove.toml");
        let config = if config_path.exists() {
            GroveConfig::load(&config_path)?
        } else {
            GroveConfig::default()
        };

        Ok(Self { path, config })
    }

    /// Find a grove repository by searching upward from the current directory
    /// Works from within a worktree or from the bare repo itself
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        Self::discover_from(&cwd)
    }

    /// Find a grove repository by searching upward from the given path
    pub fn discover_from(start: &Path) -> Result<Self> {
        let start = start.canonicalize()?;

        // First, check if we're in a git worktree
        if let Ok(git_dir) = git::git_command(&["rev-parse", "--git-dir"], Some(&start)) {
            let git_dir = PathBuf::from(git_dir);

            // If it's a worktree, the git dir will be inside the main repo's worktrees folder
            // e.g., /path/to/bare-repo/worktrees/branch-name
            if let Some(parent) = git_dir.parent() {
                if parent
                    .file_name()
                    .map(|n| n == "worktrees")
                    .unwrap_or(false)
                {
                    if let Some(bare_repo) = parent.parent() {
                        if git::is_bare_repo(bare_repo) {
                            return Self::open(bare_repo);
                        }
                    }
                }
            }

            // Check if the git dir itself is a bare repo
            if git::is_bare_repo(&git_dir) {
                return Self::open(&git_dir);
            }

            // Resolve absolute path for git dir
            let resolved_git_dir = if git_dir.is_absolute() {
                git_dir
            } else {
                start.join(&git_dir).canonicalize()?
            };

            if git::is_bare_repo(&resolved_git_dir) {
                return Self::open(&resolved_git_dir);
            }
        }

        // Search upward for .grove.toml
        let mut current = start.as_path();
        loop {
            let config_path = current.join(".grove.toml");
            if config_path.exists() && git::is_bare_repo(current) {
                return Self::open(current);
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        anyhow::bail!(
            "Not inside a grove repository. Run 'grove init' first or navigate to a grove repo."
        )
    }

    /// Get the effective trees directory (absolute path)
    pub fn trees_dir(&self) -> PathBuf {
        let trees = self.config.trees_dir();
        if trees.is_absolute() {
            trees.clone()
        } else {
            self.path.join(trees)
        }
    }

    /// Get the path to a worktree for a given branch
    pub fn worktree_path(&self, branch: &str) -> PathBuf {
        let sanitized = crate::worktree::sanitize_branch_name(branch);
        self.trees_dir().join(sanitized)
    }

    /// Get the main branch name
    pub fn main_branch(&self) -> &str {
        &self.config.grove.main_branch
    }

    /// Reload the configuration from disk
    pub fn reload_config(&mut self) -> Result<()> {
        let config_path = self.path.join(".grove.toml");
        if config_path.exists() {
            self.config = GroveConfig::load(&config_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;
    use tempfile::TempDir;

    fn setup_grove_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create a regular git repo first
        git::git_command(&["init"], Some(temp_dir.path())).unwrap();
        git::git_command(
            &["config", "user.email", "test@test.com"],
            Some(temp_dir.path()),
        )
        .unwrap();
        git::git_command(&["config", "user.name", "Test"], Some(temp_dir.path())).unwrap();
        git::git_command(
            &["config", "commit.gpgsign", "false"],
            Some(temp_dir.path()),
        )
        .unwrap();
        git::git_command(&["checkout", "-b", "main"], Some(temp_dir.path())).unwrap();

        let file_path = temp_dir.path().join("README.md");
        std::fs::write(&file_path, "# Test").unwrap();
        git::git_command(&["add", "."], Some(temp_dir.path())).unwrap();
        git::git_command(&["commit", "-m", "Initial commit"], Some(temp_dir.path())).unwrap();

        // Convert to grove
        init::init(None, Some(temp_dir.path())).unwrap();

        temp_dir
    }

    #[test]
    fn test_open_grove_repo() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        assert!(repo.path.exists());
        assert_eq!(repo.main_branch(), "main");
    }

    #[test]
    fn test_discover_from_bare_repo() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::discover_from(temp_dir.path()).unwrap();

        assert_eq!(repo.path, temp_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_discover_from_worktree() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();
        let main_tree = repo.worktree_path("main");

        let discovered = GroveRepo::discover_from(&main_tree).unwrap();
        assert_eq!(discovered.path, temp_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_trees_dir_is_absolute() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let trees_dir = repo.trees_dir();
        assert!(trees_dir.is_absolute());
    }

    #[test]
    fn test_worktree_path() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let path = repo.worktree_path("feature/test");
        assert!(path.to_string_lossy().contains("feature--test"));
    }
}
