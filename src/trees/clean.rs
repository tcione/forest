use anyhow::Result;
use console::style;
use dialoguer::{Confirm, MultiSelect};

use super::Tree;
use crate::application::Application;
use crate::trees::delete::call as delete_call;
use crate::trees::list::call as list_call;
use crate::utils::cli_ui;

pub fn call(application: &Application, root: Option<String>) -> Result<()> {
    let trees_map = list_call(application, &root)?;

    let all_trees: Vec<(String, Tree)> = trees_map
        .iter()
        .flat_map(|(root_name, trees)| {
            trees
                .iter()
                .map(move |tree| (root_name.clone(), tree.clone()))
        })
        .collect();

    if all_trees.is_empty() {
        println!("\n{}", cli_ui::warn("No trees found to clean"));
        return Ok(());
    }

    let display_items: Vec<String> = all_trees
        .iter()
        .map(|(root, tree)| cli_ui::tree(root, &tree))
        .collect();

    let select_prompt = cli_ui::prompt("\nSelect trees to delete (j/k to navigate, space to toggle");

    let selections = MultiSelect::new()
        .with_prompt(select_prompt)
        .items(&display_items)
        .interact()?;

    if selections.is_empty() {
        println!("\n{}", cli_ui::warn("No trees selected"));
        return Ok(());
    }

    println!("\n{}", cli_ui::prompt("⚠ Trees selected for deletion:"));
    for &i in &selections {
        let (root, tree) = &all_trees[i];
        let item = format!(
            "-> {}",
            cli_ui::tree(root, &tree),
        );
        println!("{}", style(item).dim());
    }

    println!("\n{}", cli_ui::critical("This action cannot be undone!"));

    if Confirm::new()
        .with_prompt(cli_ui::prompt("\nDelete these trees?"))
        .default(false)
        .interact()?
    {
        println!("");
        for &i in &selections {
            let (root, tree) = &all_trees[i];
            let display = cli_ui::tree(root, &tree);
            match delete_call(application, root, &tree.branch) {
                Ok(()) => println!(
                    "{} {}",
                    style("• Deleted: ").green().dim(),
                    display,
                ),
                Err(e) => eprintln!(
                    "{} {} {}: {}",
                    style("• Failed to delete '").red().dim(),
                    display,
                    "' with error",
                    e,
                ),
            }
        }
        println!("\n{}", cli_ui::success("Worktrees deleted"));
    } else {
        println!("\n{}", cli_ui::warn("Clean cancelled"));
    }

    println!("");

    Ok(())
}
