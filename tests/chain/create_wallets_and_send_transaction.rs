use std::path::PathBuf;

use cleyto_coin::{
    chain::wallet::{Wallet, WalletPK},
    generate,
};
use openssl::pkey::PKey;

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

    // The private keys load back and match the public keys written next to them
    let sender_pk = PKey::private_key_from_pem_passphrase(
        &std::fs::read(&sender_private_key_file).unwrap(),
        SENDER_PASSWORD.as_bytes(),
    )
    .unwrap();
    let sender_public =
        Wallet::from(std::fs::read_to_string(&sender_public_key_file).unwrap());
    assert_eq!(WalletPK::from(sender_pk).public_wallet(), sender_public);

    let receiver_pk = PKey::private_key_from_pem(
        &std::fs::read(&receiver_private_key_file).unwrap(),
    )
    .unwrap();
    let receiver_public = Wallet::from(
        std::fs::read_to_string(&receiver_public_key_file).unwrap(),
    );
    assert_eq!(WalletPK::from(receiver_pk).public_wallet(), receiver_public);
}

#[test]
fn test_wallet_creation() {
    let (wallet, wallet_pk) = Wallet::new();
    wallet_pk.save_as("test_wallet", Some("palmeiras"));

    let loaded = WalletPK::from_pem(
        &String::from("test_wallet"),
        Some(String::from("palmeiras")),
    )
    .unwrap()
    .expect("saved wallet not found");
    assert_eq!(loaded.public_wallet(), wallet);
}
