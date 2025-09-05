use std::path::PathBuf;
use anyhow::{Result, Context};

// TODO
// - Handle github:org/repo
pub fn run(roots_dir: &PathBuf, repository_address: String) -> Result<()> {
    let gitless_repo_address = repository_address.replace(".git", "");
    let repo_name = gitless_repo_address.split('/').last().context("Invalid repository URL")?;
    let repo_dir = roots_dir.join(repo_name);

    let output = std::process::Command::new("git")
        .args(["clone", &repository_address])
        .arg(&repo_dir)
        .output()
        .context("Failed to run git command")?;

    if !output.status.success() {
        anyhow::bail!("Git clone failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    println!("{} cloned into {}", repo_name, repo_dir.display());
    Ok(())
}

//

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const REPO_ADDRESS: &str = "https://github.com/tcione/test-repo.git";

    #[test]
    fn test_clone_repo() {
        let roots_dir = tempfile::TempDir::new().unwrap();
        let cloned_path = roots_dir.path().join("test-repo");
        let git_path = cloned_path.join(".git");

        run(&roots_dir.path().to_path_buf(), REPO_ADDRESS.to_string()).unwrap();

        assert!(cloned_path.exists());
        assert!(git_path.exists());
    }

    #[test]
    fn test_clone_with_invalid_url() {
        let roots_dir = TempDir::new().unwrap();
        let result = run(&roots_dir.path().to_path_buf(), "invalid-url".to_string());
        let err = result.unwrap_err();

        assert!(err.to_string().contains("repository 'invalid-url' does not exist"))
    }
}
