pub fn title() {
    println!("Save and Execute v{} (c) Brandon Fowler", env!("CARGO_PKG_VERSION"));
}

pub fn show_help(ui: bool) {
    println!();

    if ui {
        println!("usage: <action> <identifier>");
    } else {
        println!("usage: se <action> <identifier>");
    }

    println!("
<action> is one of:
    -a, add    Add a new command using the given <identifier> as the initial name.
    -d, del    Delete the specified command.
    -e, edit   Edit the specified command.
    -h, help   Show this help message. <identifier> is ignored.
    -l, list   Show the command list. This is the default action when no <identifier> is given. <identifier> is ignored.
    -m, move   Move the specified command to a new position on the command list.
    -r, run    Run the specified command. This is the default action when only a <identifier> is given.
               Arguments passed will replace %0, %1, %2, etc. with %0 being the command's name.
    -v, view   View the specified command.");

    if ui {
        println!("        exit   Exit this command line.")
    }

    println!("
<identifier> is either the name of the command or the index of the command on the command list.");
}