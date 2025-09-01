use std::path::PathBuf;

const PROJECT_NAME: &str = "forest";

pub fn home_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base_dirs = directories::BaseDirs::new().ok_or("Could not determine home directory")?;
    let home = base_dirs.home_dir();

    Ok(home.to_path_buf())
}

pub fn config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dirs = directories::ProjectDirs::from("", "", PROJECT_NAME)
        .ok_or("Could not find config directory")?;

    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(&config_dir)?;

    Ok(config_dir.to_path_buf())
}
