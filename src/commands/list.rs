pub fn run(root: Option<String>) {
    match root {
        Some(root_filter) => println!("[NOT IMPLEMENTED] Listing worktrees filtered by {root_filter}"),
        None => println!("[NOT IMPLEMENTED] Listing all worktrees"),
    }
}