use std::path::PathBuf;

use crate::trees;
use crate::roots;
use crate::config::{Config, load_config};
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
        self.pvt_handle(roots::clone::call(&self.roots_dir, repository_address))
    }

    pub fn roots_list(&self) {
        match roots::list::call(&self.roots_dir) {
            Ok(roots) => {
                if roots.is_empty() {
                    println!("No roots available");
                    return;
                }

                for root in roots {
                    println!("{} -> {}", &root.name, &root.path.display());
                }
            },
            Err(_) => self.expected_error("Roots directory does not exist!")
        }
    }

    pub fn roots_enter(&self, root: String) {
        self.pvt_handle(roots::enter::call(&self.roots_dir, root))
    }

    pub fn roots_exec(&self, root: String, command: String) {
        self.pvt_handle(roots::exec::call(&self.roots_dir, root, command))
    }

    pub fn trees_create(&self, root: String, new_branch_name: String) {
        self.pvt_handle(trees::create::call(&self, &root, &new_branch_name))
    }

    pub fn trees_list(&self, root: Option<String>) {
        match trees::list::call(&self, &root) {
            Ok(trees) => {
                println!("{:?}", trees);
            },
            Err(err) => self.expected_error(err)
        }
    }

    fn expected_error<T: std::fmt::Display>(&self, message: T) {
        eprintln!("Error: {}", message);
        std::process::exit(1);
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
    roots: std::collections::HashMap<String, crate::config::RootConfig>,
) -> TestApplication {
    let base_temp_dir = tempfile::TempDir::new().unwrap();
    let base_dir = base_temp_dir.path();

    let application = Application {
        roots_dir: base_dir.join("roots"),
        trees_dir: base_dir.join("trees"),
        config: crate::config::Config {
            general: crate::config::GeneralConfig {
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
