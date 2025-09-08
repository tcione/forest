use anyhow::Result;
use std::fs::read_dir;
use std::path::PathBuf;

pub fn run(roots_dir: &PathBuf) -> Result<()> {
    let mut has_projects = false;

    for root in read_dir(roots_dir.to_string_lossy().to_string())? {
        let root = root?;
        if !root.file_type()?.is_dir() {
            continue;
        }

        has_projects = true;
        let project = root.file_name();
        let path = root.path();

        println!(
            "{} -> {}",
            project.to_string_lossy(),
            path.to_string_lossy(),
        );
    }

    if !has_projects {
        println!("No roots yet...");
    }

    Ok(())
}

// TODO: Write tests once I figure out a better output than stdout
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_nonexistent_directory() {
        let nonexistent_dir = PathBuf::from("/path/that/does/not/exist");

        let result = run(&nonexistent_dir);

        assert!(result.is_err());
    }
}
