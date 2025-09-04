use std::path::PathBuf;

use cleyto_coin::{generate, kill_server, run_server_thread, send};

const SENDER_PUBLIC_KEY_PATH: &str = "./wallets/sender/public.pem";
const SENDER_PRIVATE_KEY_PATH: &str = "./wallets/sender/private.pem";
const SENDER_PASSWORD: &str = "palmeiras";

const RECEIVER_PUBLIC_KEY_PATH: &str = "./wallets/receiver/public.pem";
const RECEIVER_PRIVATE_KEY_PATH: &str = "./wallets/receiver/private.pem";

#[test]
fn test_wallet_creation() {
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

#[tokio::test] //mark a function as a test.
async fn test_send_transaction() {
    test_wallet_creation();
    let sender_private_key_file = PathBuf::from(SENDER_PRIVATE_KEY_PATH);
    let sender_password = Some(String::from(SENDER_PASSWORD));

    let receiver_public_key_file = PathBuf::from(RECEIVER_PUBLIC_KEY_PATH);

    run_server_thread();

    send(
        None,
        Some(receiver_public_key_file),
        None,
        Some(sender_private_key_file),
        sender_password,
        100,
    )
    .await
    .unwrap();

    kill_server();
}
