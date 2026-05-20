use crate::{
    chain::{
        transaction::{self, Transaction, TransactionInfo},
        utxo::UTXO,
        wallet::{Wallet, WalletPK},
        Chain,
    },
    configs::{SERVERS_NAMES_LIST, SOCKETS_DIR},
    error_handling::{CleytoResult, CleytonError, TransactionError},
    node::ui::App,
};
use openssl::pkey::{PKey, Private, Public};
use reqwest::{Client, StatusCode};
use std::{
    io::Write,
    os::unix::net::UnixStream,
    path::PathBuf,
    process::{Command, Stdio},
};
use std::{sync::Arc, thread};

pub mod chain;
mod configs;
pub mod error_handling;
pub mod node;

pub use configs::{add_name_to_running_servers, new_server_name, remove_name_from_running_servers};

async fn send_transaction(transaction: transaction::Transaction) -> Result<(), TransactionError> {
    let client = Client::new();

    let transaction_json = transaction.serialize();

    // Send the POST request
    let response = client
        .post("http://localhost:9473/submit-transaction")
        .header("Content-Type", "application/json")
        .body(transaction_json)
        .send()
        .await
        .unwrap();

    // Check the response status
    let status = response.status();
    let response_body = response.text().await.unwrap();
    match status {
        StatusCode::OK => Ok(()),

        _ => Err(TransactionError::ConnectionError(format!(
            "Error: {status}\n{response_body}"
        ))),
    }
}

fn read_key_string_or_file(string: &Option<String>, file: &Option<PathBuf>) -> String {
    if let Some(s) = string {
        s.clone()
    } else if let Some(path) = file {
        std::fs::read_to_string(path).expect("Failed to read key file")
    } else {
        panic!("Key not provided");
    }
}

pub fn generate(private_key_file: &PathBuf, public_key_file: &PathBuf, password: &Option<String>) {
    let (wallet, walletpk) = Wallet::new();

    let parents = [
        private_key_file
            .parent()
            .expect("The path provided has no parent"),
        public_key_file
            .parent()
            .expect("The path provided has no parent"),
    ];

    // Checks to see if parents exist. if not, creates them
    for parent in parents {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Could not create parent directory");
        }
    }

    if let Some(password) = password {
        std::fs::write(private_key_file, walletpk.to_pem_with_password(password))
            .expect("Could not write new wallet's private key to file");
    } else {
        std::fs::write(private_key_file, walletpk.to_pem())
            .expect("Could not write new wallet's private key to file");
    }

    std::fs::write(public_key_file, wallet.to_pem())
        .expect("Could not write new wallet's public key to file");
}

pub async fn send(
    recipient_key: Option<String>,
    recipient_key_file: Option<PathBuf>,
    sender_key: Option<String>,
    sender_key_file: Option<PathBuf>,
    password: Option<String>,
    amount: u64,
) -> Result<(), TransactionError> {
    let recipient_key_str = read_key_string_or_file(&recipient_key, &recipient_key_file);
    let sender_key_str = read_key_string_or_file(&sender_key, &sender_key_file);

    // convert to PKey objects
    let sender_pkey: PKey<Private> = if let Some(password) = password {
        PKey::private_key_from_pem_passphrase(sender_key_str.as_bytes(), password.as_bytes())
            .expect("Failed to parse sender private key")
    } else {
        PKey::private_key_from_pem(sender_key_str.as_bytes())
            .expect("Failed to parse sender private key")
    };

    let recipient_pkey: PKey<Public> = PKey::public_key_from_pem(recipient_key_str.as_bytes())
        .expect("Failed to parse recipient public key");

    // create wallets
    let sender_wallet = WalletPK::from(sender_pkey);
    let recipient_wallet = Wallet::from(recipient_pkey);

    // find input utxos
    let input_utxos = match sender_wallet.public_wallet().get_utxos(amount) {
        Ok(vec) => vec,
        Err(_) => return Err(TransactionError::InsufficientFunds),
    };

    // Create output UTXOs
    let input_sum = UTXO::sum(&input_utxos);
    let rec_utxo = UTXO::new(amount, recipient_wallet.clone());
    let change_utxo = UTXO::new(input_sum - amount, sender_wallet.public_wallet());
    let output_utxos = vec![change_utxo, rec_utxo];

    // create transaction info
    let transaction_info = TransactionInfo::new(input_utxos, output_utxos);

    // sign the transaction
    let signature = sender_wallet
        .sign_transaction(&transaction_info)
        .expect("Failed on signing of transaction");

    let transaction = Transaction::new(
        sender_wallet.public_wallet(),
        recipient_wallet,
        transaction_info,
        signature,
    )
    .inspect_err(|e| eprintln!("Failed creating the transaction: {e}"))
    .unwrap();

    send_transaction(transaction).await
}

pub fn run_server_with_gui(server_name: String) -> color_eyre::Result<()> {
    // Channel to kill thread
    // let rx = Arc::new(Mutex::new(rx));

    let (mut node, logger) = node::Node::new(Chain::new(), server_name);

    let node_name = node.name.to_string();
    // Run server thread
    let server = thread::spawn(move || {
        // let rx = Arc::clone(&rx);

        node.run(true, 0);
    });

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new(Arc::clone(&logger), node::Node::DEFAULT_PORT).run(terminal);
    ratatui::restore();

    // Quits server
    kill_node(node_name).expect("Failed to kill node");

    server.join().unwrap();
    result
}

/// Spawns thread with server and return the channel that sends the kill signal
/// Mostly useful for testing
/// Returns the created server's name, to enable killing it later
pub fn run_server_thread(server_name: String) -> String {
    let (mut node, _) = node::Node::new(Chain::new(), server_name.to_string());

    thread::spawn(move || {
        node.run(true, 0);
    });

    server_name
}

pub fn run_server(server_name: String) {
    let (mut node, _) = node::Node::new(Chain::new(), server_name);
    node.run(true, 0);
}

pub fn run_server_new_process(server_name: String) {
    #[allow(clippy::zombie_processes)]
    let child = Command::new(std::env::current_exe().unwrap())
        .arg("start")
        .arg("--blocking")
        .arg("--name")
        .arg(&server_name)
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("Failed to start server process");
    println!("Spawned process with pid {}", child.id());
}

/// Sends the kill signal to the server
pub fn kill_node(node: String) -> CleytoResult<()> {
    let socket_path = PathBuf::from(format!("{}/{}.sock:", SOCKETS_DIR, node));

    println!("Socket exists? {}", socket_path.exists());
    if !socket_path.exists() {
        return Ok(());
    }

    let mut stream = match UnixStream::connect(socket_path.clone()) {
        Ok(s) => s,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                let _ = std::fs::remove_file(socket_path);
                println!("Deleted empty socket file");
                return Ok(());
            }

            return Err(CleytonError::KillServerError(e.to_string()));
        }
    };

    stream
        .write_all("kill".as_bytes())
        .expect("Error writing kill signal");

    println!("Killed node {}", node);

    std::fs::remove_file(socket_path).map_err(|e| CleytonError::KillServerError(e.to_string()))?;
    remove_name_from_running_servers(node);

    Ok(())
}

pub fn kill_all_nodes() {
    for server in SERVERS_NAMES_LIST {
        // I don't care if it fails
        let _ = kill_node(server.to_string());
    }
}
