use anyhow::{Context, Result};
use std::path::PathBuf;

use super::{Tree, Trees, RootsTrees};
use crate::application::Application;
use crate::roots;
use crate::utils::git::Git;

pub fn call(application: &Application, root: &Option<String>) -> Result<RootsTrees> {
    let roots = roots::list::call(&application.roots_dir).context("Failed to list roots")?;

    let filtered_roots = if let Some(given_root) = root {
        roots
            .into_iter()
            .filter(|current_root| &current_root.name == given_root)
            .collect::<Vec<roots::Root>>()
    } else {
        roots
    };

    let mut trees = RootsTrees::new();
    for root in filtered_roots {
        let git = Git::new(&root.path);
        let git_root_trees = git_root_trees(&root)?;
        let default_branch = git.default_branch()?;
        let root_trees = root_trees(git_root_trees, &default_branch)?;
        trees.insert(root.name.clone(), root_trees);
    }

    Ok(trees)
}

fn git_root_trees(root: &roots::Root) -> Result<String> {
    match Git::new(&root.path).list_worktrees() {
        Ok(success) => Ok(success.stdout),
        Err(_) => Ok(String::from("")),
    }
}

fn root_trees(raw_trees: String, default_branch: &str) -> Result<Trees> {
    if raw_trees.trim().is_empty() {
        return Ok(vec![]);
    }

    let root_trees: Trees = raw_trees
        .split("\n\n")
        .filter_map(|chunk| {
            if chunk.is_empty() {
                return None;
            }

            let lines: Vec<&str> = chunk.lines().collect();
            let path = lines
                .get(0)
                .and_then(|line| line.strip_prefix("worktree "))?;
            let head = lines.get(1).and_then(|line| line.strip_prefix("HEAD "))?;
            let branch = lines
                .get(2)
                .and_then(|line| line.strip_prefix("branch refs/heads/"))?;
            let name = path.split("/").last().unwrap_or("undefined");

            if branch == default_branch {
                return None;
            }

            Some(Tree {
                name: name.to_string(),
                path: PathBuf::from(path),
                branch: branch.to_string(),
                head: head.to_string(),
            })
        })
        .collect();

    Ok(root_trees)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::test_application;
    use std::collections::HashMap;
    use std::fs::create_dir_all;
    use tempfile::TempDir;

    fn setup_repo(path: &PathBuf) {
        create_dir_all(path).unwrap();

        std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(path)
            .output()
            .unwrap();
        std::fs::write(path.join("README.md"), "test").unwrap();
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(path)
            .output()
            .unwrap();
    }

    fn add_worktree(root_path: &PathBuf, tree_path: &PathBuf, branch: &str) {
        std::process::Command::new("git")
            .arg("-C")
            .arg(root_path)
            .args([
                "worktree",
                "add",
                "-b",
                branch,
            ])
            .arg(tree_path)
            .output()
            .unwrap();
    }

    #[test]
    fn test_run_without_root_filter() {
        let application = test_application(vec![], vec![], HashMap::new());
        let root1_path = application.roots_dir.join("repo1");
        let root1_worktree_path = application.trees_dir.join("repo1--feature");
        let root2_path = application.roots_dir.join("repo2");
        let root2_worktree_path = application.trees_dir.join("repo2--fix--a-bug");

        setup_repo(&root1_path);
        add_worktree(&root1_path, &root1_worktree_path, "feature");
        setup_repo(&root2_path);
        add_worktree(&root2_path, &root2_worktree_path, "fix/a-bug");

        let result = call(&application, &None).unwrap();

        assert_eq!(result.len(), 2);

        assert_eq!(result["repo1"].len(), 1);
        assert_eq!(result["repo1"][0].name, "repo1--feature");
        assert!(result["repo1"][0].path.to_string_lossy().ends_with("trees/repo1--feature"));
        assert_eq!(result["repo1"][0].branch, "feature");
        assert!(!result["repo1"][0].head.is_empty());

        assert_eq!(result["repo2"].len(), 1);
        assert_eq!(result["repo2"][0].name, "repo2--fix--a-bug");
        assert!(result["repo2"][0].path.to_string_lossy().ends_with("trees/repo2--fix--a-bug"));
        assert_eq!(result["repo2"][0].branch, "fix/a-bug");
        assert!(!result["repo2"][0].head.is_empty());
    }

    #[test]
    fn test_run_with_root_filter() {
        let application = test_application(vec![], vec![], HashMap::new());

        let root1_path = application.roots_dir.join("repo1");
        let root2_path = application.roots_dir.join("repo2");
        create_dir_all(&root2_path).unwrap();

        setup_repo(&root1_path);
        setup_repo(&root2_path);

        let result = call(&application, &Some("repo1".to_string())).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result["repo1"].len(), 0);
        assert!(!result.contains_key("repo2"));
    }

    #[test]
    fn test_git_root_trees_no_trees() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        create_dir_all(&repo_path).unwrap();

        setup_repo(&repo_path);

        let root = roots::Root {
            name: "test-repo".to_string(),
            path: repo_path,
        };

        let result = git_root_trees(&root).unwrap();

        assert_eq!(result.matches("worktree").count(), 1);
    }

    #[test]
    fn test_git_root_trees_one_tree() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        let trees_path = temp_dir.path().join("trees");
        let worktree_path = trees_path.join("test-repo--feature");

        setup_repo(&repo_path);
        add_worktree(&repo_path, &worktree_path, "feature");

        let root = roots::Root {
            name: "test-repo".to_string(),
            path: repo_path,
        };

        let result = git_root_trees(&root).unwrap();
        println!("{:?}", result);

        assert_eq!(result.matches("worktree").count(), 2);
        assert_eq!(result.matches("test-repo--feature").count(), 1);
    }

    #[test]
    fn test_root_trees_empty_string() {
        let result = root_trees(String::from(""), "main").unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_root_trees_empty_string_with_white_space() {
        let result = root_trees(String::from("    \n  \t  "), "main").unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_root_trees_with_worktrees() {
        let input = concat!(
            "worktree /path/to/trees/repo--main\n",
            "HEAD abc123def456\n",
            "branch refs/heads/main\n",
            "\n",
            "worktree /path/to/trees/repo--feature--ui\n",
            "HEAD 789ghi012jkl\n",
            "branch refs/heads/feature/ui\n"
        )
        .to_string();

        let result = root_trees(input, "main").unwrap();

        assert_eq!(result.len(), 1);

        let tree = &result[0];
        assert_eq!(tree.name, "repo--feature--ui");
        assert_eq!(
            tree.path,
            PathBuf::from("/path/to/trees/repo--feature--ui")
        );
        assert_eq!(tree.branch, "feature/ui");
        assert_eq!(tree.head, "789ghi012jkl");
    }
}
