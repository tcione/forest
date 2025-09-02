use std::path::PathBuf;

pub fn run(roots_dir: &PathBuf, repository_address: String) -> Result<(), Box<dyn std::error::Error>> {
    let gitless_repo_address = repository_address.replace(".git", "");
    let repo_name = gitless_repo_address.split('/').last().unwrap();
    let repo_dir = roots_dir.join(repo_name);
    let output = std::process::Command::new("git")
        .args([
            "clone",
            "--bare",
            &repository_address
        ])
        .arg(&repo_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git clone failed: {}", stderr).into());
    }

    println!("{} cloned into {}", repo_name, repo_dir.display());
    Ok(())
}
