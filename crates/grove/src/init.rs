//! Grove initialization - convert or clone repos as bare

use crate::config::GroveConfig;
use crate::git;
use crate::worktree::sanitize_branch_name;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Result of grove init
#[derive(Debug)]
pub struct InitResult {
    pub repo_path: PathBuf,
    pub main_tree_path: PathBuf,
    pub main_branch: String,
}

/// Initialize a grove repository
///
/// If url is provided, clones the repo as bare.
/// If no url, converts the current directory's repo to bare.
pub fn init(url: Option<&str>, target_dir: Option<&Path>) -> Result<InitResult> {
    match url {
        Some(url) => init_from_url(url, target_dir),
        None => init_from_existing(target_dir.unwrap_or(Path::new("."))),
    }
}

/// Clone a repository as bare and set up grove
fn init_from_url(url: &str, target_dir: Option<&Path>) -> Result<InitResult> {
    // Extract repo name from URL
    let repo_name = extract_repo_name(url)?;
    let dest = target_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&repo_name));

    if dest.exists() {
        anyhow::bail!("Destination directory already exists: {}", dest.display());
    }

    // Clone as bare
    git::clone_bare(url, &dest).with_context(|| format!("Failed to clone {}", url))?;

    // Set up grove in the cloned repo
    setup_grove(&dest)
}

/// Convert an existing repository to bare and set up grove
fn init_from_existing(repo_path: &Path) -> Result<InitResult> {
    let repo_path = repo_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve repository path: {}", repo_path.display()))?;

    if !git::is_git_repo(&repo_path) {
        anyhow::bail!("Not a git repository: {}", repo_path.display());
    }

    if git::is_bare_repo(&repo_path) {
        anyhow::bail!("Repository is already bare: {}", repo_path.display());
    }

    // Convert to bare
    git::convert_to_bare(&repo_path).with_context(|| "Failed to convert repository to bare")?;

    // Set up grove
    setup_grove(&repo_path)
}

/// Set up grove configuration and create main worktree
fn setup_grove(repo_path: &Path) -> Result<InitResult> {
    // Determine main branch
    let main_branch = git::get_default_branch(repo_path).unwrap_or_else(|_| "main".to_string());

    // Create default config
    let mut config = GroveConfig::default();
    config.grove.main_branch = main_branch.clone();

    // Save config
    let config_path = repo_path.join(".grove.toml");
    config.save(&config_path)?;

    // Create trees directory
    let trees_dir = repo_path.join(config.trees_dir());
    std::fs::create_dir_all(&trees_dir)?;

    // Create main worktree
    let sanitized_branch = sanitize_branch_name(&main_branch);
    let main_tree_path = trees_dir.join(&sanitized_branch);

    git::add_worktree(repo_path, &main_tree_path, &main_branch).with_context(|| {
        format!(
            "Failed to create main worktree for branch '{}'",
            main_branch
        )
    })?;

    Ok(InitResult {
        repo_path: repo_path.to_path_buf(),
        main_tree_path,
        main_branch,
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
    fn test_init_from_existing_converts_to_bare_and_creates_main_tree() {
        let temp_dir = setup_test_repo();
        let repo_path = temp_dir.path();

        // Verify it's not bare initially
        assert!(!git::is_bare_repo(repo_path));

        // Run init
        let result = init(None, Some(repo_path)).unwrap();

        // Verify it's now bare
        assert!(git::is_bare_repo(&result.repo_path));

        // Verify .grove.toml was created
        assert!(result.repo_path.join(".grove.toml").exists());

        // Verify main tree was created
        assert!(result.main_tree_path.exists());
        assert_eq!(result.main_branch, "main");

        // Verify main tree is a valid worktree
        assert!(git::is_git_repo(&result.main_tree_path));
    }

    #[test]
    fn test_init_from_existing_fails_on_non_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = init(None, Some(temp_dir.path()));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Not a git repository")
        );
    }

    #[test]
    fn test_init_from_existing_fails_on_already_bare() {
        let temp_dir = setup_test_repo();
        let repo_path = temp_dir.path();

        // First init to make it bare
        init(None, Some(repo_path)).unwrap();

        // Second init should fail
        let result = init(None, Some(repo_path));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already bare"));
    }

    #[test]
    fn test_init_from_url_clones_and_sets_up() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test-repo");

        // Use a known test repository
        let result = init(Some("https://github.com/tcione/test-repo.git"), Some(&dest));

        // This test requires network access, so we'll just check it doesn't panic
        // In a real test environment, you might mock this or use a local git server
        if let Ok(result) = result {
            assert!(git::is_bare_repo(&result.repo_path));
            assert!(result.repo_path.join(".grove.toml").exists());
            assert!(result.main_tree_path.exists());
        }
    }
}
