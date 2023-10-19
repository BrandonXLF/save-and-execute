pub fn title() {
    println!("Save & Execute v{} (c) Brandon Fowler", env!("CARGO_PKG_VERSION"));
}

pub fn show_help() {
    println!("\nusage: se <action> <identifier>

<action> is one of:
    -a, add    Add a new command using the given <identifier> as the initial name.
    -d, del    Delete the specified command.
    -e, edit   Edit the specified command.
    -h, help   Show this help message. <identifier> is ignored.
    -l, list   Show the command list. This is the default action when no <identifier> is given. <identifier> is ignored.
    -m, move   Move the specified command to a new position on the command list.
    -r, run    Run the specified command. This is the default action when a <identifier> is given.

<identifier> is either the name of the command or the index of the command on the command list.");
}