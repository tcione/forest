use std::path::PathBuf;

pub type Roots = Vec<Root>;

#[derive(Debug)]
pub struct Root {
    pub name: String,
    pub path: PathBuf,
}

pub mod clone;
pub mod exec;
pub mod fuzzy_selector;
pub mod get;
pub mod list;
