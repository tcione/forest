use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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
