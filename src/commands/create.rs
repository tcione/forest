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
