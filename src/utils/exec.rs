use anyhow::Result;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

use crate::utils::cli_ui;

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
pub fn call(base_dir: &PathBuf, command: &str) -> Result<(), ExecError> {
    let start = format!(">> command: {}", command);
    println!("{}", cli_ui::context(&start));

    let status = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(base_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        eprintln!("{}", cli_ui::critical("<< command error"));
        return Err(ExecError::CommandFailed {
            command: command.to_string(),
            code: status.code().unwrap_or(-1),
        })
    }

    println!("{}", cli_ui::context("<< command done"));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    use tempfile::TempDir;

    #[test]
    fn test_exec_with_valid_command() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().to_path_buf();

        call(&base_dir, "echo testing > testfile.txt").unwrap();

        assert!(base_dir.join("testfile.txt").exists());
        assert_eq!(
            read_to_string(base_dir.join("testfile.txt")).unwrap(),
            "testing\n"
        );
    }

    #[test]
    fn test_exec_with_invalid_command() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().to_path_buf();

        let result = call(&base_dir, "nosuchcommand");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("nosuchcommand")
        );
    }
}
