use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands {
    pub mod clone;
}

mod utils {
    pub mod config;
    pub mod path;
}

#[derive(Debug, Parser)]
#[command(name = "forest")]
#[command(about = "Convention-over-configuration CLI tool to manager git worktrees", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Clones git repository inside roots/
    #[command(arg_required_else_help = true)]
    Clone {
        /// The git repository you want to clone. Use the the same address you'd
        /// use for "git clone"
        repository_address: String,
    },

    /// Creates a worktree for the repo inside trees/
    #[command(arg_required_else_help = true)]
    Create {
        /// The repository name (same as the name of the folder in your system)
        root: String,
        /// Name for your new branch. Follow normal git conventions
        new_branch_name: String,
    },

    /// List all worktrees under our forest service (TM)
    #[command(arg_required_else_help = true)]
    List {
        /// Filters list by root (repository name)
        #[arg(long)]
        root: Option<String>,
    },

    /// Easy access for a particular tree
    #[command(arg_required_else_help = true)]
    Goto {
        /// Filters options by root when no tree is given
        #[arg(long)]
        root: Option<String>,

        /// Command to be executed when going to tree
        #[arg(long, default_value = "cd")]
        cmd: String,

        /// Tree to go to using the <root:branch> format
        #[arg(value_name = "ROOT:BRANCH", help = "Example: myrepo:feature/branch")]
        tree: Option<String>,
    },

    /// Much necessary clean-up utility
    #[command(arg_required_else_help = true)]
    Clean {
        /// Filters options by root when no tree is given
        #[arg(long)]
        root: Option<String>,
    },
}

struct Application {
    roots_dir: PathBuf,
    trees_dir: PathBuf,
}

impl Application {
    fn new() -> Self {
        let config_dir = utils::path::config_dir().unwrap();
        let config = utils::config::load_config(config_dir).unwrap();

        Self {
            roots_dir: PathBuf::from(&config.general.base_dir).join("roots"),
            trees_dir: PathBuf::from(&config.general.base_dir).join("trees"),
        }
    }

    fn setup(&self) {
        self.pvt_handle(std::fs::create_dir_all(&self.roots_dir));
        self.pvt_handle(std::fs::create_dir_all(&self.trees_dir));
    }

    fn clone(&self, repository_address: String) {
        self.pvt_handle(commands::clone::run(&self.roots_dir, repository_address))
    }

    fn create(&self, root: String, new_branch_name: String) {
        self.pvt_handle(commands::create::run(&self.roots_dir, &self.trees_dir, &root, &new_branch_name))
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

fn main() {
    let args = Cli::parse();
    let forest = Application::new();

    forest.setup();

    match args.command {
        Commands::Clone { repository_address } => forest.clone(repository_address),
        Commands::Create { root, new_branch_name } => {},
        Commands::List { root } => {},
        Commands::Goto { root, cmd, tree } => {},
        Commands::Clean { root } => {},
    }
}
