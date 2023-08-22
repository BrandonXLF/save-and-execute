use std::{collections::HashMap, process::Command};
use dialoguer::Input;
use crate::{arg_parser, command_store::{CommandStore, CommandInfo}, screen};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

type Action = fn(&mut Runner, &str) -> Result<(), String>;

pub struct Runner {
    actions: HashMap<String, Action>,
    aliases: HashMap<String, String>,
    store: CommandStore,
    commands: Vec<CommandInfo>
}

impl Runner {
    pub fn new() -> Self {
        let actions: HashMap<String, Action> = HashMap::from([
            ("add".into(), |runner: &mut Self, _: &str| -> Result<(), String> {
                runner.commands.push(runner.create_cmd(None)?);
                runner.save_commands()?;
        
                return Err("Command created successfully.".into());
            } as Action),
            ("del".into(), |runner: &mut Self, identifier: &str| -> Result<(), String> {
                println!();

                let index = runner.get_command_index(identifier)?;
        
                let confirm = Input::<String>::new()
                    .with_prompt(format!("Are you sure you want to delete command #{}? (Y/N)", index + 1).as_str())
                    .interact_text()
                    .unwrap();
        
                if confirm != "y" && confirm != "yes" {
                    return Err("Deletion aborted.".into());
                }
                
                runner.commands.remove(index);
                runner.save_commands()?;
        
                return Err("Command delete successfully.".into());
            } as Action),
            ("edit".into(), |runner: &mut Self, identifier: &str| -> Result<(), String> {
                let index: usize = runner.get_command_index(identifier)?;
            
                runner.commands[index] = runner.create_cmd(Some(&runner.commands[index]))?;
                runner.save_commands()?;
        
                return Err("Command edited successfully.".into());
            } as Action),
            ("move".into(), |runner: &mut Self, identifier: &str| -> Result<(), String> {
                println!();

                let index = runner.get_command_index(identifier)?;
            
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
    
                return Err("Command moved successfully.".into());
            } as Action), 
            ("run".into(), |runner: &mut Self, identifier: &str| -> Result<(), String> {
                let cmd_info = &runner.commands[runner.get_command_index(identifier)?];
                
                screen::clear();
                println!("Running command {}.\n", cmd_info.cmd);

                #[cfg(target_os = "windows")]
                let status = Command::new("cmd").arg("/c").raw_arg(&cmd_info.cmd).status().map_err(|_| "Failed to run cmd.")?;
                #[cfg(not(target_os = "windows"))]
                let status = Command::new("sh").arg("-c").arg(&cmd_info.cmd).status().map_err(|_| "Failed to run sh.")?;
            
                println!(
                    "\nCommand exited with status code {}.",
                    status.code().map(|code| code.to_string()).unwrap_or("N/A".into())
                );

                return Ok(());
            } as Action)
        ]);
    
        let aliases: HashMap<String, String> = HashMap::from([
            ("a".into(), "add".into()),
            ("d".into(), "del".into()),
            ("e".into(), "edit".into()),
            ("m".into(), "move".into()),
            ("r".into(), "run".into())
        ]);

        let store = CommandStore::new();
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

    fn exec(&mut self, action: &str, identifier: &str) -> Result<(), String> {
        let resolved_action = match self.aliases.get(action) {
            Some(target_action) => target_action,
            _ => action
        };

        let func = self.actions.get(resolved_action)
            .ok_or(format!("Unknown action \"{}\".", action))?;

        func(self, identifier)
    }
    
    pub fn exec_from_args(&mut self, args: &Vec<String>) -> Result<(), String>  {
        let arg_count = args.len();
    
        if arg_count > 2 {
            return Err("Too many arguments.".into());
        }
    
        if arg_count == 2 {
            return self.exec(&args[0], &args[1])
        }
    
        if arg_count == 0 || args[0].is_empty() {
            return self.select_ui(None);
        }
    
        if self.actions.contains_key(&args[0]) || self.aliases.contains_key(&args[0]) {
            return self.exec(&args[0], "")
        }
    
        self.exec("run", &args[0])
    }

    pub fn show_error(&mut self, error: Option<String>) {
        if let Some(error) = error {
            println!("\n{}", error);
        }
    }

    pub fn cmd_ui(&mut self, args: &Vec<String>)  {
        screen::title();

        let res = self.exec_from_args(args).err();
        self.show_error(res);
    }
    
    pub fn select_ui(&mut self, prev_error: Option<String>) -> Result<(), String> {
        screen::title();
        println!();
    
        for (i, cmd_info) in self.commands.iter().enumerate() {
            println!("{}. {}", i + 1, arg_parser::escape_string(&cmd_info.name));
        }

        if self.commands.len() == 0 {
            println!("No commands saved. Run \"add\" to get started.");
        }

        self.show_error(prev_error);
    
        println!("\n<a[dd]|d[el]|e[dit]|m[ove]|r[un]|> <number or name>\n");
    
        let line = Input::<String>::new()
            .with_prompt("Action and identifier")
            .interact_text()
            .unwrap();
    
        self.exec_from_args(&arg_parser::string_to_arguments(line.trim()))
    }
}