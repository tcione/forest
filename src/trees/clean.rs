use anyhow::Result;
use console::style;
use dialoguer::{Confirm, MultiSelect};

use super::Tree;
use crate::application::Application;
use crate::trees::delete::call as delete_call;
use crate::trees::list::call as list_call;

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
        print_end_warning("No trees found to clean");
        return Ok(());
    }

    let display_items: Vec<String> = all_trees
        .iter()
        .map(|(root, tree)| format!("[{}] {}", root, tree.branch))
        .collect();

    let select_prompt = format_prompt("Select trees to delete (j/k to navigate, space to toggle");

    let selections = MultiSelect::new()
        .with_prompt(select_prompt)
        .items(&display_items)
        .interact()?;

    if selections.is_empty() {
        print_end_warning("No trees selected");
        return Ok(());
    }

    println!("{}", format_prompt("⚠ Trees selected for deletion:"));
    for &i in &selections {
        let (root, tree) = &all_trees[i];
        let item = format!(
            "-> [{}] {}",
            style(root),
            style(tree.branch.clone()),
        );
        println!("{}", style(item).dim());
    }

    println!("\n{}", style("This action cannot be undone!").red());

    if Confirm::new()
        .with_prompt(format_prompt("Delete these trees?"))
        .default(false)
        .interact()?
    {
        println!("");
        for &i in &selections {
            let (root, tree) = &all_trees[i];
            let display = format!("[{}] {}", root, tree.branch);
            match delete_call(application, root.clone(), tree.branch.clone()) {
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
        println!("\n{}", style("Worktrees deleted").on_green().black().bold());
    } else {
        print_end_warning("Clean cancelled");
    }

    println!("");

    Ok(())
}

fn print_end_warning(message: &str) {
    println!("\n{}", style(message).on_yellow().black().bold());
}

fn format_prompt(message: &str) -> String {
    format!("\n{}", style(message).cyan())
}
