use cleyto_coin::chain::{
    transaction::{self, Transaction, TransactionInfo},
    wallet::{Wallet, WalletPK},
};
use openssl::pkey::{PKey, Private};
use reqwest::Client;
use std::path::PathBuf;
use structopt::StructOpt;

/// CLI for key management and transactions
#[derive(Debug, StructOpt)]
#[structopt(name = "cleyto-coin-wallet")]
enum Args {
    /// Generate a new keypair
    Generate {
        /// Where to store the generated private key
        #[structopt(long, parse(from_os_str), default_value = "private.pem")]
        private_key_file: PathBuf,

        /// Where to store the generated public key
        #[structopt(long, parse(from_os_str), default_value = "public.pem")]
        public_key_file: PathBuf,

        #[structopt(long, short)]
        password: Option<String>,
    },

    /// Send a transaction
    Send {
        /// Recipient’s public key as a string
        #[structopt(
            long = "recipient-key",
            short = "r",
            required_unless = "recipient-key-file"
        )]
        recipient_key: Option<String>,

        /// Recipient’s public key from a file
        #[structopt(
            long = "recipient-key-file",
            parse(from_os_str),
            required_unless = "recipient-key"
        )]
        recipient_key_file: Option<PathBuf>,

        /// Sender’s private key as a string
        #[structopt(long = "sender-key", short = "s", required_unless = "sender-key-file")]
        sender_key: Option<String>,

        /// Sender’s private key from a file
        #[structopt(
            long = "sender-key-file",
            parse(from_os_str),
            required_unless = "sender-key"
        )]
        sender_key_file: Option<PathBuf>,

        /// Transaction amount
        #[structopt(long, short = "a")]
        amount: i64,
    },
}

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

pub async fn send(
    recipient_key: Option<String>,
    recipient_key_file: Option<PathBuf>,
    sender_key: Option<String>,
    sender_key_file: Option<PathBuf>,
    amount: i64,
) {
    let recipient_key_str = read_key_string_or_file(&recipient_key, &recipient_key_file);
    let sender_key_str = read_key_string_or_file(&sender_key, &sender_key_file);

    // convert to PKey objects
    let sender_pkey: PKey<Private> = PKey::private_key_from_pem(sender_key_str.as_bytes())
        .expect("Failed to parse sender private key");

    let recipient_pkey: PKey<openssl::pkey::Public> =
        PKey::public_key_from_pem(recipient_key_str.as_bytes())
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

fn generate(private_key_file: PathBuf, public_key_file: PathBuf, password: Option<String>) {
    let (wallet, walletpk) = Wallet::new();
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

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    match args {
        Args::Generate {
            private_key_file,
            public_key_file,
            password,
        } => generate(private_key_file, public_key_file, password),
        Args::Send {
            recipient_key,
            recipient_key_file,
            sender_key,
            sender_key_file,
            amount,
        } => {
            send(
                recipient_key,
                recipient_key_file,
                sender_key,
                sender_key_file,
                amount,
            )
            .await
        }
    }
}
