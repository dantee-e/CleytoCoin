use std::path::PathBuf;

use cleyto_coin::{chain::wallet::Wallet, generate};

const SENDER_PUBLIC_KEY_PATH: &str = "./wallets/sender/public.pem";
const SENDER_PRIVATE_KEY_PATH: &str = "./wallets/sender/private.pem";
const SENDER_PASSWORD: &str = "palmeiras";

const RECEIVER_PUBLIC_KEY_PATH: &str = "./wallets/receiver/public.pem";
const RECEIVER_PRIVATE_KEY_PATH: &str = "./wallets/receiver/private.pem";

#[test]
fn create_null_wallet() {
    Wallet::null_wallet();
}

#[test]
fn test_wallet_creation_cli() {
    let sender_private_key_file = PathBuf::from(SENDER_PRIVATE_KEY_PATH);
    let sender_public_key_file = PathBuf::from(SENDER_PUBLIC_KEY_PATH);
    let sender_password = Some(String::from(SENDER_PASSWORD));
    generate(
        &sender_private_key_file,
        &sender_public_key_file,
        &sender_password,
    );

    let receiver_private_key_file = PathBuf::from(RECEIVER_PRIVATE_KEY_PATH);
    let receiver_public_key_file = PathBuf::from(RECEIVER_PUBLIC_KEY_PATH);
    generate(&receiver_private_key_file, &receiver_public_key_file, &None);
}

#[test]
fn test_wallet_creation() {
    let wallet = Wallet::new();
    wallet.1.save_as("test_wallet", Some("palmeiras"));
}
