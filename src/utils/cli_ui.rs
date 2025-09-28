use console::style;

use crate::trees::Tree;
use crate::roots::Root;

pub fn prompt(msg: &str) -> String {
    format!("\n{}", style(msg).cyan())
}

pub fn warn(msg: &str) -> String {
    format!("\n{}", style(msg).on_yellow().black().bold())
}

pub fn error(msg: &str) -> String {
    format!("\n{}", style(msg).on_red().black().bold())
}

pub fn success(msg: &str) -> String {
    format!("\n{}", style(msg).on_green().black().bold())
}

pub fn critical(msg: &str) -> String {
    format!("\n{}", style(msg).red())
}

pub fn tree(root: &str, tree: &Tree) -> String {
    format!("[{}] {}", root, tree.branch)
}

pub fn tree_with_path(root: &str, tree: &Tree) -> String {
    let f_root = format!("[{}]", root);
    let path = format!("-> {}", tree.path.display());
    format!("{} {} {}", style(f_root).cyan(), tree.branch, style(path).dim())
}

pub fn root_with_path(root: &Root) -> String {
    let path = format!("-> {}", root.path.display());
    format!("[{}] {}", root.name, style(path).dim())
}
