use super::Tree;
use crate::application::Application;
use crate::trees::list::call as list_call;
use anyhow::{Context, Result};

// TODO: bind this to forest init (gen bash/zsh/fish functions)
// TODO: Should I return Option instead?
pub fn call(application: &Application, root: String, tree: String) -> Result<Tree> {
    let trees = list_call(application, &Some(root.clone())).context("Failed to list trees")?;

    let root_trees = trees
        .get(&root)
        .with_context(|| format!("Root '{}' not found", root))?;

    let found_tree = root_trees
        .iter()
        .find(|t| t.name == tree || t.branch == tree);

    match found_tree {
        Some(t) => Ok(t.clone()),
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

    const TEST_REPO_URL: &str = "https://github.com/tcione/test-repo.git";

    #[test]
    fn test_root_does_not_exist() {
        let application = test_application(vec![], vec![], HashMap::new());

        let result = call(
            &application,
            "nonexistent-root".to_string(),
            "some-tree".to_string(),
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

        let result = call(&application, "test-repo".to_string(), "feature".to_string());

        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.branch, "feature");
        assert!(tree.path.ends_with("test-repo--feature"));
    }
}
