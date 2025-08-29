use cleyto_coin::chain::transaction::{Transaction, TransactionInfo};
use cleyto_coin::chain::wallet::Wallet;
use cleyto_coin::node;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use cleyto_coin::chain::Chain;
use reqwest::blocking::Client;

fn thread_post(n: u16) {
    let url = "http://localhost:9473/submit-transaction"; // Replace with your server URL
    let client = Client::new();

    let mut handles = vec![];

    let (wallet1, wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    let transaction_info = TransactionInfo::new(105);

    let signature = match wallet1_pk.sign_transaction(&transaction_info) {
        Ok(value) => value,
        Err(e) => panic!("Error creating signed message: {e}"),
    };

    let new_transaction = Transaction::new(wallet1, wallet2, transaction_info, signature).unwrap();
    let json_transaction = new_transaction.serialize();
    println!("json_transaction is:\n{}", json_transaction);

    for i in 0..n {
        let client = client.clone();
        let url = url.to_string();
        let transaction_copy = json_transaction.clone();

        let handle = thread::spawn(move || {
            match client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(transaction_copy)
                .send()
            {
                Ok(resp) => {
                    assert_eq!(resp.status(), 200);
                    println!("Thread #{i}: {}", resp.status())
                }
                Err(err) => eprintln!("Thread #{i} failed: {err}"),
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}

fn thread_get(n: u16) {
    let url = "http://localhost:9473/get-transaction-pool"; // Replace with your server URL
    let client = Client::new();

    let mut handles = vec![];

    for i in 0..n {
        let client = client.clone();
        let url = url.to_string();

        let handle = thread::spawn(move || {
            match client
                .get(&url)
                .header("Content-Type", "application/json")
                .send()
            {
                Ok(resp) => {
                    assert_eq!(resp.status(), 200);
                    println!("Thread #{i}: {:#?}", resp.status())
                }
                Err(err) => eprintln!(
                    "Thread #{i} failed (timeout) (connect): {} {}",
                    err.is_timeout(),
                    err.is_connect()
                ),
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}

#[test]
fn main() {
    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);
        let (mut node, _) = node::Node::new(Chain::new());
        node.run(true, rx, 0);
    });

    // 10.000 breaks the os (client), but the server seems fine
    // Error accepting connection: Too many open files (os error 24)
    thread_get(10);
    thread_post(10);

    tx.send(()).expect("Failed to send termination signal.");

    // Wait for the server thread to finish (this will block until the server thread terminates)
    server.join().expect("Server thread panicked.");
}
