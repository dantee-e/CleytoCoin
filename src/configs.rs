use std::collections::HashSet;

// pub const CONFIG_PATH: &str = ".config/cleyto_coin"; // /home/
const SERVERS_RUNNING_FILE: &str = ".config/cleyto_coin/servers_running.json";
pub const SOCKETS_DIR: &str = ".config/cleyto_coin/sockets";

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
    let contents = std::fs::read_to_string(SERVERS_RUNNING_FILE).unwrap_or("[]".to_string());
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
    let contents = std::fs::read_to_string(SERVERS_RUNNING_FILE).unwrap_or("[]".to_string());
    serde_json::from_str(&contents)
        .expect("Badly formed ~/.config/cleyto_coin/servers_running.json")
}

pub fn add_name_to_running_servers(name: String) {
    let mut servers_running: HashSet<String> = get_running_servers();
    servers_running.insert(name);

    if let Some(parent) = std::path::Path::new(SERVERS_RUNNING_FILE).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        SERVERS_RUNNING_FILE,
        serde_json::to_string(&servers_running).unwrap(),
    )
    .unwrap();
}

pub fn remove_name_from_running_servers(name: String) {
    let mut servers_running: HashSet<String> = get_running_servers();
    servers_running.remove(&name);

    if let Some(parent) = std::path::Path::new(SERVERS_RUNNING_FILE).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        SERVERS_RUNNING_FILE,
        serde_json::to_string(&servers_running).unwrap(),
    )
    .unwrap();
}
