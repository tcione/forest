use anyhow::{Context, Result};

use crate::application::Application;
use crate::trees::get::call as get_call;
use crate::utils::exec::call as exec_call;

pub fn call(application: &Application, root: String, tree: String, command: String) -> Result<()> {
    let tree = get_call(application, root, tree)?;

    exec_call(&tree.path, &command).with_context(|| {
        format!(
            "Failed to execute '{}' in tree '{}'",
            command,
            &tree.path.display()
        )
    })
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
    fn test_exec_command_success() {
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
