//! List worktrees

use crate::git;
use crate::repo::GroveRepo;
use anyhow::Result;
use std::path::PathBuf;

/// Information about a worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub is_default: bool,
}

/// List all worktrees in the repository
pub fn list(repo: &GroveRepo) -> Result<Vec<WorktreeInfo>> {
    let worktrees = git::list_worktrees(&repo.path)?;
    let default_branch = repo.default_branch();

    let mut result: Vec<WorktreeInfo> = worktrees
        .into_iter()
        .map(|(path, branch)| WorktreeInfo {
            path: PathBuf::from(path),
            branch: branch.clone(),
            is_default: branch == default_branch,
        })
        .collect();

    result.sort_by(|a, b| {
        if a.is_default {
            std::cmp::Ordering::Less
        } else if b.is_default {
            std::cmp::Ordering::Greater
        } else {
            a.branch.cmp(&b.branch)
        }
    });

    Ok(result)
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
    fn test_list_shows_main_worktree() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let worktrees = list(&repo).unwrap();

        assert!(!worktrees.is_empty());
        assert!(worktrees.iter().any(|w| w.branch == "main" && w.is_default));
    }

    #[test]
    fn test_list_shows_all_worktrees() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        // Create additional worktrees
        create::create(&repo, "feature-a").unwrap();
        create::create(&repo, "feature-b").unwrap();

        let worktrees = list(&repo).unwrap();

        assert_eq!(worktrees.len(), 3);
        assert_eq!(worktrees[0].branch, "main"); // main should be first
        assert!(worktrees.iter().any(|w| w.branch == "feature-a"));
        assert!(worktrees.iter().any(|w| w.branch == "feature-b"));
    }

    #[test]
    fn test_list_main_is_first() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        create::create(&repo, "aaa-feature").unwrap();
        create::create(&repo, "zzz-feature").unwrap();

        let worktrees = list(&repo).unwrap();

        // Main should be first even though 'aaa' would come before 'main' alphabetically
        assert_eq!(worktrees[0].branch, "main");
        assert!(worktrees[0].is_default);
    }
}
