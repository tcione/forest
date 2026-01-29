//! Grove configuration (.grove.toml)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Grove configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroveConfig {
    #[serde(default)]
    pub grove: GroveSettings,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub copy: CopyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveSettings {
    #[serde(default = "default_trees_dir")]
    pub trees: PathBuf,
    #[serde(default = "default_branch")]
    pub default_branch: String,
    #[serde(default)]
    pub forest: Option<ForestSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForestSettings {
    #[serde(default)]
    pub managed: bool,
    pub trees: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    #[serde(default)]
    pub post_create: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopyConfig {
    #[serde(default)]
    pub paths: Vec<String>,
}

fn default_trees_dir() -> PathBuf {
    PathBuf::from("./trees")
}

fn default_branch() -> String {
    "main".to_string()
}

impl Default for GroveSettings {
    fn default() -> Self {
        Self {
            trees: default_trees_dir(),
            default_branch: default_branch(),
            forest: None,
        }
    }
}

impl GroveConfig {
    pub fn trees_dir(&self) -> &PathBuf {
        if let Some(ref forest) = self.grove.forest
            && forest.managed
            && let Some(ref trees) = forest.trees
        {
            return trees;
        }
        &self.grove.trees
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: GroveConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_defaults_and_forest_override() {
        let config = GroveConfig::default();
        assert_eq!(config.grove.trees, PathBuf::from("./trees"));
        assert_eq!(config.grove.default_branch, "main");
        assert_eq!(config.trees_dir(), &PathBuf::from("./trees"));

        let mut managed = GroveConfig::default();
        managed.grove.forest = Some(ForestSettings {
            managed: true,
            trees: Some(PathBuf::from("/custom/trees")),
        });
        assert_eq!(managed.trees_dir(), &PathBuf::from("/custom/trees"));

        let mut unmanaged = GroveConfig::default();
        unmanaged.grove.forest = Some(ForestSettings {
            managed: false,
            trees: Some(PathBuf::from("/custom/trees")),
        });
        assert_eq!(unmanaged.trees_dir(), &PathBuf::from("./trees"));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".grove.toml");

        let mut config = GroveConfig::default();
        config.grove.default_branch = "develop".to_string();
        config.hooks.post_create = vec!["npm install".to_string()];
        config.copy.paths = vec![".env".to_string()];

        config.save(&config_path).unwrap();
        let loaded = GroveConfig::load(&config_path).unwrap();

        assert_eq!(loaded.grove.default_branch, "develop");
        assert_eq!(loaded.hooks.post_create, vec!["npm install"]);
        assert_eq!(loaded.copy.paths, vec![".env"]);
    }
}
