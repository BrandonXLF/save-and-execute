mod arg_parser;
mod runner;
mod command_store;
mod screen;
use std::env;
use runner::Runner;

fn show_help() {
    screen::title();

    println!("\nusage: se <action> <identifier>

Only providing <identifier> runs the specified command.
Providing neither <action> nor <identifier> shows the command list.

<action> is one of:
    a, add    Add a new command. 
    d, del    Delete the specified command.
    e, edit   Edit the specified command.
    h, help   Show this help message.
    l, list   Show the command list.
    m, move   Move the specified command to a new position on the command list.
    r, run    Run the specified command.

<identifier> is either the name of the command or the index of the command on the command list.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut runner = Runner::new();
    let mut has_args: bool = false;

    screen::create();

    if args.len() > 1 {
        has_args = true;

        match args[1].as_str() {
            "--help" | "-h" | "help" | "h" => return show_help(),
            "list" | "l" => has_args = false,
            _ => {}
        }
    }
    
    if has_args {
        runner.cmd_ui(&args[1..].to_vec());
        return;
    }

    let mut prev_error = None;

    while {
        prev_error = runner.select_ui(prev_error).err();
        prev_error.is_some()
    } {
        screen::clear();
    }
}
