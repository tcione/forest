//! Create new worktrees

use crate::git;
use crate::repo::GroveRepo;
use crate::worktree::sanitize_branch_name;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct CreateResult {
    pub worktree_path: PathBuf,
    pub branch: String,
}

/// Create a new worktree from the latest remote default branch
pub fn create(repo: &GroveRepo, branch: &str) -> Result<CreateResult> {
    let sanitized_branch = sanitize_branch_name(branch);
    let worktree_path = repo.trees_dir().join(&sanitized_branch);

    if worktree_path.exists() {
        anyhow::bail!("Worktree already exists at: {}", worktree_path.display());
    }

    if git::branch_exists(&repo.path, branch) {
        anyhow::bail!(
            "Branch '{}' already exists. Use a different name or delete the existing branch.",
            branch
        );
    }

    ensure_default_tree(repo)?;

    if let Err(e) = git::fetch(&repo.path, "origin") {
        eprintln!("Warning: Failed to fetch from origin: {}", e);
    }

    let default_branch = repo.default_branch();
    let start_point = if git::git_command(
        &["rev-parse", &format!("origin/{}", default_branch)],
        Some(&repo.path),
    )
    .is_ok()
    {
        format!("origin/{}", default_branch)
    } else {
        default_branch.to_string()
    };

    git::add_worktree_new_branch(&repo.path, &worktree_path, branch, &start_point)
        .with_context(|| format!("Failed to create worktree for branch '{}'", branch))?;

    let default_tree_path = repo.worktree_path(default_branch);
    if default_tree_path.exists() {
        copy_paths(repo, &default_tree_path, &worktree_path)?;
    }

    run_post_create_hooks(repo, &worktree_path)?;

    Ok(CreateResult {
        worktree_path,
        branch: branch.to_string(),
    })
}

fn ensure_default_tree(repo: &GroveRepo) -> Result<()> {
    let default_branch = repo.default_branch();
    let default_tree_path = repo.worktree_path(default_branch);

    if !default_tree_path.exists() {
        git::add_worktree(&repo.path, &default_tree_path, default_branch).with_context(|| {
            format!(
                "Failed to create worktree for default branch '{}'",
                default_branch
            )
        })?;
    }

    Ok(())
}

/// Copy configured files/directories from source to destination
fn copy_paths(repo: &GroveRepo, source_tree: &Path, dest_tree: &Path) -> Result<()> {
    for path in &repo.config.copy.paths {
        let source = source_tree.join(path);
        let dest = dest_tree.join(path);

        if !source.exists() {
            continue;
        }

        if source.is_dir() {
            copy_dir_recursive(&source, &dest)
                .with_context(|| format!("Failed to copy directory: {}", path))?;
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&source, &dest)
                .with_context(|| format!("Failed to copy file: {}", path))?;
        }
    }

    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &dest_path)?;
        } else {
            std::fs::copy(&source_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Run post-create hooks
fn run_post_create_hooks(repo: &GroveRepo, worktree_path: &Path) -> Result<()> {
    for cmd in &repo.config.hooks.post_create {
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(worktree_path)
            .status()
            .with_context(|| format!("Failed to execute hook: {}", cmd))?;

        if !status.success() {
            anyhow::bail!("Post-create hook failed: {}", cmd);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_create_worktree() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = create(&repo, "feature-test").unwrap();

        assert!(result.worktree_path.exists());
        assert_eq!(result.branch, "feature-test");
        assert!(git::is_git_repo(&result.worktree_path));
    }

    #[test]
    fn test_create_worktree_sanitizes_branch_name() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let result = create(&repo, "feature/test").unwrap();

        assert!(result.worktree_path.to_string_lossy().contains("feature--test"));
        assert_eq!(result.branch, "feature/test");
    }

    #[test]
    fn test_create_worktree_fails_if_exists() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        create(&repo, "feature-test").unwrap();
        let result = create(&repo, "feature-test");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_create_copies_files() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();
        let default_tree = repo.worktree_path("main");
        std::fs::write(default_tree.join(".env"), "SECRET=123").unwrap();

        let mut config = repo.config.clone();
        config.copy.paths = vec![".env".to_string()];
        config.save(&repo.config_path()).unwrap();

        let repo = GroveRepo::open(temp_dir.path()).unwrap();
        let result = create(&repo, "feature-copy").unwrap();

        assert!(result.worktree_path.join(".env").exists());
        let content = std::fs::read_to_string(result.worktree_path.join(".env")).unwrap();
        assert_eq!(content, "SECRET=123");
    }

    #[test]
    fn test_create_copies_directories() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();
        let default_tree = repo.worktree_path("main");
        let config_dir = default_tree.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("local.json"), r#"{"key": "value"}"#).unwrap();

        let mut config = repo.config.clone();
        config.copy.paths = vec!["config".to_string()];
        config.save(&repo.config_path()).unwrap();

        let repo = GroveRepo::open(temp_dir.path()).unwrap();
        let result = create(&repo, "feature-dir-copy").unwrap();

        assert!(result.worktree_path.join("config").exists());
        assert!(result.worktree_path.join("config/local.json").exists());
    }

    #[test]
    fn test_create_runs_post_create_hooks() {
        let temp_dir = setup_grove_repo();
        let repo = GroveRepo::open(temp_dir.path()).unwrap();

        let mut config = repo.config.clone();
        config.hooks.post_create = vec!["touch .hook-ran".to_string()];
        config.save(&repo.config_path()).unwrap();

        let repo = GroveRepo::open(temp_dir.path()).unwrap();
        let result = create(&repo, "feature-hooks").unwrap();

        assert!(result.worktree_path.join(".hook-ran").exists());
    }
}
