use std::{collections::HashSet, path::PathBuf};

use crate::error_handling::{CleytoResult, CleytonError};

// pub const CONFIG_PATH: &str = ".config/cleyto_coin"; // /home/

pub struct ConfigPaths {
    pub(crate) servers_running_file: String,
    pub(crate) sockets_dir: String,
    #[allow(unused)]
    pub(crate) block_dir: String,
    pub(crate) last_block: String,
}
impl ConfigPaths {
    pub fn get() -> Self {
        ConfigPaths {
            servers_running_file: String::from(".config/cleyto_coin/servers_running.json"),
            sockets_dir: String::from(".config/cleyto_coin/sockets"),
            block_dir: String::from(".cleyto_coin/blocks"),
            last_block: String::from(".cleyto_coin/last_block"),
        }
    }
}

pub struct Config {
    #[allow(dead_code)]
    // This will be used for a cli command to delete servers
    pub(crate) servers_running: HashSet<String>,
    last_block: u32,
}
impl Config {
    pub fn get() -> Self {
        let last_block_path = PathBuf::from(ConfigPaths::get().last_block);
        let last_block = match std::fs::read_to_string(&last_block_path) {
            Ok(v) => str::parse::<u32>(&v).expect("Last block file has non uinteger value"),
            Err(_) => {
                std::fs::create_dir_all(last_block_path.parent().unwrap())
                    .expect("Could not create dir .cleyto_coin");
                std::fs::write(last_block_path, "0").expect("Could not create last_block file");
                0
            }
        };

        Config {
            servers_running: get_running_servers(),
            last_block,
        }
    }

    pub fn last_block(&self) -> u32 {
        self.last_block
    }
    pub fn update_last_block(&mut self, amount: i32) -> CleytoResult<()> {
        let result = {
            let intermediary = self.last_block as i32 + amount;
            if intermediary < 0 {
                return Err(CleytonError::LastBlockLessThanZero);
            }
            intermediary as u32
        };

        std::fs::write(ConfigPaths::get().last_block, result.to_string())
            .expect("Could not update last_block file");
        self.last_block = result;
        Ok(())
    }
}

pub const SERVERS_NAMES_LIST: [&str; 48] = [
    "Canada",
    "Mexico",
    "US",
    "Japan",
    "NZ",
    "Iran",
    "Argentina",
    "Uzbekistan",
    "SK",
    "Jordan",
    "Australia",
    "Brazil",
    "Ecuador",
    "Uruguay",
    "Paraguay",
    "Colombia",
    "Morocco",
    "Tunisia",
    "Egypt",
    "Algeria",
    "Ghana",
    "CV",
    "SA",
    "Qatar",
    "England",
    "SA",
    "Senegal",
    "CdI",
    "France",
    "Croatia",
    "Portugal",
    "Norway",
    "Germany",
    "Netherlands",
    "Austria",
    "Belgium",
    "Scotland",
    "Spain",
    "Switzerland",
    "Curacao",
    "Haiti",
    "Panama",
    "Sweden",
    "Turkiye",
    "Czechia",
    "BnH",
    "Congo",
    "Iraq",
];

pub fn new_server_name() -> String {
    let config = ConfigPaths::get();
    let contents = std::fs::read_to_string(config.servers_running_file).unwrap_or("[]".to_string());
    let servers_running: HashSet<String> = serde_json::from_str(&contents)
        .expect("Badly formed ~/.config/cleyto_coin/servers_running.json");

    for server in SERVERS_NAMES_LIST {
        if servers_running.contains(&String::from(server)) {
            continue;
        }

        return server.to_string();
    }
    "final".to_string()
}

pub fn get_running_servers() -> HashSet<String> {
    let config = ConfigPaths::get();
    let contents = std::fs::read_to_string(config.servers_running_file).unwrap_or("[]".to_string());
    serde_json::from_str(&contents)
        .expect("Badly formed ~/.config/cleyto_coin/servers_running.json")
}

pub fn add_name_to_running_servers(name: String) {
    let mut servers_running: HashSet<String> = get_running_servers();
    servers_running.insert(name);

    let config = ConfigPaths::get();
    if let Some(parent) = std::path::Path::new(&config.servers_running_file).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        config.servers_running_file,
        serde_json::to_string(&servers_running).unwrap(),
    )
    .unwrap();
}

pub fn remove_name_from_running_servers(name: String) {
    let mut servers_running: HashSet<String> = get_running_servers();
    servers_running.remove(&name);

    let config = ConfigPaths::get();

    if let Some(parent) = std::path::Path::new(&config.servers_running_file).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        config.servers_running_file,
        serde_json::to_string(&servers_running).unwrap(),
    )
    .unwrap();
}
