use cleyto_coin::chain::transaction::Transaction;
use cleyto_coin::chain::utxo::{TransactionInput, TransactionOutput};
use cleyto_coin::chain::wallet::Wallet;
use cleyto_coin::chain::Chain;
use cleyto_coin::node::Message;
use cleyto_coin::{kill_node, run_server_thread_with_chain};
use std::thread;

use reqwest::blocking::Client;
use serial_test::serial;

const MESSAGES_URL: &str = "http://localhost:9473/messages";
const POOL_URL: &str = "http://localhost:9473/get-transaction-pool";

/// Chain whose genesis gives 1000 to a sender, and a valid signed transaction
/// spending them
fn funded_chain_and_transaction() -> (Chain, Transaction) {
    let (sender, sender_pk) = Wallet::new();
    let (receiver, _) = Wallet::new();
    let chain = Chain::new(vec![TransactionOutput::new(1000, &sender)]);

    let coins = chain.utxos_owned_by(&sender.to_public_key());
    let mut transaction = Transaction::new(
        TransactionInput::from_utxos(&coins),
        vec![TransactionOutput::new(1000, &receiver)],
    )
    .unwrap();
    sender_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
        .unwrap();

    (chain, transaction)
}

/// Posts `body` to /messages from `n` threads at once, returns the statuses
fn thread_post(n: u16, body: String) -> Vec<u16> {
    let client = Client::new();

    let handles: Vec<_> = (0..n)
        .map(|_| {
            let client = client.clone();
            let body = body.clone();
            thread::spawn(move || {
                client
                    .post(MESSAGES_URL)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .expect("POST failed")
                    .status()
                    .as_u16()
            })
        })
        .collect();

    // join().unwrap() so a panic inside a thread fails the test
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// GETs the transaction pool from `n` threads at once, returns the pools
fn thread_get(n: u16) -> Vec<Vec<Transaction>> {
    let client = Client::new();

    let handles: Vec<_> = (0..n)
        .map(|_| {
            let client = client.clone();
            thread::spawn(move || {
                let response = client.get(POOL_URL).send().expect("GET failed");
                assert_eq!(response.status(), 200);
                response.json::<Vec<Transaction>>().unwrap()
            })
        })
        .collect();

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Kills the node when dropped, so a failing assert doesn't leave it holding
/// the port for the next test
struct KillOnDrop(String);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = kill_node(self.0.clone());
    }
}

// Serial because every node binds the same port (Node::DEFAULT_PORT)
#[test]
#[serial(node_port)]
fn main() {
    let (chain, transaction) = funded_chain_and_transaction();

    let server_name = cleyto_coin::new_server_name();
    run_server_thread_with_chain(server_name.clone(), chain);
    let _guard = KillOnDrop(server_name.clone());

    // 10.000 breaks the os (client), but the server seems fine
    // Error accepting connection: Too many open files (os error 24)
    assert!(thread_get(10).iter().all(|pool| pool.is_empty()));

    // The same valid transaction, many times at once: all accepted, added once
    let body =
        serde_json::to_string(&Message::Transaction(transaction.clone()))
            .unwrap();
    assert!(thread_post(10, body).iter().all(|status| *status == 200));

    for pool in thread_get(10) {
        assert_eq!(pool, vec![transaction.clone()]);
    }

    // A transaction spending a coin the node doesn't know is rejected
    let invalid =
        serde_json::to_string(&Message::Transaction(Transaction::default()))
            .unwrap();
    assert!(thread_post(3, invalid).iter().all(|status| *status == 400));
    assert_eq!(thread_get(1)[0].len(), 1);
}
