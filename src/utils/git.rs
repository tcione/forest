use std::path::PathBuf;
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Command '{command}' failed in '{base_dir}': {stderr}")]
    CommandFailed {
        command: String,
        base_dir: String,
        stderr: String,
    },
    #[error("Impossible to identify a default branch")]
    UndefinedDefaultBranch {},
    #[error("Failed to execute: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct GitSuccess {
    pub stdout: String,
}

pub struct Git {
    base_dir: PathBuf,
}

impl Git {
    pub fn new(base_dir: &PathBuf) -> Self {
        Self {
            base_dir: base_dir.clone(),
        }
    }

    pub fn clone(&self, repo_address: &str) -> Result<GitSuccess, GitError> {
        let output = Command::new("git")
            .args(["clone", &repo_address])
            .arg(&self.base_dir)
            .output()?;

        self.parsed_output("clone", output)
    }

    pub fn latest_default(&self) -> Result<GitSuccess, GitError> {
        if self.is_local_only()? {
            return Ok(GitSuccess {
                stdout: "Nop. Local only repo".to_string(),
            });
        }

        let default_branch = self.default_branch()?;
        let output = self
            .based_git()
            .args(["pull", "origin"])
            .arg(default_branch)
            .output()?;

        self.parsed_output("pull", output)
    }

    pub fn list_worktrees(&self) -> Result<GitSuccess, GitError> {
        let output = self
            .based_git()
            .args(["worktree", "list", "--porcelain"])
            .output()?;

        self.parsed_output("worktree-list", output)
    }

    pub fn add_worktree(
        &self,
        new_branch_name: &str,
        target_dir: &PathBuf,
    ) -> Result<GitSuccess, GitError> {
        let default_branch = self.default_branch()?;
        let output = self
            .based_git()
            .args(["worktree", "add"])
            .arg("-b")
            .arg(new_branch_name)
            .arg(target_dir)
            .arg(default_branch)
            .output()?;

        self.parsed_output("worktree-add", output)
    }

    pub fn remove_worktree(&self, target_dir: &PathBuf) -> Result<GitSuccess, GitError> {
        let output = self
            .based_git()
            .args(["worktree", "remove"])
            .arg(target_dir)
            .output()?;

        self.parsed_output("worktree-remove", output)
    }

    pub fn delete_branch(&self, branch_name: &str) -> Result<GitSuccess, GitError> {
        let output = self
            .based_git()
            .args(["branch", "-D"])
            .arg(branch_name)
            .output()?;

        self.parsed_output("branch-delete", output)
    }

    fn is_local_only(&self) -> Result<bool, GitError> {
        let output = self.based_git().arg("remote").output()?;

        let p_output = self.parsed_output("remote", output)?;

        Ok(p_output.stdout.trim().is_empty())
    }

    pub fn default_branch(&self) -> Result<String, GitError> {
        if self.is_local_only()? {
            let raw_branches = self.based_git().args(["branch", "--list"]).output()?;
            let p_branches = self.parsed_output("branch", raw_branches)?;
            let branches: Vec<&str> = p_branches
                .stdout
                .lines()
                .map(|line| line.trim_start_matches("* ").trim())
                .collect();

            for default in ["main", "master"] {
                if branches.contains(&default) {
                    return Ok(default.to_string());
                }
            }

            return Err(GitError::UndefinedDefaultBranch {});
        }

        let output = self
            .based_git()
            .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
            .output()?;
        let p_output = self.parsed_output("symbolic-ref", output)?;
        let filtered_branch = p_output.stdout.trim().strip_prefix("refs/remotes/origin/");

        if let Some(branch) = filtered_branch {
            return Ok(branch.to_string());
        }

        Err(GitError::UndefinedDefaultBranch {})
    }

    fn based_git(&self) -> Command {
        let mut cmd = Command::new("git");
        cmd.arg("-C");
        cmd.arg(&self.base_dir);
        cmd
    }

    fn parsed_output(&self, command: &str, output: Output) -> Result<GitSuccess, GitError> {
        if !output.status.success() {
            return Err(GitError::CommandFailed {
                command: command.to_string(),
                base_dir: self.base_dir.to_string_lossy().to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(GitSuccess {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::process::ExitStatusExt;
    use tempfile::TempDir;

    const TEST_REPO_ADDRESS: &str = "https://github.com/tcione/test-repo.git";

    fn setup_git_repo_with_branch(temp_dir: &TempDir, branch_name: &str) -> PathBuf {
        let repo_path = temp_dir.path().to_path_buf();

        std::process::Command::new("git")
            .args(["init", "-b", branch_name])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to init git repo");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to configure git user");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to configure git email");

        // Create initial commit
        fs::write(repo_path.join("README.md"), "# Test Repo").expect("Failed to create README");
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to add README");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to commit");

        repo_path
    }

    #[test]
    fn test_git_new() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        let git = Git::new(&repo_path);

        assert_eq!(git.base_dir, repo_path);
    }

    #[test]
    fn test_clone_success() {
        let temp_dir = TempDir::new().unwrap();
        let clone_target = temp_dir.path().join("test-repo");
        let git = Git::new(&clone_target);
        git.clone(TEST_REPO_ADDRESS).unwrap();

        assert!(clone_target.exists());
        assert!(clone_target.join(".git").exists());
        assert!(clone_target.join("README.md").exists());
    }

    #[test]
    fn test_clone_invalid_repo() {
        let temp_dir = TempDir::new().unwrap();
        let git = Git::new(&temp_dir.path().to_path_buf());
        let result = git.clone("invalid-repo-url");

        assert!(result.is_err());
    }

    #[test]
    fn test_latest_default_no_remotes() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let git = Git::new(&repo_path);
        let result = git.latest_default().unwrap();

        assert_eq!(result.stdout, "Nop. Local only repo".to_string());
    }

    #[test]
    fn test_latest_default_remote() {
        let temp_dir = TempDir::new().unwrap();
        let parent_dir = temp_dir.path().to_path_buf();
        let clone_target = parent_dir.join("test-repo");
        let git = Git::new(&clone_target);

        git.clone(TEST_REPO_ADDRESS).unwrap();

        let git_cloned = Git::new(&clone_target);
        let result = git_cloned.latest_default().unwrap();

        assert!(result.stdout.contains("Already up to date"));
    }

    #[test]
    fn test_default_branch_local_main() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let git = Git::new(&repo_path);
        let result = git.default_branch().unwrap();

        assert_eq!(result, "main");
    }

    #[test]
    fn test_default_branch_local_master() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "master");
        let git = Git::new(&repo_path);
        let result = git.default_branch().unwrap();

