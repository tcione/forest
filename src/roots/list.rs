use anyhow::Result;
use std::fs::read_dir;
use std::path::PathBuf;

type Roots = Vec<Root>;
pub struct Root {
    pub name: String,
    pub path: PathBuf,
}

pub fn call(roots_dir: &PathBuf) -> Result<Roots> {
    let mut roots: Roots = vec![];

    for root in read_dir(roots_dir.to_string_lossy().to_string())? {
        let root = root?;
        if !root.file_type()?.is_dir() {
            continue;
        }

        let name = root.file_name().to_string_lossy().to_string();
        let path = root.path();

        roots.push(Root { name, path });
    }

    roots.sort_by_key(|key| key.name.clone());

    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::create_dir_all;

    #[test]
    fn test_list_run_with_empty_directory() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();

        let result = call(&roots_dir);

        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_list_run_with_filled_directory() {
        let roots_tmp_dir = TempDir::new().unwrap();
        let roots_dir = roots_tmp_dir.path().to_path_buf();

        let root_dir1 = roots_dir.join("test-repo1").to_path_buf();
        let root_dir2 = roots_dir.join("test-repo2").to_path_buf();

        create_dir_all(&root_dir1).unwrap();
        create_dir_all(&root_dir2).unwrap();

        let result = call(&roots_dir).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "test-repo1");
        assert_eq!(result[0].path, root_dir1);
        assert_eq!(result[1].name, "test-repo2");
        assert_eq!(result[1].path, root_dir2);
    }

    #[test]
    fn test_list_run_with_nonexistent_directory() {
        let nonexistent_dir = PathBuf::from("/path/that/does/not/exist");

        let result = call(&nonexistent_dir);

        assert!(result.is_err());
    }
}
