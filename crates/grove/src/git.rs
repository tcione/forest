//! Git operations wrapper

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Execute a git command and return stdout
pub fn git_command(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute git {:?}", args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {:?} failed: {}", args, stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Execute a git command, returning success/failure without output
pub fn git_command_status(args: &[&str], cwd: Option<&Path>) -> Result<bool> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let status = cmd
        .status()
        .with_context(|| format!("Failed to execute git {:?}", args))?;

    Ok(status.success())
}

/// Check if a directory is a git repository
pub fn is_git_repo(path: &Path) -> bool {
    git_command(&["rev-parse", "--git-dir"], Some(path)).is_ok()
}

/// Check if a repository is bare
pub fn is_bare_repo(path: &Path) -> bool {
    git_command(&["rev-parse", "--is-bare-repository"], Some(path))
        .map(|s| s == "true")
        .unwrap_or(false)
}

/// Clone a repository as bare
pub fn clone_bare(url: &str, dest: &Path) -> Result<()> {
    git_command(&["clone", "--bare", url, dest.to_str().unwrap()], None)?;
    Ok(())
}

/// Convert an existing repository to bare
pub fn convert_to_bare(repo_path: &Path) -> Result<()> {
    // Get the .git directory
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        anyhow::bail!("Not a git repository: {}", repo_path.display());
    }

    // Move .git contents to repo root and mark as bare
    let temp_git = repo_path.join(".git_temp");
    std::fs::rename(&git_dir, &temp_git)?;

    // Remove working directory files (keep only what was in .git)
    for entry in std::fs::read_dir(repo_path)? {
        let entry = entry?;
        let path = entry.path();
        if path != temp_git {
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
        }
    }

    // Move git internals to root
    for entry in std::fs::read_dir(&temp_git)? {
        let entry = entry?;
        let dest = repo_path.join(entry.file_name());
        std::fs::rename(entry.path(), dest)?;
    }
    std::fs::remove_dir(&temp_git)?;

    // Set bare = true in config
    git_command(&["config", "core.bare", "true"], Some(repo_path))?;

    Ok(())
}

/// Fetch from remote
pub fn fetch(repo_path: &Path, remote: &str) -> Result<()> {
    git_command(&["fetch", remote], Some(repo_path))?;
    Ok(())
}

/// Get the default branch name (main or master)
pub fn get_default_branch(repo_path: &Path) -> Result<String> {
    // Try to get from remote HEAD
    if let Ok(output) = git_command(
        &["symbolic-ref", "refs/remotes/origin/HEAD"],
        Some(repo_path),
    ) && let Some(branch) = output.strip_prefix("refs/remotes/origin/")
    {
        return Ok(branch.to_string());
    }

    // Fallback: check if main or master exists
    if git_command(
        &["show-ref", "--verify", "refs/heads/main"],
        Some(repo_path),
    )
    .is_ok()
    {
        return Ok("main".to_string());
    }
    if git_command(
        &["show-ref", "--verify", "refs/heads/master"],
        Some(repo_path),
    )
    .is_ok()
    {
        return Ok("master".to_string());
    }

    anyhow::bail!("Could not determine default branch")
}

/// Create a worktree
pub fn add_worktree(repo_path: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    git_command(
        &["worktree", "add", worktree_path.to_str().unwrap(), branch],
        Some(repo_path),
    )?;
    Ok(())
}

/// Create a worktree with a new branch from a start point
pub fn add_worktree_new_branch(
    repo_path: &Path,
    worktree_path: &Path,
    new_branch: &str,
    start_point: &str,
) -> Result<()> {
    git_command(
        &[
            "worktree",
            "add",
            "-b",
            new_branch,
            worktree_path.to_str().unwrap(),
            start_point,
        ],
        Some(repo_path),
    )?;
    Ok(())
}

/// Remove a worktree
pub fn remove_worktree(repo_path: &Path, worktree_path: &Path) -> Result<()> {
    git_command(
        &["worktree", "remove", worktree_path.to_str().unwrap()],
        Some(repo_path),
    )?;
    Ok(())
}

/// Delete a branch
pub fn delete_branch(repo_path: &Path, branch: &str) -> Result<()> {
    git_command(&["branch", "-D", branch], Some(repo_path))?;
    Ok(())
}

/// List worktrees
pub fn list_worktrees(repo_path: &Path) -> Result<Vec<(String, String)>> {
    let output = git_command(&["worktree", "list", "--porcelain"], Some(repo_path))?;
    let mut worktrees = Vec::new();
    let mut current_path = String::new();

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = path.to_string();
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/")
            && !current_path.is_empty()
        {
            worktrees.push((current_path.clone(), branch.to_string()));
        }
    }

    Ok(worktrees)
}

/// Check if a branch exists
pub fn branch_exists(repo_path: &Path, branch: &str) -> bool {
    git_command(
        &["show-ref", "--verify", &format!("refs/heads/{}", branch)],
        Some(repo_path),
    )
    .is_ok()
}

/// Get current branch name
pub fn current_branch(worktree_path: &Path) -> Result<String> {
    git_command(&["rev-parse", "--abbrev-ref", "HEAD"], Some(worktree_path))
}

/// Merge a branch into another
pub fn merge(worktree_path: &Path, source_branch: &str) -> Result<()> {
    git_command(&["merge", source_branch], Some(worktree_path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        // Disable commit signing for tests
        git_command(
            &["config", "commit.gpgsign", "false"],
            Some(temp_dir.path()),
        )
        .unwrap();

        // Create initial commit
        let file_path = temp_dir.path().join("README.md");
        std::fs::write(&file_path, "# Test").unwrap();
        git_command(&["add", "."], Some(temp_dir.path())).unwrap();
        git_command(&["commit", "-m", "Initial commit"], Some(temp_dir.path())).unwrap();

        temp_dir
    }

    #[test]
    fn test_is_git_repo() {
        let temp_dir = setup_test_repo();
        assert!(is_git_repo(temp_dir.path()));

        let non_repo = TempDir::new().unwrap();
        assert!(!is_git_repo(non_repo.path()));
    }

    #[test]
    fn test_is_bare_repo() {
        let temp_dir = setup_test_repo();
        assert!(!is_bare_repo(temp_dir.path()));
    }

    #[test]
    fn test_branch_exists() {
        let temp_dir = setup_test_repo();

        // main or master should exist depending on git version
        let has_main = branch_exists(temp_dir.path(), "main");
        let has_master = branch_exists(temp_dir.path(), "master");
        assert!(has_main || has_master);

        assert!(!branch_exists(temp_dir.path(), "nonexistent"));
    }

    #[test]
    fn test_list_worktrees() {
        let temp_dir = setup_test_repo();
        let worktrees = list_worktrees(temp_dir.path()).unwrap();

        // Should have at least the main worktree
        assert!(!worktrees.is_empty());
    }
}
