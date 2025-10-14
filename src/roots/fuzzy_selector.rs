use anyhow::Result;
use dialoguer::FuzzySelect;
use std::path::Path;

use super::{Root, list, get};

use crate::utils::cli_ui;

pub fn call(roots_dir: &Path, root: Option<String>) -> Result<Root> {
    if let Some(_) = root {
        let root_struct = get::call(roots_dir, root)?;
        return Ok(root_struct);
    }

    let roots = list::call(&roots_dir)?;

    if roots.is_empty() {
        anyhow::bail!("No roots cloned yet")
    }

    let names: Vec<String> = roots.iter().map(|root| root.name.clone()).collect();

    let selection = FuzzySelect::new()
        .with_prompt(cli_ui::prompt("Select root"))
        .items(&names)
        .interact_opt()?;

    match selection {
        Some(index) => {
            let selected_root = Some(names[index].clone());
            let root_struct = get::call(roots_dir, selected_root)?;
            Ok(root_struct)
        },
        None => anyhow::bail!("Root selection cancelled")
    }
}
