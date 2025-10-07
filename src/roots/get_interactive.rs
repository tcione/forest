use anyhow::Result;
use dialoguer::FuzzySelect;
use std::path::Path;

use super::{Root, list, get};
use crate::utils::cli_ui;

pub fn call(roots_dir: &Path, root: Option<&str>) -> Result<Root> {
    match root {
        Some(r) => get::call(roots_dir, r),
        None => {
            let roots = list::call(&roots_dir.to_path_buf())?;

            if roots.is_empty() {
                anyhow::bail!("No roots available");
            }

            let names: Vec<String> = roots.iter().map(|r| r.name.clone()).collect();

            let selection = FuzzySelect::new()
                .with_prompt(cli_ui::prompt("Select root"))
                .items(&names)
                .interact()?;

            get::call(roots_dir, &names[selection])
        }
    }
}
