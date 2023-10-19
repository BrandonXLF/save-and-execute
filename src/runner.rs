use std::{collections::HashMap, process::Command};
use dialoguer::Input;
use crate::{arg_parser, store::{Store, CommandInfo}, screen};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

type Action = fn(&mut Runner, &str) -> Result<(), String>;

pub struct Runner {
    actions: HashMap<&'static str, Action>,
    aliases: HashMap<&'static str, &'static str>,
    store: Store,
    commands: Vec<CommandInfo>
}

impl Runner {
    pub fn new() -> Self {
        let actions: HashMap<&str, Action> = HashMap::from([
            ("add", |runner: &mut Self, _: &str| -> Result<(), String> {
                runner.commands.push(runner.create_cmd(None)?);
                runner.save_commands()?;
        
                println!("\nCommand created successfully.");

                return Ok(());
            } as Action),
            ("del", |runner: &mut Self, identifier: &str| -> Result<(), String> {
                let index = runner.get_command_index(identifier)?;

                println!();
                let confirm = Input::<String>::new()
                    .with_prompt(format!("Are you sure you want to delete command #{}? (Y/N)", index + 1).as_str())
                    .interact_text()
                    .unwrap();
        
                if confirm != "y" && confirm != "yes" {
                    return Err("Deletion aborted.".into());
                }
                
                runner.commands.remove(index);
                runner.save_commands()?;
        
                println!("\nCommand deleted successfully.");

                return Ok(());
            } as Action),
            ("edit", |runner: &mut Self, identifier: &str| -> Result<(), String> {
                let index: usize = runner.get_command_index(identifier)?;
            
                runner.commands[index] = runner.create_cmd(Some(&runner.commands[index]))?;
                runner.save_commands()?;
        
                println!("\nCommand edited successfully.");

                return Ok(());
            } as Action),
            ("move", |runner: &mut Self, identifier: &str| -> Result<(), String> {
                let index = runner.get_command_index(identifier)?;

                println!();
                let pos_line = Input::<String>::new()
                    .with_prompt("Enter new position")
                    .interact_text()
                    .unwrap();
    
                let mut new_index = pos_line.parse::<usize>().map_err(|_| "Invalid number.")? - 1;
    
                if new_index > runner.commands.len() {
                    new_index = runner.commands.len();
                }
    
                let cmd = runner.commands.remove(index);
                runner.commands.insert(new_index, cmd);
                runner.save_commands()?;
    
                println!("\nCommand moved successfully.");

                return Ok(());
            } as Action), 
            ("run", |runner: &mut Self, identifier: &str| -> Result<(), String> {
                let cmd_info = &runner.commands[runner.get_command_index(identifier)?];
                
                println!("\nRunning command: {}\n", cmd_info.cmd);

                #[cfg(target_os = "windows")]
                let status = Command::new("cmd").arg("/c").raw_arg(&cmd_info.cmd).status().map_err(|_| "Failed to run cmd.")?;
                #[cfg(not(target_os = "windows"))]
                let status = Command::new("sh").arg("-c").arg(&cmd_info.cmd).status().map_err(|_| "Failed to run sh.")?;
            
                println!(
                    "\nCommand exited with status code {}.",
                    status.code().map(|code| code.to_string()).unwrap_or("N/A".into())
                );

                return Ok(());
            } as Action),
            ("help", |_: &mut Self, _: &str| -> Result<(), String> {
                println!("\nusage: se <action> <identifier>

<action> is one of:
    -a, add    Add a new command. <identifier> is ignored.
    -d, del    Delete the specified command.
    -e, edit   Edit the specified command.
    -h, help   Show this help message. <identifier> is ignored.
    -l, list   Show the command list. This is the default action when no <identifier> is given. <identifier> is ignored.
    -m, move   Move the specified command to a new position on the command list.
    -r, run    Run the specified command. This is the default action when a <identifier> is given.

<identifier> is either the name of the command or the index of the command on the command list.");

                return Ok(());
            } as Action),
            ("list", |runner: &mut Self, _: &str| -> Result<(), String> {
                println!();

                for (i, cmd_info) in runner.commands.iter().enumerate() {
                    println!("{}. {}", i + 1, arg_parser::escape_string(&cmd_info.name));
                }
        
                if runner.commands.len() == 0 {
                    println!("No commands saved. Run \"se -a\" to get started.");
                }

                return Ok(());
            } as Action),
        ]);
    
        let aliases: HashMap<&str, &str> = HashMap::from([
            ("-a", "add"),
            ("-d", "del"),
            ("-e", "edit"),
            ("-m", "move"),
            ("-r", "run"),
            ("-h", "help"),
            ("-l", "list"),
        ]);

        let store = Store::new();
        let commands = store.get_commands();

        Self { actions, aliases, store, commands }
    }

    fn create_cmd(&self, base: Option<&CommandInfo>) -> Result<CommandInfo, String> {
        println!();

        let name = Input::<String>::new()
            .with_prompt("Name")
            .with_initial_text(base.map(|base| base.name.as_str()).unwrap_or_default())
            .interact_text()
            .unwrap();
    
        let cmd = Input::<String>::new()
            .with_prompt("Command")
            .with_initial_text(base.map(|base| base.cmd.as_str()).unwrap_or_default())
            .interact_text()
            .unwrap();

        if (base.is_none() || base.unwrap().name != name) &&
            self.commands.iter().any(|command| command.name == name)
        {
            return Err(format!("Command with name \"{}\" already exists.", name));
        }
    
        Ok(CommandInfo { name, cmd })
    }

    fn save_commands(&self) -> Result<(), String> {
        self.store.save_commands(&self.commands)
    }

    fn get_command_index(&self, line: &str) -> Result<usize, String>  {
        if line.len() == 0 {
            return Err("No command specified for action.".into());
        }
    
        let maybe_number: Option<usize> = line.parse::<usize>().ok();

        let index = if let Some(number) = maybe_number {
            number - 1
        } else {
            self.commands.iter().position(|command| command.name == line)
                .ok_or(format!("Command with name \"{}\" not found.", line))?
        };
    
        if index >= self.commands.len() {
            return Err(format!("Command #{} does not exist.", line));
        }
    
        Ok(index)
    }

    fn get_action(&mut self, action: &str) -> Option<&Action> {
        let resolved_action = match self.aliases.get(action) {
            Some(target_action) => target_action,
            _ => action
        };

        return self.actions.get(resolved_action);
    }

    fn exec_from_args(&mut self, args: &Vec<String>) -> Result<(), String> {
        let arg_count = args.len();

        if arg_count > 2 {
            return Err("Too many arguments.".into());
        }

        if arg_count == 0 || args[0].is_empty() {
            return Err("No action or identifier given!".into());
        }

        let mut action: Option<&Action> = self.get_action(&args[0]);
        let identifier: &str;

        if arg_count == 2 && action.is_none() {
            return Err(format!("Unknown action \"{}\".", &args[0]));
        }

        if arg_count == 2 {
            identifier = &args[1];
        } else if action.is_none() {
            action = self.get_action("run");
            identifier = &args[0];
        } else {
            identifier = "";
        }

        return action.unwrap()(self, identifier);
    }

    fn process_args(&mut self, args: &Vec<String>) {
        let res = self.exec_from_args(args);

        if let Err(error) = res {
            println!("\nError: {}", error);
        }
    }

    pub fn run_args(&mut self, args: &Vec<String>) {
        screen::title();

        self.process_args(args);
    }

    pub fn show_runner(&mut self) {
        screen::title();

        loop {
            println!();
            let line = Input::<String>::new()
                .with_prompt("se")
                .interact_text()
                .unwrap();

            let args = &arg_parser::string_to_arguments(line.trim());

            if args.len() > 0 && args[0] == "exit" {
                break;
            }

            self.process_args(&arg_parser::string_to_arguments(line.trim()));
        }
    }
}