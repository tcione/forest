use std::path::PathBuf;

use crate::commands::{roots, trees};
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

    pub fn roots_clone(&self, repository_address: String) {
        self.pvt_handle(roots::clone::run(&self.roots_dir, repository_address))
    }

    pub fn roots_list(&self) {
        self.pvt_handle(roots::list::run(&self.roots_dir))
    }

    pub fn roots_enter(&self, root: String) {
        self.pvt_handle(roots::enter::run(&self.roots_dir, root))
    }

    pub fn roots_exec(&self, root: String, command: String) {
        self.pvt_handle(roots::exec::run(&self.roots_dir, root, command))
    }

    pub fn trees_create(&self, root: String, new_branch_name: String) {
        self.pvt_handle(trees::create::run(&self, &root, &new_branch_name))
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
pub struct TestApplication {
    pub application: Application,
    pub _temp_dir: tempfile::TempDir, // kept for RAII
}

#[cfg(test)]
impl std::ops::Deref for TestApplication {
    type Target = Application;

    fn deref(&self) -> &Self::Target {
        &self.application
    }
}

#[cfg(test)]
pub fn test_application(
    copy: Vec<String>,
    exec: Vec<String>,
    roots: std::collections::HashMap<String, crate::utils::config::RootConfig>,
) -> TestApplication {
    let base_temp_dir = tempfile::TempDir::new().unwrap();
    let base_dir = base_temp_dir.path();

    let application = Application {
        roots_dir: base_dir.join("roots"),
        trees_dir: base_dir.join("trees"),
        config: crate::utils::config::Config {
            general: crate::utils::config::GeneralConfig {
                base_dir: base_dir.to_string_lossy().to_string(),
                copy,
                exec,
            },
            roots,
        },
    };

    application.setup();

    TestApplication {
        application,
        _temp_dir: base_temp_dir,
    }
}
