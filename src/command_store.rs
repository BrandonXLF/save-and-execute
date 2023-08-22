use std::{fs::File, path::PathBuf, io::{BufReader, BufWriter}};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub cmd: String
}

pub struct  CommandStore {
    store_path: Option<PathBuf>
}

impl CommandStore {
    fn get_file_path() -> Option<PathBuf> {
        return Some(PathBuf::from_iter([dirs::home_dir()?.to_str()?, "se-commands.json"]));
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
        let path = self.store_path.clone().ok_or::<String>("Failed to get save file path.".into())?;
        let file = File::create(path).map_err(|_| "Failed to open save file.".to_owned())?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer(writer, &commands).map_err(|_| "Failed to write commands to save file.".to_owned())?;
        
        Ok(())
    }
}