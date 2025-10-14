use anyhow::Result;
use std::path::Path;

use super::Root;
use super::fuzzy_selector::call as fuzzy_selector_call;

pub fn call(roots_dir: &Path, root: Option<String>) -> Result<Root> {
    match root {
        Some(root_name) => {
            let root_dir = roots_dir.join(&root_name);
            if !root_dir.exists() {
                anyhow::bail!("Root '{}' does not exist", root_name);
            }

            Ok(Root {
                name: root_name,
                path: root_dir,
            })
        },
        None => {
            let root_struct = fuzzy_selector_call(roots_dir, root)?;
            Ok(root_struct)
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::TempDir;

    #[test]
    fn test_enter_existing_root() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();
        let root_dir = roots_dir.join("test-repo");

        create_dir_all(&root_dir).unwrap();

        let result = call(&roots_dir, Some("test-repo".to_string())).unwrap();
        assert_eq!(result.name, "test-repo");
        assert_eq!(result.path, root_dir);
    }

    #[test]
    fn test_enter_nonexistent_root() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();

        let result = call(&roots_dir, Some("nonexistent-repo".to_string()));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Root 'nonexistent-repo' does not exist")
        );
    }
}
