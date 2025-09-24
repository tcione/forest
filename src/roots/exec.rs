use anyhow::Result;
use std::path::PathBuf;
use crate::utils::exec::{ExecError, call as exec_call};

pub fn call(roots_dir: &PathBuf, root: String, command: String) -> Result<(), ExecError> {
    let root_dir = roots_dir.join(&root);
    exec_call(&root_dir, &command)
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
            "test-repo".to_string(),
            "echo testing > testfile.txt".to_string(),
        )
        .unwrap();

        assert!(root_dir.join("testfile.txt").exists());
        assert_eq!(
            read_to_string(root_dir.join("testfile.txt")).unwrap(),
            "testing\n"
        );
    }
}
