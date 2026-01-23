//! Grove - A repository-scoped git worktree manager
//!
//! Grove manages git worktrees using bare repositories as the source,
//! where all working copies exist as worktrees.

pub mod config;
pub mod git;
pub mod worktree;

pub use config::GroveConfig;
