//! CLI output formatting

use console::style;

pub fn context(msg: &str) -> String {
    format!("{}", style(msg).dim())
}

pub fn success(msg: &str) -> String {
    format!("{}", style(msg).green())
}

pub fn warn(msg: &str) -> String {
    format!("{}", style(msg).yellow())
}

pub fn error(msg: &str) -> String {
    format!("{}", style(msg).red())
}

pub fn highlight(msg: &str) -> String {
    format!("{}", style(msg).cyan())
}

pub fn branch_with_path(branch: &str, path: &std::path::Path, is_default: bool) -> String {
    let marker = if is_default { " *" } else { "" };
    let path_str = format!("-> {}", path.display());
    format!(
        "{}{} {}",
        style(branch).cyan(),
        marker,
        style(path_str).dim()
    )
}
