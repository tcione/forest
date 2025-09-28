use anyhow::Result;
use std::path::Path;

use super::Root;

pub fn call(roots_dir: &Path, root: &str) -> Result<Root> {
    let root_dir = roots_dir.join(&root);

    if !root_dir.exists() {
        anyhow::bail!("Root '{}' does not exist", root);
    }

    Ok(Root {
        name: root.to_string(),
        path: root_dir,
    })
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

        let result = call(&roots_dir, "test-repo").unwrap();
        assert_eq!(result.name, "test-repo");
        assert_eq!(result.path, root_dir);
    }

    #[test]
    fn test_enter_nonexistent_root() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();

        let result = call(&roots_dir, "nonexistent-repo");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Root 'nonexistent-repo' does not exist")
        );
    }
}
