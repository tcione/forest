use anyhow::{Context, Result};
use regex::Regex;
use std::path::PathBuf;

use crate::application::Application;

// TODO: root completion
// TODO: Handle already exists by using goto
// TODO: Handle exec in background and notify user using notify-rust
//       For that I also need to have proper logs somewhere
pub fn run(application: &Application, root: &str, new_branch_name: &str) -> Result<()> {
    // TODO: check if repo folder exists
    let roots_dir = &application.roots_dir;
    let trees_dir = &application.trees_dir;
    let repo_root = roots_dir.join(root);
    let branch_tree = trees_dir.join(tree_name(root, new_branch_name));
    let default_branch = default_branch(&repo_root)?;

    pull_latest(&repo_root, &default_branch)?;
    add_worktree(&repo_root, &new_branch_name, &branch_tree, &default_branch)?;
    set_up_worktree(application, root, &repo_root, &branch_tree)?;

    Ok(())
}

fn tree_name(root: &str, new_branch_name: &str) -> String {
    let trimmed = new_branch_name.trim();
    let regex = Regex::new(r"[^A-Za-z0-9\-_]+").unwrap();
    let normalized = regex.replace_all(trimmed, "--");

    format!("{}--{}", root, &normalized)
}

fn default_branch(repo_root: &PathBuf) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to get default branch")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to find default branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let branch_ref =
        String::from_utf8(output.stdout).context("Failed to convert default branch to string")?;
    let filtered_branch = branch_ref
        .trim()
        .strip_prefix("refs/remotes/origin/")
        .unwrap_or("main")
        .to_string();

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

fn add_worktree(
    repo_root: &PathBuf,
    new_branch_name: &str,
    branch_tree: &PathBuf,
    default_branch: &str,
) -> Result<()> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["worktree", "add"])
        .arg("-b")
        .arg(new_branch_name)
        .arg(branch_tree)
        .arg(default_branch)
        .output()
        .context("Failed to create new worktree")?;

    if !output.status.success() {
        anyhow::bail!("Create failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn set_up_worktree(
    application: &Application,
    root: &str,
    repo_root: &PathBuf,
    branch_tree: &PathBuf,
) -> Result<()> {
    let (copy, exec) = if let Some(root_config) = application.config.roots.get(root) {
        (&root_config.copy, &root_config.exec)
    } else {
        (
            &application.config.general.copy,
            &application.config.general.exec,
        )
    };

    copy_files(root, repo_root, branch_tree, copy);
    exec_commands(root, branch_tree, exec);

    Ok(())
}

fn copy_files(root: &str, repo_root: &PathBuf, branch_tree: &PathBuf, copy: &Vec<String>) {
    for file_name in copy {
        let source = repo_root.join(file_name);
        let destination = branch_tree.join(file_name);

        if !source.exists() {
            println!(
                "Skipping: \"{}\" does not exist in \"{}\"",
                &file_name, &root
            );
            continue;
        }

        if let Err(e) = std::fs::copy(&source, &destination) {
            println!(
                "Failed to copy \"{}\" to \"{}\". Error: {:?}",
                &file_name, &root, &e
            );
            continue;
        }

        println!("Copied \"{}\" to \"{}\"", &file_name, &root);
    }
}

fn exec_commands(root: &str, branch_tree: &PathBuf, exec: &Vec<String>) {
    for command in exec {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(branch_tree)
            .output();

        if let Err(e) = output {
            println!(
                "Failed to execute \"{}\" for \"{}\". Error: {:?}",
                &command, &root, &e
            );
            continue;
        }

        let output_unwrapped = output.unwrap();

        if !output_unwrapped.status.success() {
            println!(
                "Failed to execute \"{}\" for \"{}\". Error: {}",
                &command,
                &root,
                String::from_utf8_lossy(&output_unwrapped.stderr)
            );
            continue;
        }

        println!("Executed \"{}\" in \"{}\"", &command, &root);
        println!("Output:\n{}", String::from_utf8_lossy(&output_unwrapped.stdout));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clone;
    // use tempfile::TempDir;
    use std::collections::HashMap;
    use crate::application::test_application;

    const TEST_REPO_URL: &str = "https://github.com/tcione/test-repo.git";

    fn tree_branch(tree_path: &PathBuf) -> Result<String> {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(tree_path)
            .args(["branch", "--show-current"])
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get current branch: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    #[test]
    fn test_create_worktree_success() {
        let application = test_application(vec![], vec![], HashMap::new());
        let tree_path = application.trees_dir.join("test-repo--feature--new-feature");

        clone::run(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        run(&application, "test-repo", "feature/new-feature").unwrap();

        let tree_branch = tree_branch(&tree_path).unwrap();

        assert_eq!(tree_branch, "feature/new-feature".to_string())
    }

    #[test]
    fn test_tree_name() {
        assert_eq!(
            tree_name("myrepo", "feature/normal"),
            "myrepo--feature--normal"
        );
        assert_eq!(
            tree_name("myrepo", "hotfix@at-you"),
            "myrepo--hotfix--at-you"
        );
        assert_eq!(
            tree_name("myrepo", "hotfix/@slash-at-you"),
            "myrepo--hotfix--slash-at-you"
        );
        assert_eq!(
            tree_name("myrepo", "feat/user-mgmt_new"),
            "myrepo--feat--user-mgmt_new"
        );
        assert_eq!(
            tree_name("myrepo", "feat//too-many-hyphens"),
            "myrepo--feat--too-many-hyphens"
        );
        assert_eq!(
            tree_name("myrepo", "feat/////way-too-many-hyphens"),
            "myrepo--feat--way-too-many-hyphens"
        );
        assert_eq!(
            tree_name("myrepo", "        feat/trimmed   "),
            "myrepo--feat--trimmed"
        );
    }

    #[test]
    fn test_create_with_nonexistent_repo() {
        let application = test_application(vec![], vec![], HashMap::new());
        let err = run(&application, "nonexistent-repo", "feature/test").unwrap_err();

        assert!(err.to_string().contains("No such file or directory"))
    }

    #[test]
    fn test_duplicate_branch_name() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::run(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        run(&application, "test-repo", "feature/new-feature").unwrap();
        let err = run(&application, "test-repo", "feature/new-feature").unwrap_err();

        assert!(
            err.to_string()
                .contains("a branch named 'feature/new-feature' already exists")
        )
    }
}
