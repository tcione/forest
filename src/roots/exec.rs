use anyhow::Result;
use console::style;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("Command '{command}' failed with exit code {code}")]
    CommandFailed {
        command: String,
        code: i32,
    },
    #[error("Failed to execute: {0}")]
    IoError(#[from] std::io::Error),
}

// TODO: Background execution
pub fn call(roots_dir: &PathBuf, root: String, command: String) -> Result<(), ExecError> {
    let root_dir = roots_dir.join(&root);

    let start = format!(">> command: {}", &command);
    println!("{}", style(start).dim());

    let status = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(root_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        eprintln!("{}", style("<< command error").dim().red());
        return Err(ExecError::CommandFailed {
            command,
            code: status.code().unwrap_or(-1),
        })
    }

    println!("{}", style("<< command done").dim());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, read_to_string};
    use tempfile::TempDir;

    #[test]
    fn test_roots_exec_with_valid_command() {
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

    #[test]
    fn test_roots_exec_with_invalid_command() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();
        let root_dir = roots_dir.join("test-repo").to_path_buf();

        create_dir_all(&root_dir).unwrap();

        let output = call(
            &roots_dir,
            "test-repo".to_string(),
            "nosuchcommand".to_string(),
        );

        assert!(output.is_err());
        assert!(
            output
                .unwrap_err()
                .to_string()
                .contains("command not found")
        );
    }
}
