//! Grove - A repository-scoped git worktree manager
//!
//! Grove manages git worktrees using bare repositories as the source,
//! where all working copies exist as worktrees.

pub mod config;
pub mod create;
pub mod git;
pub mod init;
pub mod repo;
pub mod worktree;

pub use config::GroveConfig;
pub use create::{CreateResult, create};
pub use init::{InitResult, init};
pub use repo::GroveRepo;
