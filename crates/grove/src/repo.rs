//! Grove repository discovery and context

use crate::config::GroveConfig;
use crate::git;
use crate::worktree::sanitize_branch_name;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// A Grove repository context
pub struct GroveRepo {
    pub path: PathBuf,
    pub config: GroveConfig,
    pub config_initialized: bool,
}

impl GroveRepo {
    /// Open a grove repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_worktree(path, None)
    }

    /// Open a grove repository, optionally checking a specific worktree for config first
    fn open_with_worktree(path: &Path, current_worktree: Option<&Path>) -> Result<Self> {
        let path = path.canonicalize()?;

        if !git::is_bare_repo(&path) {
            anyhow::bail!("Not a bare git repository: {}", path.display());
        }

        let (config, initialized) = Self::load_config(&path, current_worktree)?;
        Ok(Self { path, config, config_initialized: initialized })
    }

    /// Load config with discovery order: current worktree → default branch worktree → defaults
    fn load_config(bare_repo: &Path, current_worktree: Option<&Path>) -> Result<(GroveConfig, bool)> {
        // 1. Check current worktree first (if provided)
        if let Some(worktree) = current_worktree {
            let config_path = worktree.join(".grove.toml");
            if config_path.exists() {
                return Ok((GroveConfig::load(&config_path)?, false));
            }
        }

        // 2. Check default branch worktree
        if let Ok(default_branch) = git::get_default_branch(bare_repo) {
            let trees_dir = bare_repo.join("trees");
            let worktree_config = trees_dir
                .join(sanitize_branch_name(&default_branch))
                .join(".grove.toml");
            if worktree_config.exists() {
                return Ok((GroveConfig::load(&worktree_config)?, false));
            }
        }

        // 3. No config found - return defaults and mark as initialized
        Ok((GroveConfig::default(), true))
    }

    /// Find a grove repository from the current directory
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        Self::discover_from(&cwd)
    }

    /// Find a grove repository by searching upward from the given path
    pub fn discover_from(start: &Path) -> Result<Self> {
        let start = start.canonicalize()?;

        if let Ok(git_dir) = git::git_command(&["rev-parse", "--git-dir"], Some(&start)) {
            let git_dir = PathBuf::from(git_dir);

            // Check if we're in a worktree (git dir is inside worktrees folder)
            if let Some(parent) = git_dir.parent()
                && parent.file_name().map(|n| n == "worktrees").unwrap_or(false)
                && let Some(bare_repo) = parent.parent()
                && git::is_bare_repo(bare_repo)
            {
                // Pass the worktree root for config discovery
                let worktree_root = git::git_command(&["rev-parse", "--show-toplevel"], Some(&start))
                    .ok()
                    .map(|s| PathBuf::from(s.trim()));
                return Self::open_with_worktree(bare_repo, worktree_root.as_deref());
            }

            if git::is_bare_repo(&git_dir) {
                return Self::open(&git_dir);
            }

            let resolved = if git_dir.is_absolute() {
                git_dir
            } else {
                start.join(&git_dir).canonicalize()?
            };

            if git::is_bare_repo(&resolved) {
                return Self::open(&resolved);
            }
        }

        anyhow::bail!("Not inside a grove repository. Run 'grove init' first.")
    }

    pub fn trees_dir(&self) -> PathBuf {
        let trees = self.config.trees_dir();
        if trees.is_absolute() {
            trees.clone()
        } else {
            self.path.join(trees)
        }
    }

    pub fn worktree_path(&self, branch: &str) -> PathBuf {
        self.trees_dir().join(sanitize_branch_name(branch))
    }

    pub fn default_branch(&self) -> &str {
        &self.config.grove.default_branch
    }

    /// Get path where config should be stored (in default branch worktree)
    pub fn config_path(&self) -> PathBuf {
        self.worktree_path(self.default_branch()).join(".grove.toml")
    }

    pub fn reload_config(&mut self) -> Result<()> {
        let (config, initialized) = Self::load_config(&self.path, None)?;
        self.config = config;
        self.config_initialized = initialized;
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

        git::git_command(&["init"], Some(temp_dir.path())).unwrap();
        git::git_command(&["config", "user.email", "test@test.com"], Some(temp_dir.path())).unwrap();
        git::git_command(&["config", "user.name", "Test"], Some(temp_dir.path())).unwrap();
        git::git_command(&["config", "commit.gpgsign", "false"], Some(temp_dir.path())).unwrap();
        git::git_command(&["checkout", "-b", "main"], Some(temp_dir.path())).unwrap();

        std::fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
        git::git_command(&["add", "."], Some(temp_dir.path())).unwrap();
        git::git_command(&["commit", "-m", "Initial"], Some(temp_dir.path())).unwrap();

        init::init(None, Some(temp_dir.path()), "main").unwrap();
        temp_dir
    }

    #[test]
    fn test_repo_discovery_and_config() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        assert!(repo.path.exists());
        assert!(repo.trees_dir().is_absolute());
        assert!(repo.worktree_path("feature/test").to_string_lossy().contains("feature--test"));

        let from_worktree = GroveRepo::discover_from(&repo.worktree_path("main")).unwrap();
        assert_eq!(from_worktree.path, temp_dir.path().canonicalize().unwrap());
    }
}
