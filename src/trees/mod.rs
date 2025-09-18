use std::collections::HashMap;
use std::path::PathBuf;

pub type RootsTrees = HashMap<String, Trees>;
pub type Trees = Vec<Tree>;

#[derive(Debug, PartialEq)]
pub struct Tree {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
    pub head: String,
}

pub mod list;
pub mod create;
