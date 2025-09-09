use anyhow::{Result};
use std::path::PathBuf;

pub fn run(roots_dir: &PathBuf, root: String, command: String) -> Result<()> {
    let root_dir = roots_dir.join(&root);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(root_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to execute \"{}\" for \"{}\". Error: {}",
            &command,
            &root,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("Executed \"{}\" in \"{}\"", command, root);
    println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{create_dir_all, read_to_string};

    #[test]
    fn test_roots_exec_with_valid_command() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();
        let root_dir = roots_dir.join("test-repo").to_path_buf();

        create_dir_all(&root_dir).unwrap();

        run(&roots_dir, "test-repo".to_string(), "echo testing > testfile.txt".to_string()).unwrap();

        assert!(root_dir.join("testfile.txt").exists());
        assert_eq!(read_to_string(root_dir.join("testfile.txt")).unwrap(), "testing\n");
    }

    #[test]
    fn test_roots_exec_with_invalid_command() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();
        let root_dir = roots_dir.join("test-repo").to_path_buf();

        create_dir_all(&root_dir).unwrap();

        let output = run(&roots_dir, "test-repo".to_string(), "nosuchcommand".to_string());

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains("command not found"));
    }
}
