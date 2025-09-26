use anyhow::{Context, Result};

use crate::application::Application;
use crate::trees::list::call as list_call;
use crate::utils::exec::call as exec_call;

// TODO: Handle clone better
pub fn call(application: &Application, root: String, tree: String, command: String) -> Result<()> {
    let trees = list_call(application, &Some(root.clone())).context("Failed to list trees")?;

    let root_trees = trees
        .get(&root)
        .with_context(|| format!("Root '{}' not found", root))?;

    let found_tree = root_trees
        .iter()
        .find(|t| t.name == tree || t.branch == tree);

    match found_tree {
        Some(t) => exec_call(&t.path, &command)
            .with_context(|| format!("Failed to execute '{}' in tree '{}'", command, tree)),
        None => anyhow::bail!("Tree '{}' not found in root '{}'", tree, root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_application;
    use crate::roots::clone;
    use crate::trees::create;
    use std::collections::HashMap;
    use std::fs::read_to_string;

    const TEST_REPO_URL: &str = "https://github.com/tcione/test-repo.git";

    #[test]
    fn test_root_does_not_exist() {
        let application = test_application(vec![], vec![], HashMap::new());

        let result = call(
            &application,
            "nonexistent-root".to_string(),
            "some-tree".to_string(),
            "echo test".to_string(),
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Root 'nonexistent-root' not found")
        );
    }

    #[test]
    fn test_tree_does_not_exist_empty_trees() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();

        let result = call(
            &application,
            "test-repo".to_string(),
            "nonexistent-tree".to_string(),
            "echo test".to_string(),
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Tree 'nonexistent-tree' not found in root 'test-repo'")
        );
    }

    #[test]
    fn test_tree_does_not_exist_nonempty_trees() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        create::call(&application, "test-repo", "other-tree").unwrap();

        let result = call(
            &application,
            "test-repo".to_string(),
            "nonexistent-tree".to_string(),
            "echo test".to_string(),
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Tree 'nonexistent-tree' not found in root 'test-repo'")
        );
    }

    #[test]
    fn test_tree_exists() {
        let application = test_application(vec![], vec![], HashMap::new());

        clone::call(&application.roots_dir, TEST_REPO_URL.to_string()).unwrap();
        create::call(&application, "test-repo", "feature").unwrap();

        let tree_dir = application.trees_dir.join("test-repo--feature");
        let result = call(
            &application,
            "test-repo".to_string(),
            "feature".to_string(),
            "echo testing > testfile.txt".to_string(),
        );

        assert!(result.is_ok());
        assert!(tree_dir.join("testfile.txt").exists());
        assert_eq!(
            read_to_string(tree_dir.join("testfile.txt")).unwrap(),
            "testing\n"
        );
    }
}
