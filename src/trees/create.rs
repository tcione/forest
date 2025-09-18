use anyhow::{Context, Result};
use regex::Regex;
use std::path::PathBuf;

use crate::application::Application;

// TODO: root completion
// TODO: Handle already exists by using goto
// TODO: Handle exec in background and notify user using notify-rust
//       For that I also need to have proper logs somewhere
pub fn call(application: &Application, root: &str, new_branch_name: &str) -> Result<()> {
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

// TODO: Fix so this is more reliable. What happens if there's no remote?
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

// TODO: Fix so this is more reliable. What happens if there's no remote?
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
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;
    use crate::roots::clone;
    use crate::application::test_application;
    use crate::utils::config::RootConfig;

    const TEST_REPO_URL: &str = "https://github.com/tcione/test-repo.git";

    // Utils
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

    // End-to-end
    #[test]
    fn test_create_worktree_success() {
        let application = test_application(
            vec![".env".to_string()],
            vec!["echo 'setup complete' > setup_via_exec.txt".to_string()],
            HashMap::new()
        );
        let tree_path = application.trees_dir.join("test-repo--feature--new-feature");

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        fs::write(&application.roots_dir.join("test-repo").join(".env"), "VAR=test").unwrap();

        call(&application, "test-repo", "feature/new-feature").unwrap();

        let tree_branch = tree_branch(&tree_path).unwrap();

        assert_eq!(tree_branch, "feature/new-feature".to_string());
        assert!(tree_path.join(".env").exists());
        assert!(tree_path.join("setup_via_exec.txt").exists());
    }

    #[test]
    fn test_create_with_nonexistent_repo() {
        let application = test_application(vec![], vec![], HashMap::new());
        let err = call(&application, "nonexistent-repo", "feature/test").unwrap_err();

        assert!(err.to_string().contains("No such file or directory"))
    }

    #[test]
    fn test_duplicate_branch_name() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        call(&application, "test-repo", "feature/new-feature").unwrap();
        let err = call(&application, "test-repo", "feature/new-feature").unwrap_err();

        assert!(
            err.to_string()
                .contains("a branch named 'feature/new-feature' already exists")
        )
    }

    // Unit
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
    fn test_copy_files_with_empty_list() {
        let repo_root = TempDir::new().unwrap();
        let branch_tree = TempDir::new().unwrap();
        let empty_copy_list = vec![];

        copy_files(
            "test-repo",
            &repo_root.path().to_path_buf(),
            &branch_tree.path().to_path_buf(),
            &empty_copy_list
        );

        assert_eq!(branch_tree.path().read_dir().unwrap().count(), 0);
        // TODO: Augment checking logs once I implement logging
    }

    #[test]
    fn test_copy_files_filled_list() {
        let repo_root = TempDir::new().unwrap();
        let branch_tree = TempDir::new().unwrap();

        fs::write(repo_root.path().join("file1.txt"), "content1").unwrap();
        fs::write(repo_root.path().join("file2.txt"), "content2").unwrap();

        let copy_list = vec![
            "file1.txt".to_string(),
            "nonexistent.txt".to_string(),
            "file2.txt".to_string()
        ];

        copy_files(
            "test-repo",
            &repo_root.path().to_path_buf(),
            &branch_tree.path().to_path_buf(),
            &copy_list
        );

        assert!(branch_tree.path().join("file1.txt").exists());
        assert!(branch_tree.path().join("file2.txt").exists());
        assert_eq!(fs::read_to_string(branch_tree.path().join("file1.txt")).unwrap(), "content1");
        assert_eq!(fs::read_to_string(branch_tree.path().join("file2.txt")).unwrap(), "content2");
        assert!(!branch_tree.path().join("nonexistent.txt").exists());
        // TODO: Augment checking logs once I implement logging
    }

    #[test]
    fn test_exec_commands_with_empty_list() {
        let branch_tree = TempDir::new().unwrap();
        let empty_exec_list = vec![];

        exec_commands(
            "test-repo",
            &branch_tree.path().to_path_buf(),
            &empty_exec_list
        );

        // Function completes without panicking - that's the test
        // TODO: Augment checking logs once I implement logging
    }

    #[test]
    fn test_exec_commands_with_existing_commands() {
        let branch_tree = TempDir::new().unwrap();
        let exec_list = vec![
            "echo 'test output' > output.txt".to_string(),
            "ls".to_string()
        ];

        exec_commands(
            "test-repo",
            &branch_tree.path().to_path_buf(),
            &exec_list
        );

        // Verify first command created the file
        assert!(branch_tree.path().join("output.txt").exists());
        assert_eq!(
            fs::read_to_string(branch_tree.path().join("output.txt")).unwrap().trim(),
            "test output"
        );
        // TODO: Augment checking logs once I implement logging
    }

    #[test]
    fn test_exec_commands_with_non_existing_commands() {
        let branch_tree = TempDir::new().unwrap();
        let exec_list = vec![
            "nonexistent_command".to_string(),
            "echo 'still works' > success.txt".to_string()
        ];

        exec_commands(
            "test-repo",
            &branch_tree.path().to_path_buf(),
            &exec_list
        );

        assert!(branch_tree.path().join("success.txt").exists());
        assert_eq!(
            fs::read_to_string(branch_tree.path().join("success.txt")).unwrap().trim(),
            "still works"
        );
        // TODO: Augment checking logs once I implement logging
    }

    #[test]
    fn test_set_up_worktree_no_root_config() {
        let application = test_application(
            vec!["general_file.txt".to_string()],
            vec!["echo 'general command' > general_output.txt".to_string()],
            HashMap::new()
        );
        let repo_root = TempDir::new().unwrap();
        let branch_tree = TempDir::new().unwrap();

        fs::write(repo_root.path().join("general_file.txt"), "general content").unwrap();

        set_up_worktree(
            &application,
            "test-repo",
            &repo_root.path().to_path_buf(),
            &branch_tree.path().to_path_buf()
        ).unwrap();

        assert!(branch_tree.path().join("general_file.txt").exists());
        assert_eq!(
            fs::read_to_string(branch_tree.path().join("general_file.txt")).unwrap(),
            "general content"
        );
        assert!(branch_tree.path().join("general_output.txt").exists());
        assert_eq!(
            fs::read_to_string(branch_tree.path().join("general_output.txt")).unwrap().trim(),
            "general command"
        );
    }

    #[test]
    fn test_set_up_worktree_with_root_config() {
        let mut root_configs = HashMap::new();
        root_configs.insert("test-repo".to_string(), RootConfig {
            copy: vec!["root_file.txt".to_string()],
            exec: vec!["echo 'root command' > root_output.txt".to_string()],
        });

        let application = test_application(
            vec!["general_file.txt".to_string()],
            vec!["echo 'general command' > general_output.txt".to_string()],
            root_configs
        );
        let repo_root = TempDir::new().unwrap();
        let branch_tree = TempDir::new().unwrap();

        fs::write(repo_root.path().join("general_file.txt"), "general content").unwrap();
        fs::write(repo_root.path().join("root_file.txt"), "root content").unwrap();

        set_up_worktree(
            &application,
            "test-repo",
            &repo_root.path().to_path_buf(),
            &branch_tree.path().to_path_buf()
        ).unwrap();

        assert!(branch_tree.path().join("root_file.txt").exists());
        assert!(!branch_tree.path().join("general_file.txt").exists());
        assert!(branch_tree.path().join("root_output.txt").exists());
        assert!(!branch_tree.path().join("general_output.txt").exists());
    }
}
