//! Grove initialization - convert or clone repos as bare

use crate::config::GroveConfig;
use crate::git;
use crate::worktree::sanitize_branch_name;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct InitResult {
    pub repo_path: PathBuf,
    pub default_tree_path: PathBuf,
    pub default_branch: String,
}

/// Initialize a grove repository
///
/// If url is provided, clones the repo as bare.
/// If no url, converts the current directory's repo to bare.
/// The default_branch parameter specifies which branch to use as default.
pub fn init(
    url: Option<&str>,
    target_dir: Option<&Path>,
    default_branch: &str,
) -> Result<InitResult> {
    match url {
        Some(url) => init_from_url(url, target_dir, default_branch),
        None => init_from_existing(target_dir.unwrap_or(Path::new(".")), default_branch),
    }
}

/// Clone a repository as bare and set up grove
fn init_from_url(url: &str, target_dir: Option<&Path>, default_branch: &str) -> Result<InitResult> {
    let repo_name = extract_repo_name(url)?;
    let dest = target_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&repo_name));

    if dest.exists() {
        anyhow::bail!("Destination directory already exists: {}", dest.display());
    }

    git::clone_bare(url, &dest).with_context(|| format!("Failed to clone {}", url))?;

    setup_grove(&dest, default_branch)
}

/// Convert an existing repository to bare and set up grove
fn init_from_existing(repo_path: &Path, default_branch: &str) -> Result<InitResult> {
    let repo_path = repo_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve repository path: {}", repo_path.display()))?;

    if !git::is_git_repo(&repo_path) {
        anyhow::bail!("Not a git repository: {}", repo_path.display());
    }

    if git::is_bare_repo(&repo_path) {
        anyhow::bail!("Repository is already bare: {}", repo_path.display());
    }

    git::convert_to_bare(&repo_path).with_context(|| "Failed to convert repository to bare")?;

    setup_grove(&repo_path, default_branch)
}

/// Set up grove configuration and create default worktree
fn setup_grove(repo_path: &Path, default_branch: &str) -> Result<InitResult> {
    let mut config = GroveConfig::default();
    config.grove.default_branch = default_branch.to_string();

    let trees_dir = repo_path.join(config.trees_dir());
    std::fs::create_dir_all(&trees_dir)?;

    let sanitized_branch = sanitize_branch_name(default_branch);
    let default_tree_path = trees_dir.join(&sanitized_branch);

    git::add_worktree(repo_path, &default_tree_path, default_branch).with_context(|| {
        format!(
            "Failed to create worktree for default branch '{}'",
            default_branch
        )
    })?;

    // Save config in the default branch worktree (not bare repo)
    let config_path = default_tree_path.join(".grove.toml");
    config.save(&config_path)?;

    Ok(InitResult {
        repo_path: repo_path.to_path_buf(),
        default_tree_path,
        default_branch: default_branch.to_string(),
    })
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
    use crate::git::git_command;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        git_command(&["init"], Some(temp_dir.path())).unwrap();
        git_command(
            &["config", "user.email", "test@test.com"],
            Some(temp_dir.path()),
        )
        .unwrap();
        git_command(&["config", "user.name", "Test"], Some(temp_dir.path())).unwrap();
        git_command(
            &["config", "commit.gpgsign", "false"],
            Some(temp_dir.path()),
        )
        .unwrap();

        // Create initial commit on main branch
        git_command(&["checkout", "-b", "main"], Some(temp_dir.path())).unwrap();
        let file_path = temp_dir.path().join("README.md");
        std::fs::write(&file_path, "# Test").unwrap();
        git_command(&["add", "."], Some(temp_dir.path())).unwrap();
        git_command(&["commit", "-m", "Initial commit"], Some(temp_dir.path())).unwrap();

        temp_dir
    }

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
        assert_eq!(
            extract_repo_name("git@github.com:user/repo").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo/").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_init_converts_to_bare_and_creates_default_worktree() {
        let temp_dir = setup_test_repo();
        let repo_path = temp_dir.path();

        assert!(!git::is_bare_repo(repo_path));

        let result = init(None, Some(repo_path), "main").unwrap();

        assert!(git::is_bare_repo(&result.repo_path));
        assert!(result.default_tree_path.exists());
        assert_eq!(result.default_branch, "main");
        assert!(git::is_git_repo(&result.default_tree_path));
        // Config is in worktree, not bare repo
        assert!(result.default_tree_path.join(".grove.toml").exists());
        assert!(!result.repo_path.join(".grove.toml").exists());
    }

    #[test]
    fn test_init_fails_on_non_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = init(None, Some(temp_dir.path()), "main");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a git repository"));
    }

    #[test]
    fn test_init_fails_on_already_bare() {
        let temp_dir = setup_test_repo();
        let repo_path = temp_dir.path();

        init(None, Some(repo_path), "main").unwrap();

        let result = init(None, Some(repo_path), "main");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already bare"));
    }
}
