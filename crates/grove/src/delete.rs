//! Delete worktrees and their branches

use crate::git;
use crate::repo::GroveRepo;
use crate::worktree::sanitize_branch_name;
use anyhow::{Context, Result};

/// Delete a worktree and its branch
pub fn delete(repo: &GroveRepo, branch: &str) -> Result<()> {
    let sanitized = sanitize_branch_name(branch);
    let worktree_path = repo.trees_dir().join(&sanitized);

    if branch == repo.default_branch() {
        anyhow::bail!("Cannot delete the default branch worktree");
    }

    if !worktree_path.exists() {
        anyhow::bail!("Worktree not found: {}", worktree_path.display());
    }

    git::remove_worktree(&repo.path, &worktree_path)
        .with_context(|| format!("Failed to remove worktree at: {}", worktree_path.display()))?;

    if git::branch_exists(&repo.path, branch) {
        git::delete_branch(&repo.path, branch)
            .with_context(|| format!("Failed to delete branch: {}", branch))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;
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

        init::init(None, Some(temp_dir.path()), "main").unwrap();

        temp_dir
    }

    #[test]
    fn test_delete_worktree_and_branch() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let create_result = create::create(&repo, "feature-delete").unwrap();
        assert!(create_result.worktree_path.exists());
        assert!(git::branch_exists(&repo.path, "feature-delete"));

        delete(&repo, "feature-delete").unwrap();

        assert!(!create_result.worktree_path.exists());
        assert!(!git::branch_exists(&repo.path, "feature-delete"));
    }

    #[test]
    fn test_delete_nonexistent_worktree() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = delete(&repo, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_cannot_delete_default_branch() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = delete(&repo, "main");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot delete the default branch"));
    }
}
