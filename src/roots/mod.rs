use std::path::PathBuf;

pub type Roots = Vec<Root>;

#[derive(Debug)]
pub struct Root {
    pub name: String,
    pub path: PathBuf,
}

pub mod clone;
pub mod get;
pub mod exec;
pub mod list;
