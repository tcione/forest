//! Grove configuration (.grove.toml)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main Grove configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroveConfig {
    #[serde(default)]
    pub grove: GroveSettings,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub copy: CopyConfig,
}

/// Core grove settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveSettings {
    /// Directory where worktrees are created (default: "./trees")
    #[serde(default = "default_trees_dir")]
    pub trees: PathBuf,
    /// Main branch name (default: "main")
    #[serde(default = "default_main_branch")]
    pub main_branch: String,
    /// Forest-specific settings (set when managed by Forest)
    #[serde(default)]
    pub forest: Option<ForestSettings>,
}

/// Forest-specific settings (when repo is managed by Forest)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForestSettings {
    /// Whether this repo is managed by Forest
    #[serde(default)]
    pub managed: bool,
    /// Forest-controlled trees directory (supersedes grove.trees when managed)
    pub trees: Option<PathBuf>,
}

/// Post-creation hooks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    /// Commands to run after creating a worktree
    #[serde(default)]
    pub post_create: Vec<String>,
}

/// Files/directories to copy when creating a worktree
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopyConfig {
    /// Paths to copy (files or directories)
    #[serde(default)]
    pub paths: Vec<String>,
}

fn default_trees_dir() -> PathBuf {
    PathBuf::from("./trees")
}

fn default_main_branch() -> String {
    "main".to_string()
}

impl Default for GroveSettings {
    fn default() -> Self {
        Self {
            trees: default_trees_dir(),
            main_branch: default_main_branch(),
            forest: None,
        }
    }
}

impl GroveConfig {
    /// Get the effective trees directory (Forest path takes precedence if managed)
    pub fn trees_dir(&self) -> &PathBuf {
        if let Some(ref forest) = self.grove.forest
            && forest.managed
            && let Some(ref trees) = forest.trees
        {
            return trees;
        }
        &self.grove.trees
    }

    /// Load config from a .grove.toml file
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: GroveConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to a .grove.toml file
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
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
    fn test_default_config() {
        let config = GroveConfig::default();
        assert_eq!(config.grove.trees, PathBuf::from("./trees"));
        assert_eq!(config.grove.main_branch, "main");
        assert!(config.grove.forest.is_none());
        assert!(config.hooks.post_create.is_empty());
        assert!(config.copy.paths.is_empty());
    }

    #[test]
    fn test_trees_dir_default() {
        let config = GroveConfig::default();
        assert_eq!(config.trees_dir(), &PathBuf::from("./trees"));
    }

    #[test]
    fn test_trees_dir_forest_managed() {
        let mut config = GroveConfig::default();
        config.grove.forest = Some(ForestSettings {
            managed: true,
            trees: Some(PathBuf::from("/custom/trees")),
        });
        assert_eq!(config.trees_dir(), &PathBuf::from("/custom/trees"));
    }

    #[test]
    fn test_trees_dir_forest_not_managed() {
        let mut config = GroveConfig::default();
        config.grove.forest = Some(ForestSettings {
            managed: false,
            trees: Some(PathBuf::from("/custom/trees")),
        });
        // Should use grove.trees since not managed
        assert_eq!(config.trees_dir(), &PathBuf::from("./trees"));
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".grove.toml");

        let mut config = GroveConfig::default();
        config.grove.main_branch = "master".to_string();
        config.hooks.post_create = vec!["echo hello".to_string()];
        config.copy.paths = vec![".env".to_string(), "config/".to_string()];

        config.save(&config_path).unwrap();
        let loaded = GroveConfig::load(&config_path).unwrap();

        assert_eq!(loaded.grove.main_branch, "master");
        assert_eq!(loaded.hooks.post_create, vec!["echo hello"]);
        assert_eq!(loaded.copy.paths, vec![".env", "config/"]);
    }

    #[test]
    fn test_parse_toml_with_forest_settings() {
        let toml_content = r#"
[grove]
trees = "./my-trees"
main_branch = "develop"

[grove.forest]
managed = true
trees = "~/trees/my-project--"

[hooks]
post_create = ["npm install", "cp .env.example .env"]

[copy]
paths = [".env.local", "config/local/"]
"#;
        let config: GroveConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.grove.trees, PathBuf::from("./my-trees"));
        assert_eq!(config.grove.main_branch, "develop");
        assert!(config.grove.forest.as_ref().unwrap().managed);
        assert_eq!(
            config.grove.forest.as_ref().unwrap().trees,
            Some(PathBuf::from("~/trees/my-project--"))
        );
        assert_eq!(config.hooks.post_create.len(), 2);
        assert_eq!(config.copy.paths.len(), 2);
    }
}