        assert_eq!(result, "master");
    }

    #[test]
    fn test_default_branch_local_unconventional() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "unconventional-main-branch");
        let git = Git::new(&repo_path);
        let result = git.default_branch();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GitError::UndefinedDefaultBranch {}
        ));
    }

    #[test]
    fn test_default_branch_remote() {
        let temp_dir = TempDir::new().unwrap();
        let parent_dir = temp_dir.path().to_path_buf();
        let clone_target = parent_dir.join("test-repo");
        let git = Git::new(&clone_target);

        git.clone(TEST_REPO_ADDRESS).unwrap();

        let git_cloned = Git::new(&clone_target);
        let result = git_cloned.default_branch().unwrap();

        assert!(result == "main");
    }

    #[test]
    fn test_add_worktree_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let worktree_path = temp_dir.path().join("feature-branch");
        let git = Git::new(&repo_path);
        let result = git.add_worktree("feature-branch", &worktree_path);

        assert!(result.is_ok());
        assert!(worktree_path.exists());
        assert!(worktree_path.join(".git").exists());
    }

    #[test]
    fn test_add_worktree_duplicate_branch() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let worktree_path1 = temp_dir.path().join("feature-branch1");
        let worktree_path2 = temp_dir.path().join("feature-branch2");
        let git = Git::new(&repo_path);

        git.add_worktree("duplicate-branch", &worktree_path1)
            .unwrap();

        let result = git.add_worktree("duplicate-branch", &worktree_path2);

        assert!(result.is_err());
        if let Err(GitError::CommandFailed { stderr, .. }) = result {
            assert!(stderr.contains("already exists"));
        }
    }

    #[test]
    fn test_based_git_returns_command() {
        let temp_dir = TempDir::new().unwrap();
        let git = Git::new(&temp_dir.path().to_path_buf());
        let mut cmd = git.based_git();

        cmd.arg("status");

        let result = cmd.output();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parsed_output_success() {
        let temp_dir = TempDir::new().unwrap();
        let git = Git::new(&temp_dir.path().to_path_buf());
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"test output".to_vec(),
            stderr: b"".to_vec(),
        };
        let result = git.parsed_output("test", output).unwrap();

        assert_eq!(result.stdout, "test output");
    }

    #[test]
    fn test_parsed_output_failure() {
        let temp_dir = TempDir::new().unwrap();
        let git = Git::new(&temp_dir.path().to_path_buf());
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(256),
            stdout: b"".to_vec(),
            stderr: b"error message".to_vec(),
        };
        let result = git.parsed_output("test", output);

        assert!(result.is_err());
        if let Err(GitError::CommandFailed {
            command, stderr, ..
        }) = result
        {
            assert_eq!(command, "test");
            assert_eq!(stderr, "error message");
        }
    }

    #[test]
    fn test_remove_worktree_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let worktree_path = temp_dir.path().join("feature-branch");
        let git = Git::new(&repo_path);

        git.add_worktree("feature-branch", &worktree_path).unwrap();
        assert!(worktree_path.exists());

        let result = git.remove_worktree(&worktree_path);

        assert!(result.is_ok());
        assert!(!worktree_path.exists());
    }

    #[test]
    fn test_remove_worktree_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let nonexistent_path = temp_dir.path().join("nonexistent-worktree");
        let git = Git::new(&repo_path);
        let result = git.remove_worktree(&nonexistent_path);

        assert!(result.is_err());
        if let Err(GitError::CommandFailed { stderr, .. }) = result {
            assert!(stderr.contains("not a working tree"));
        }
    }

    #[test]
    fn test_delete_branch_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let git = Git::new(&repo_path);

        std::process::Command::new("git")
            .args(["checkout", "-b", "test-branch"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to create branch");

        std::process::Command::new("git")
            .args(["checkout", "main"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to switch to main");

        let result = git.delete_branch("test-branch");

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_branch_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let git = Git::new(&repo_path);
        let result = git.delete_branch("nonexistent-branch");

        assert!(result.is_err());
        if let Err(GitError::CommandFailed { stderr, .. }) = result {
            assert!(stderr.contains("not found"));
        }
    }

    #[test]
    fn test_delete_branch_current() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = setup_git_repo_with_branch(&temp_dir, "main");
        let git = Git::new(&repo_path);
        let result = git.delete_branch("main");

        println!("{:?}", result);

        assert!(result.is_err());
        if let Err(GitError::CommandFailed { stderr, .. }) = result {
            assert!(stderr.contains("cannot delete branch"));
        }
    }
}
