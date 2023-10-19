mod arg_parser;
mod runner;
mod store;
mod screen;
use std::env;
use runner::Runner;

fn main() {
    let all_args: Vec<String> = env::args().collect();
    let args = &all_args[1..].to_vec();
    let runner = &mut Runner::new();

    if args.len() > 0 {
        runner.run_args(args);
        return;
    }

    runner.show_runner();
}
