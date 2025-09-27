use crate::application::Application;
use crate::roots::get::call as root_get_call;
use crate::trees::get::call as get_call;
use crate::utils::git::Git;
use anyhow::{Context, Result};

pub fn call(application: &Application, root: &str, tree: &str) -> Result<()> {
    let root_obj = root_get_call(&application.roots_dir, root)
        .with_context(|| format!("Failed to find root '{}'", root))?;
    let tree_obj = get_call(application, root, tree)
        .with_context(|| format!("Failed to find tree '{}' in root '{}'", tree, root))?;
    let git = Git::new(&root_obj.path);

    git.remove_worktree(&tree_obj.path)
        .with_context(|| format!("Failed to remove worktree at '{}'", tree_obj.path.display()))?;

    git.delete_branch(&tree_obj.branch)
        .with_context(|| format!("Failed to delete branch '{}'", tree_obj.branch))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_application;
    use crate::roots::clone;
    use crate::trees::create;
    use std::collections::HashMap;

    const TEST_REPO_URL: &str = "https://github.com/tcione/test-repo.git";

    #[test]
    fn test_delete_tree_success() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        create::call(&application, "test-repo", "feature-branch").unwrap();

        let tree_path = application.trees_dir.join("test-repo--feature-branch");
        assert!(tree_path.exists());

        let result = call(
            &application,
            "test-repo",
            "feature-branch",
        );

        assert!(result.is_ok());
        assert!(!tree_path.exists());

        let root_path = application.roots_dir.join("test-repo");
        let branch_check = std::process::Command::new("git")
            .arg("-C")
            .arg(&root_path)
            .args(["branch", "--list", "feature-branch"])
            .output()
            .unwrap();
        assert!(!String::from_utf8_lossy(&branch_check.stdout).contains("feature-branch"));
    }

    #[test]
    fn test_delete_nonexistent_tree() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();

        let result = call(
            &application,
            "test-repo",
            "nonexistent-tree",
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to find tree 'nonexistent-tree' in root 'test-repo'")
        );
    }

    #[test]
    fn test_delete_tree_nonexistent_root() {
        let application = test_application(vec![], vec![], HashMap::new());

        let result = call(
            &application,
            "nonexistent-root",
            "some-tree",
        );

        println!("{:?}", result);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to find root 'nonexistent-root'")
        );
    }
}
