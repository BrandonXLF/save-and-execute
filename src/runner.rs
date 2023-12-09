use std::{collections::HashMap, process::Command, cell::RefCell};
use rustyline::DefaultEditor;
use crate::{arg_parser, store::{Store, CommandInfo}, screen};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

type Action = fn(&mut Runner, &str, bool) -> Result<(), String>;

pub struct Runner {
    actions: HashMap<&'static str, Action>,
    aliases: HashMap<&'static str, &'static str>,
    store: Store,
    commands: Vec<CommandInfo>,
    editor: RefCell<DefaultEditor>,
}

impl Runner {
    pub fn new() -> Self {
        let actions: HashMap<&str, Action> = HashMap::from([
            ("add", |runner, name, _| -> Result<(), String> {
                runner.commands.push(runner.create_cmd(
                    &CommandInfo { name: name.to_owned(), cmd: "".to_owned() },
                    false
                )?);
                runner.save_commands()?;
        
                println!("\nCommand created successfully.");

                return Ok(());
            } as Action),
            ("del", |runner, identifier, ui| -> Result<(), String> {
                let index = runner.get_command_index(identifier, ui)?;

                println!();
                let confirm = runner.prompt(
                    format!("Are you sure you want to delete command #{}? (Y/N) ", index + 1).as_str()
                ).to_lowercase();
        
                if confirm != "y" && confirm != "yes" {
                    return Err("Deletion aborted.".into());
                }
                
                runner.commands.remove(index);
                runner.save_commands()?;
        
                println!("\nCommand deleted successfully.");

                return Ok(());
            } as Action),
            ("edit", |runner, identifier, ui| -> Result<(), String> {
                let index: usize = runner.get_command_index(identifier, ui)?;
            
                runner.commands[index] = runner.create_cmd(&runner.commands[index], true)?;
                runner.save_commands()?;
        
                println!("\nCommand edited successfully.");

                return Ok(());
            } as Action),
            ("move", |runner, identifier, ui| -> Result<(), String> {
                let index = runner.get_command_index(identifier, ui)?;

                println!();
                let pos_line = runner.prompt("New position: ");

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
            ("run", |runner, identifier, ui| -> Result<(), String> {
                let cmd_info = &runner.commands[runner.get_command_index(identifier, ui)?];
                
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
            ("help", |_, _, ui| -> Result<(), String> {
                screen::show_help(ui);
                
                return Ok(());
            } as Action),
            ("list", |runner, _, ui| -> Result<(), String> {
                println!();

                for (i, cmd_info) in runner.commands.iter().enumerate() {
                    println!("{}. {}", i + 1, arg_parser::escape_string(&cmd_info.name));
                }
        
                if runner.commands.len() == 0 {
                    let add_cmd = if ui { "add" } else { "se add" };

                    println!("No commands saved. Run \"{}\" to get started.", add_cmd);
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
        let editor = DefaultEditor::new().unwrap();

        Self { actions, aliases, store, commands, editor: RefCell::new(editor) }
    }

    fn prompt(&self, prompt: &str) -> String {
        let mut editor = self.editor.borrow_mut();
        let input = editor.readline(prompt).unwrap();
        let _ = editor.add_history_entry(&input);

        return input;
    }

    fn prompt_with_initial(&self, prompt: &str, initial: &str) -> String {
        let mut editor = self.editor.borrow_mut();
        let input = editor.readline_with_initial(prompt, (initial, "")).unwrap();
        let _ = editor.add_history_entry(&input);

        return input;
    }

    fn create_cmd(&self, base: &CommandInfo, name_reserved: bool) -> Result<CommandInfo, String> {
        println!();

        let name = self.prompt_with_initial("Name: ", &base.name);
        let cmd = self.prompt_with_initial("Command: ", &base.cmd);

        if (!name_reserved || base.name.is_empty() || base.name != name) &&
            self.commands.iter().any(|command| command.name == name)
        {
            return Err(format!("Command with name \"{}\" already exists.", name));
        }
    
        Ok(CommandInfo { name, cmd })
    }

    fn save_commands(&self) -> Result<(), String> {
        self.store.save_commands(&self.commands)
    }

    fn get_command_index(&self, line: &str, ui: bool) -> Result<usize, String>  {
        if line.len() == 0 {
            return Err("No command specified for action.".into());
        }
    
        let help_cmd = if ui { "help" } else { "se help" };
        let maybe_number: Option<usize> = line.parse::<usize>().ok();

        let index = if let Some(number) = maybe_number {
            number - 1
        } else {
            self.commands.iter().position(|command| command.name == line)
                .ok_or(format!("Command with name \"{}\" not found. Run \"{}\" for help.", line, help_cmd))?
        };
    
        if index >= self.commands.len() {
            return Err(format!("Command #{} does not exist. Run \"{}\" for help.", line, help_cmd));
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

    fn exec_from_args(&mut self, args: &Vec<String>, ui: bool) -> Result<(), String> {
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

        return action.unwrap()(self, identifier, ui);
    }

    fn process_args(&mut self, args: &Vec<String>, ui: bool) {
        let res = self.exec_from_args(args, ui);

        if let Err(error) = res {
            println!("\nError: {}", error);
        }
    }

    pub fn run_args(&mut self, args: &Vec<String>) {
        screen::title();

        self.process_args(args, false);
    }

    pub fn show_runner(&mut self) {
        screen::title();

        loop {
            println!();

            let line = self.prompt("se > ");
            let args = &arg_parser::string_to_arguments(line.trim());

            if args.len() > 0 && args[0] == "exit" {
                break;
            }

            self.process_args(&arg_parser::string_to_arguments(line.trim()), true);
        }
    }
}