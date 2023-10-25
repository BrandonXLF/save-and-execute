use std::{fs::{File, self}, path::PathBuf, io::{BufReader, BufWriter}};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub cmd: String
}

pub struct Store {
    store_path: Option<PathBuf>
}

impl Store {
    fn get_file_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?.to_owned();

        path.push("se");
        path.push("commands.json");

        return Some(path);
    }

    pub fn new() -> Self {
        Self {
            store_path: Self::get_file_path()
        }
    }

    fn read_store(&self) -> Option<Vec<CommandInfo>> {
        let file = File::open(&(self.store_path.clone()?)).ok()?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).ok()?
    }

    pub fn get_commands(&self) -> Vec<CommandInfo> {
        self.read_store().unwrap_or_default()
    }

    pub fn save_commands(&self, commands: &Vec<CommandInfo>) -> Result<(), String> {
        let path = self.store_path.as_ref().ok_or::<String>("Failed to get save file path.".into())?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| "Filed to open or create save directory.".to_owned())?;
        }

        let file = File::create(path).map_err(|_| "Failed to open save file.".to_owned())?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer(writer, &commands).map_err(|_| "Failed to write commands to save file.".to_owned())?;
        
        Ok(())
    }
}