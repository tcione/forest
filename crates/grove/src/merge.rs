//! Merge branches between worktrees

use crate::git;
use crate::repo::GroveRepo;
use crate::worktree::sanitize_branch_name;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Result of a merge operation
#[derive(Debug)]
pub struct MergeResult {
    /// Source branch that was merged
    pub source: String,
    /// Target branch that received the merge
    pub target: String,
    /// Path to the target worktree
    pub target_worktree: PathBuf,
}

/// Merge source branch into target branch
///
/// Use "." for source or target to mean the current branch (determined from cwd)
pub fn merge(repo: &GroveRepo, source: &str, target: &str) -> Result<MergeResult> {
    // Resolve "." to current branch
    let source_branch = resolve_branch(source)?;
    let target_branch = resolve_branch(target)?;

    if source_branch == target_branch {
        anyhow::bail!("Cannot merge a branch into itself");
    }

    // Verify source branch exists
    if !git::branch_exists(&repo.path, &source_branch) {
        anyhow::bail!("Source branch does not exist: {}", source_branch);
    }

    // Get the target worktree path
    let sanitized_target = sanitize_branch_name(&target_branch);
    let target_worktree = repo.trees_dir().join(&sanitized_target);

    if !target_worktree.exists() {
        anyhow::bail!(
            "Target worktree not found for branch '{}'. Create it first with 'grove create'.",
            target_branch
        );
    }

    // Fetch latest from remote (best effort)
    if let Err(e) = git::fetch(&repo.path, "origin") {
        eprintln!("Warning: Failed to fetch from origin: {}", e);
    }

    // Perform the merge in the target worktree
    git::merge(&target_worktree, &source_branch).with_context(|| {
        format!(
            "Failed to merge '{}' into '{}'",
            source_branch, target_branch
        )
    })?;

    Ok(MergeResult {
        source: source_branch,
        target: target_branch,
        target_worktree,
    })
}

/// Resolve branch reference - "." means current branch from cwd
fn resolve_branch(branch: &str) -> Result<String> {
    if branch == "." {
        let cwd = std::env::current_dir()?;
        git::current_branch(&cwd).with_context(
            || "Cannot determine current branch. Make sure you're in a worktree when using '.'",
        )
    } else {
        Ok(branch.to_string())
    }
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

        init::init(None, Some(temp_dir.path())).unwrap();

        temp_dir
    }

    #[test]
    fn test_merge_feature_into_main() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        // Create a feature branch
        let feature_result = create::create(&repo, "feature-merge-test").unwrap();

        // Make a commit on the feature branch
        let feature_file = feature_result.worktree_path.join("feature.txt");
        std::fs::write(&feature_file, "feature content").unwrap();
        git::git_command(&["add", "."], Some(&feature_result.worktree_path)).unwrap();
        git::git_command(
            &["commit", "-m", "Add feature"],
            Some(&feature_result.worktree_path),
        )
        .unwrap();

        // Merge feature into main
        let result = merge(&repo, "feature-merge-test", "main").unwrap();

        assert_eq!(result.source, "feature-merge-test");
        assert_eq!(result.target, "main");

        // Verify the file is now in main
        let main_tree = repo.worktree_path("main");
        assert!(main_tree.join("feature.txt").exists());
    }

    #[test]
    fn test_merge_fails_same_branch() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = merge(&repo, "main", "main");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("into itself"));
    }

    #[test]
    fn test_merge_fails_nonexistent_source() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = merge(&repo, "nonexistent", "main");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_merge_fails_nonexistent_target_worktree() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        // Create a branch but not a worktree for it
        git::git_command(&["branch", "orphan-branch"], Some(temp_dir.path())).unwrap();

        let result = merge(&repo, "main", "orphan-branch");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("worktree not found")
        );
    }
}
