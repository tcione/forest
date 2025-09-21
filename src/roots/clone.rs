use std::path::PathBuf;
use anyhow::{Result, Context};

use super::Root;

use crate::utils::git::Git;

// TODO: Handle github:org/repo
pub fn call(roots_dir: &PathBuf, repository_address: String) -> Result<Root> {
    let gitless_repo_address = repository_address.replace(".git", "");
    let repo_name = gitless_repo_address.split('/').last().context("Invalid repository URL")?;
    let repo_dir = roots_dir.join(repo_name);

    Git::new(&repo_dir).clone(&repository_address)?;

    Ok(Root {
        name: repo_name.to_string(),
        path: repo_dir,
    })
}

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

        let result = call(&roots_dir.path().to_path_buf(), REPO_ADDRESS.to_string()).unwrap();

        assert!(cloned_path.exists());
        assert!(git_path.exists());
        assert_eq!(result.name, "test-repo");
        assert_eq!(result.path, cloned_path);
    }

    #[test]
    fn test_clone_with_invalid_url() {
        let roots_dir = TempDir::new().unwrap();
        let result = call(&roots_dir.path().to_path_buf(), "invalid-url".to_string());
        let err = result.unwrap_err();

        assert!(err.to_string().contains("repository 'invalid-url' does not exist"))
    }
}
