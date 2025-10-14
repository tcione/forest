use anyhow::Result;
use std::path::PathBuf;

use crate::utils::exec::call as exec_call;
use super::fuzzy_selector::call as fuzzy_selector_call;

pub fn call(roots_dir: &PathBuf, command: String, root: Option<String>) -> Result<()> {
    let root_struct = fuzzy_selector_call(roots_dir, root)?;
    exec_call(&root_struct.path, &command)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, read_to_string};
    use tempfile::TempDir;

    #[test]
    fn test_calls_exec_with_correct_directory() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();
        let root_dir = roots_dir.join("test-repo").to_path_buf();

        create_dir_all(&root_dir).unwrap();

        call(
            &roots_dir,
            "echo testing > testfile.txt".to_string(),
            Some("test-repo".to_string()),
        )
        .unwrap();

        assert!(root_dir.join("testfile.txt").exists());
        assert_eq!(
            read_to_string(root_dir.join("testfile.txt")).unwrap(),
            "testing\n"
        );
    }
}
