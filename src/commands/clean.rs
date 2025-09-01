pub fn run(root: Option<String>) {
    match root {
        Some(root_filter) => println!("[NOT IMPLEMENTED] Cleaning worktrees filtered by {root_filter}"),
        None => println!("[NOT IMPLEMENTED] Cleaning all worktrees"),
    }
}