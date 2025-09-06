use std::path::PathBuf;

use crate::commands::{clone, create};
use crate::utils::config::{Config, load_config};
use crate::utils::path::config_dir;

pub struct Application {
    pub roots_dir: PathBuf,
    pub trees_dir: PathBuf,
    pub config: Config,
}

impl Application {
    pub fn new() -> Self {
        let config_dir = config_dir().unwrap();
        let config = load_config(config_dir).unwrap();

        Self {
            roots_dir: PathBuf::from(&config.general.base_dir).join("roots"),
            trees_dir: PathBuf::from(&config.general.base_dir).join("trees"),
            config,
        }
    }

    pub fn setup(&self) {
        self.pvt_handle(std::fs::create_dir_all(&self.roots_dir));
        self.pvt_handle(std::fs::create_dir_all(&self.trees_dir));
    }

    pub fn clone(&self, repository_address: String) {
        self.pvt_handle(clone::run(&self.roots_dir, repository_address))
    }

    pub fn create(&self, root: String, new_branch_name: String) {
        self.pvt_handle(create::run(&self, &root, &new_branch_name))
    }

    fn pvt_handle<T, E: std::fmt::Debug>(&self, rs: Result<T, E>) -> T {
        match rs {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
pub fn test_application(
    copy: Vec<String>,
    exec: Vec<String>,
    roots: std::collections::HashMap<String, crate::utils::config::RootConfig>,
) -> Application {
    let unique_id = format!("{:?}_{}", std::thread::current().id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());
    let base_dir = std::env::temp_dir().join(format!("forest_test_{}", unique_id));

    let application = Application {
        roots_dir: base_dir.join("roots").to_path_buf(),
        trees_dir: base_dir.join("trees").to_path_buf(),
        config: crate::utils::config::Config {
            general: crate::utils::config::GeneralConfig {
                base_dir: base_dir.to_string_lossy().to_string(),
                copy: copy,
                exec: exec,
            },
            roots: roots,
        },
    };

    application.setup();

    application
}
