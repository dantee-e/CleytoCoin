use cleyto_coin::{generate, send};
use std::path::PathBuf;
use structopt::StructOpt;

/// CLI for key management and transactions
#[derive(Debug, StructOpt)]
#[structopt(name = "cleyto-coin-wallet")]
enum Args {
    /// Generate a new keypair
    Generate {
        /// Where to store the generated private key
        #[structopt(long, parse(from_os_str), default_value = "./private.pem")]
        private_key_file: PathBuf,

        /// Where to store the generated public key
        #[structopt(long, parse(from_os_str), default_value = "./public.pem")]
        public_key_file: PathBuf,

        #[structopt(long, short)]
        password: Option<String>,
    },

    /// Send a transaction
    Send {
        /// Recipient’s public key as a string
        #[structopt(long = "recipient-key", required_unless = "recipient-key-file")]
        recipient_key: Option<String>,

        /// Recipient’s public key from a file
        #[structopt(
            long = "recipient-key-file",
            parse(from_os_str),
            required_unless = "recipient-key"
        )]
        recipient_key_file: Option<PathBuf>,

        /// Sender’s private key as a string
        #[structopt(long = "sender-key", short = "sk", required_unless = "sender-key-file")]
        sender_key: Option<String>,

        /// Sender’s private key from a file
        #[structopt(
            long = "sender-key-file",
            parse(from_os_str),
            required_unless = "sender-key"
        )]
        sender_key_file: Option<PathBuf>,

        /// Password used to encode the private key
        #[structopt(long, short)]
        password: Option<String>,

        /// Transaction amount
        #[structopt(long, short)]
        amount: i64,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    match args {
        Args::Generate {
            private_key_file,
            public_key_file,
            password,
        } => generate(&private_key_file, &public_key_file, &password),
        Args::Send {
            recipient_key,
            recipient_key_file,
            sender_key,
            sender_key_file,
            password,
            amount,
        } => {
            send(
                recipient_key,
                recipient_key_file,
                sender_key,
                sender_key_file,
                password,
                amount,
            )
            .await
        }
    }
}
