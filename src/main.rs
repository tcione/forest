use clap::{Parser, Subcommand};

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
    #[command(visible_alias = "clone", arg_required_else_help = true)]
    Plant {
        /// The git repository you want to clone. Use the the same address you'd
        /// use for "git clone"
        repository_address: String,
    },

    /// Creates a worktree for the repo inside trees/
    #[command(visible_alias = "create", arg_required_else_help = true)]
    Grow {
        /// The repository name (same as the name of the folder in your system)
        root: String,
        /// Name for your new branch. Follow normal git conventions
        new_branch_name: String,
    },

    /// List all worktrees under our forest service (TM)
    #[command(visible_alias = "list", arg_required_else_help = true)]
    Check {
        /// Filters list by root (repository name)
        #[arg(long)]
        root: Option<String>,
    },

    /// Easy access for a particular tree
    #[command(visible_alias = "goto", arg_required_else_help = true)]
    Nurture {
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
    #[command(visible_alias = "clean", arg_required_else_help = true)]
    Trim {
        /// Filters options by root when no tree is given
        #[arg(long)]
        root: Option<String>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Plant { repository_address } => {
            println!("[NOT IMPLEMENTED] Plant");
        }
        Commands::Grow { root, new_branch_name } => {
            println!("[NOT IMPLEMENTED] Grow");
        }
        Commands::Check { root } => {
            println!("[NOT IMPLEMENTED] Check");
        }
        Commands::Nurture { root, cmd, tree } => {
            println!("[NOT IMPLEMENTED] Nurture");
        }
        Commands::Trim { root } => {
            println!("[NOT IMPLEMENTED] Trim");
        }
    }
}
