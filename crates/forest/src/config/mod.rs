use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub roots: HashMap<String, RootConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub base_dir: String,
    pub copy: Vec<String>,
    pub exec: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RootConfig {
    pub copy: Vec<String>,
    pub exec: Vec<String>,
}

mod loader;
pub use loader::load as load_config;
