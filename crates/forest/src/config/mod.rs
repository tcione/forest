use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    #[serde(default)]
    pub roots: HashMap<String, RootConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub roots_dir: String,
    pub trees_dir: String,
    #[serde(default)]
    pub copy: Vec<String>,
    #[serde(default)]
    pub exec: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RootConfig {
    #[serde(default)]
    pub copy: Vec<String>,
    #[serde(default)]
    pub exec: Vec<String>,
}

const PROJECT_NAME: &str = "forest";

fn home_dir() -> Result<PathBuf> {
    let base_dirs = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(base_dirs.home_dir().to_path_buf())
}

pub fn config_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", PROJECT_NAME)
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir)?;

    Ok(config_dir.to_path_buf())
}

fn default_copy() -> Vec<String> {
    vec![".env".to_string(), ".envrc".to_string()]
}

pub fn load_config(config_dir: &Path) -> Result<Config> {
    let config_file = config_dir.join("config.toml");

    if config_file.exists() {
        let config_content = std::fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&config_content)?;
        return Ok(config);
    }

    // Create default config
    let home = home_dir()?;
    let default_config = Config {
        general: GeneralConfig {
            roots_dir: home.join("roots").to_string_lossy().to_string(),
            trees_dir: home.join("trees").to_string_lossy().to_string(),
            copy: default_copy(),
            exec: vec![],
        },
        roots: HashMap::new(),
    };

    let config_toml = toml::to_string(&default_config)?;
    std::fs::write(&config_file, config_toml)?;

    Ok(default_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_creates_default() {
        let temp_dir = TempDir::new().unwrap();

        let result = load_config(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());

        let config = result.unwrap();

        // Should have default paths in home directory
        assert!(config.general.roots_dir.contains("roots"));
        assert!(config.general.trees_dir.contains("trees"));
        assert_eq!(config.general.copy, vec![".env", ".envrc"]);
        assert!(config.general.exec.is_empty());

        // Config file should have been created
        assert!(temp_dir.path().join("config.toml").exists());
    }

    #[test]
    fn test_load_reads_existing() {
        let temp_dir = TempDir::new().unwrap();

        let config_file = temp_dir.path().join("config.toml");
        let custom_config = r#"
[general]
roots_dir = "/custom/roots"
trees_dir = "/custom/trees"
copy = [".env.custom"]
exec = ["make setup"]

[roots.my-project]
copy = [".env.local"]
exec = ["npm install"]
"#;
        std::fs::write(&config_file, custom_config).unwrap();

        let result = load_config(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.general.roots_dir, "/custom/roots");
        assert_eq!(config.general.trees_dir, "/custom/trees");
        assert_eq!(config.general.copy, vec![".env.custom"]);
        assert_eq!(config.general.exec, vec!["make setup"]);
        assert_eq!(config.roots["my-project"].copy, vec![".env.local"]);
        assert_eq!(config.roots["my-project"].exec, vec!["npm install"]);
    }
}
