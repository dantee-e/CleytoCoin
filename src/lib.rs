use crate::{
    chain::{
        transaction::{self, Transaction, TransactionInfo},
        wallet::{Wallet, WalletPK},
        Chain,
    },
    node::ui::App,
};
use openssl::pkey::{PKey, Private, Public};
use reqwest::Client;
use std::path::PathBuf;
use std::{
    sync::{mpsc, mpsc::Sender, Arc, Mutex},
    thread,
};

pub mod chain;
pub mod node;

async fn send_transaction(transaction: transaction::Transaction) {
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

    // Print the response
    println!("Response Status: {}", status);
    println!("Response Body: {}", response_body);
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
    amount: i64,
) {
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

    // create transaction info
    let transaction_info = TransactionInfo::new(amount);

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

    send_transaction(transaction).await;
}

pub fn run_server_with_gui() -> color_eyre::Result<()> {
    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    let (mut node, logger) = node::Node::new(Chain::new());

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);

        node.run(true, rx, 0);
    });

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new(Arc::clone(&logger), node::Node::DEFAULT_PORT).run(terminal);
    ratatui::restore();

    // Quits server
    tx.send(())?;

    server.join().unwrap();
    result
}

/// Spawns thread with server and return the channel that sends the kill signal
pub fn run_server() -> Sender<()> {
    let (tx, rx) = mpsc::channel::<()>();
    let rx = Arc::new(Mutex::new(rx));
    let (mut node, _) = node::Node::new(Chain::new());
    thread::spawn(move || {
        node.run(true, rx, 0);
    });
    tx
}

/// Sends the kill signal to the server
pub fn kill_server(tx: Sender<()>) {
    while tx.send(()).is_ok() {}
}
