use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::utils::path::{home_dir};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub roots: HashMap<String, RootConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub root_dir: String,
    pub copy: Vec<String>,
    pub exec: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RootConfig {
    pub copy: Vec<String>,
    pub exec: Vec<String>,
}

fn default_copy() -> Vec<String> {
    vec![".env".to_string(), ".envrc".to_string()]
}

pub fn load_config(config_dir: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let config_file = config_dir.join("config.toml");

    if config_file.exists() {
        let config_content = std::fs::read_to_string(config_file)?;
        let config: Config = toml::from_str(&config_content)?;

        return Ok(config);
    }

    let home_dir_string = home_dir()?.to_string_lossy().to_string();
    let default_config = Config {
        general: GeneralConfig {
            root_dir: format!("{}/Projects", home_dir_string),
            copy: default_copy(),
            exec: vec![],
        },
        roots: HashMap::new(),
    };

    let config_toml = toml::to_string(&default_config)?;
    std::fs::write(config_file, config_toml)?;

    Ok(default_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_creates_default() {
        let temp_dir = std::env::temp_dir().join("forest_test_load_config_creates_default");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let result = load_config(temp_dir.clone());
        assert!(result.is_ok());

        let config = result.unwrap();

        println!("{:?}", config);

        // Default values
        let home_dir_string = home_dir().unwrap().to_string_lossy().to_string();
        assert_eq!(config.general.root_dir, format!("{}/Projects", home_dir_string));
        assert_eq!(config.general.copy, vec![".env".to_string(), ".envrc".to_string()]);
        assert!(config.general.exec.is_empty());

        let config_file = temp_dir.join("config.toml");
        assert!(config_file.exists());

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_config_reads_existing() {
        let temp_dir = std::env::temp_dir().join("sundial_test_load_config_reads_existing");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let config_file = temp_dir.join("config.toml");
        let custom_config_content = r#"
[general]
root_dir = "/home/Custom"
copy = [".env.custom"]
exec = ["exec custom"]

[roots.repo1]
copy = [".env.repo1"]
exec = ["exec repo1"]

[roots.repo2]
copy = [".env.repo2"]
exec = ["exec repo2"]
"#;
        std::fs::write(&config_file, custom_config_content).unwrap();

        let result = load_config(temp_dir.clone());
        assert!(result.is_ok());

        let config = result.unwrap();

        assert_eq!(config.general.root_dir, "/home/Custom");
        assert_eq!(config.general.copy, vec![".env.custom"]);
        assert_eq!(config.general.exec, vec!["exec custom"]);
        assert_eq!(config.roots["repo1"].copy, vec![".env.repo1"]);
        assert_eq!(config.roots["repo1"].exec, vec!["exec repo1"]);
        assert_eq!(config.roots["repo2"].copy, vec![".env.repo2"]);
        assert_eq!(config.roots["repo2"].exec, vec!["exec repo2"]);

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
