use std::path::PathBuf;
use regex::Regex;
use anyhow::{Result, Context};

// TODO:
// - root completion
// - Maybe? "fatal: a branch named 'feature/maybe-a-rust-rewrite' already exists"
pub fn run(roots_dir: &PathBuf, trees_dir: &PathBuf, root: &str, new_branch_name: &str) -> Result<()> {
    // check if repo folder exists
    let repo_root = roots_dir.join(root);
    let branch_tree = trees_dir.join(tree_name(root, new_branch_name));
    let default_branch = default_branch(&repo_root)?;

    pull_latest(&repo_root, &default_branch)?;

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["worktree", "add"])
        .arg("-b")
        .arg(new_branch_name)
        .arg(branch_tree)
        .arg(&default_branch)
        .output()
        .context("Failed to create new worktree")?;

    if !output.status.success() {
        anyhow::bail!("Create failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn tree_name(root: &str, new_branch_name: &str) -> String {
    let regex = Regex::new(r"[^A-Za-z0-9\-_]").unwrap();
    let normalized_branch_name = regex.replace_all(new_branch_name, "--");

    format!("{}--{}", root, normalized_branch_name)
}

fn default_branch(repo_root: &PathBuf) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to get default branch")?;

    if !output.status.success() {
        anyhow::bail!("Failed to find default branch: {}", String::from_utf8_lossy(&output.stderr));
    }

    let branch_ref = String::from_utf8(output.stdout).context("Failed to convert default branch to string")?;
    let filtered_branch = branch_ref.trim().strip_prefix("refs/remotes/origin/").unwrap_or("main").to_string();

    Ok(filtered_branch)
}

fn pull_latest(repo_root: &PathBuf, branch: &str) -> Result<()> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["pull", "origin", branch])
        .output()
        .context("Failed to pull default branch")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::commands::clone;

    const TEST_REPO_URL: &str = "https://github.com/tcione/test-repo.git";

    fn tree_branch(tree_path: &PathBuf) -> Result<String> {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(tree_path)
            .args(["branch", "--show-current"])
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            anyhow::bail!("Failed to get current branch: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    #[test]
    fn test_create_worktree_success() {
        let roots_dir = TempDir::new().unwrap().path().to_path_buf();
        let trees_dir = TempDir::new().unwrap().path().to_path_buf();
        let tree_path = trees_dir.join("test-repo--feature--new-feature");

        clone::run(&roots_dir, TEST_REPO_URL.to_string()).unwrap();
        run(&roots_dir, &trees_dir, "test-repo", "feature/new-feature").unwrap();

        let tree_branch = tree_branch(&tree_path).unwrap();

        assert_eq!(tree_branch, "feature/new-feature".to_string())
    }

    #[test]
    fn test_tree_name() {
      assert_eq!(tree_name("myrepo", "feature/normal"), "myrepo--feature--normal");
      assert_eq!(tree_name("myrepo", "hotfix@at-you"), "myrepo--hotfix--at-you");
      assert_eq!(tree_name("myrepo", "hotfix/@slash-at-you"), "myrepo--hotfix--slash-at-you");
      assert_eq!(tree_name("myrepo", "feat/user-mgmt_new"), "myrepo--feat--user-mgmt_new");
      assert_eq!(tree_name("myrepo", "feat//too-many-hyphens"), "myrepo--feat--too-many-hyphens");
      assert_eq!(tree_name("myrepo", "feat/////way-too-many-hyphens"), "myrepo--feat--way-too-many-hyphens");
      assert_eq!(tree_name("myrepo", "        feat/trimmed   "), "myrepo--feat--trimmed");
    }

    #[test]
    fn test_create_with_nonexistent_repo() {
        let roots_dir = TempDir::new().unwrap().path().to_path_buf();
        let trees_dir = TempDir::new().unwrap().path().to_path_buf();
        let err = run(&roots_dir, &trees_dir, "nonexistent-repo", "feature/test").unwrap_err();

        assert!(err.to_string().contains("No such file or directory"))
    }

    #[test]
    fn test_duplicate_branch_name() {
        let roots_dir = TempDir::new().unwrap().path().to_path_buf();
        let trees_dir = TempDir::new().unwrap().path().to_path_buf();

        clone::run(&roots_dir, TEST_REPO_URL.to_string()).unwrap();
        run(&roots_dir, &trees_dir, "test-repo", "feature/new-feature").unwrap();
        let err = run(&roots_dir, &trees_dir, "test-repo", "feature/new-feature").unwrap_err();

        assert!(err.to_string().contains("a branch named 'feature/new-feature' already exists"))
    }
}
