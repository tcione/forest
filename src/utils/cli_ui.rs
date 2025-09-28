use console::style;

use crate::trees::Tree;

pub fn prompt(msg: &str) -> String {
    format!("\n{}", style(msg).cyan())
}

pub fn warn(msg: &str) -> String {
    format!("\n{}", style(msg).on_yellow().black().bold())
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
