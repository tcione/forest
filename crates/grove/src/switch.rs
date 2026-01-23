//! Switch to a worktree (print path for shell integration)

use crate::repo::GroveRepo;
use crate::worktree::sanitize_branch_name;
use anyhow::Result;
use std::path::PathBuf;

/// Get the path to a worktree for switching
/// Returns the absolute path to the worktree
pub fn switch(repo: &GroveRepo, branch: &str) -> Result<PathBuf> {
    let sanitized = sanitize_branch_name(branch);
    let worktree_path = repo.trees_dir().join(&sanitized);

    if !worktree_path.exists() {
        anyhow::bail!(
            "Worktree not found for branch '{}': {}",
            branch,
            worktree_path.display()
        );
    }

    Ok(worktree_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;
    use crate::git;
    use crate::init;
    use tempfile::TempDir;

    fn setup_grove_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

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

        init::init(None, Some(temp_dir.path())).unwrap();

        temp_dir
    }

    #[test]
    fn test_switch_returns_worktree_path() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let path = switch(&repo, "main").unwrap();
        assert!(path.exists());
        assert!(path.is_absolute());
    }

    #[test]
    fn test_switch_to_feature_branch() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        create::create(&repo, "feature-switch").unwrap();

        let path = switch(&repo, "feature-switch").unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("feature-switch"));
    }

    #[test]
    fn test_switch_nonexistent_branch() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = switch(&repo, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_switch_handles_sanitized_names() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        create::create(&repo, "feature/test").unwrap();

        // Should work with original name
        let path = switch(&repo, "feature/test").unwrap();
        assert!(path.exists());

        // Path should contain sanitized name
        assert!(path.to_string_lossy().contains("feature--test"));
    }
}
