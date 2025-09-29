use std::path::PathBuf;

use crate::trees;
use crate::roots;
use crate::config::{Config, load_config};
use crate::utils::path::config_dir;
use crate::utils::cli_ui;

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
        self.handle(std::fs::create_dir_all(&self.roots_dir));
        self.handle(std::fs::create_dir_all(&self.trees_dir));
    }

    pub fn roots_clone(&self, repository_address: String) {
        match roots::clone::call(&self.roots_dir, repository_address) {
            Ok(root) => {
                let msg = format!("{} cloned into {}", root.name, root.path.display());
                println!("\n{}", cli_ui::success(&msg));
            },
            Err(err) => self.expected_error(err)
        }
    }

    pub fn roots_list(&self) {
        match roots::list::call(&self.roots_dir) {
            Ok(roots) => {
                if roots.is_empty() {
                    println!("\n{}", cli_ui::warn("No roots available"));
                    return;
                }

                for root in roots {
                    println!("{}", cli_ui::root_with_path(&root));
                }
            },
            Err(_) => self.expected_error("Roots directory does not exist!")
        }
    }

    pub fn roots_path(&self, root: String) {
        match roots::get::call(&self.roots_dir, &root) {
            Ok(root) => {
                println!("{}", root.path.display());
            },
            Err(err) => self.expected_error(err)
        }
    }

    pub fn roots_exec(&self, root: String, command: String) {
        self.handle(roots::exec::call(&self.roots_dir, root, command))
    }

    pub fn trees_clean(&self, root: Option<String>) {
        self.handle(trees::clean::call(&self, root))
    }

    pub fn trees_create(&self, root: String, new_branch_name: String) {
        self.handle(trees::create::call(&self, &root, &new_branch_name))
    }

    pub fn trees_delete(&self, root: String, tree: String) {
        self.handle(trees::delete::call(&self, &root, &tree))
    }

    pub fn trees_list(&self, root: Option<String>) {
        match trees::list::call(&self, &root) {
            Ok(roots_trees) => {
                roots_trees.iter().for_each(|(root, trees)| {
                    for tree in trees {
                        println!("{}", cli_ui::tree_with_path(&root, &tree));
                    }
                });
            },
            Err(err) => self.expected_error(err)
        }
    }

    pub fn trees_exec(&self, root: String, tree: String, command: String) {
        self.handle(trees::exec::call(&self, &root, &tree, command))
    }

    pub fn trees_path(&self, root: String, tree: String) {
        match trees::get::call(&self, &root, &tree) {
            Ok(t) => {
                println!("{}", t.path.display());
            },
            Err(err) => self.expected_error(err)
        }
    }

    fn expected_error<T: std::fmt::Display>(&self, message: T) {
        let msg = format!("Error: {}", message);
        eprintln!("\n{}", cli_ui::error(&msg));
        std::process::exit(1);
    }

    fn handle<T, E: std::fmt::Debug>(&self, rs: Result<T, E>) -> T {
        match rs {
            Ok(r) => r,
            Err(e) => {
                let err = format!("{:?}", e);
                eprintln!("\n{}", cli_ui::critical(&err));
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
