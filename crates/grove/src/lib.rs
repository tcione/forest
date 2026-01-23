//! Grove - A repository-scoped git worktree manager
//!
//! Grove manages git worktrees using bare repositories as the source,
//! where all working copies exist as worktrees.

pub mod config;
pub mod create;
pub mod delete;
pub mod git;
pub mod init;
pub mod list;
pub mod repo;
pub mod switch;
pub mod worktree;

pub use config::GroveConfig;
pub use create::{CreateResult, create};
pub use delete::delete;
pub use init::{InitResult, init};
pub use list::{WorktreeInfo, list};
pub use repo::GroveRepo;
pub use switch::switch;
