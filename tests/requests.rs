use cleyto_coin::chain::transaction::{Transaction, TransactionInfo};
use cleyto_coin::chain::utxo::UTXO;
use cleyto_coin::chain::wallet::Wallet;
use cleyto_coin::{kill_server, run_server_thread};
use std::thread;

use reqwest::blocking::Client;

fn thread_post(n: u16) {
    let url = "http://localhost:9473/submit-transaction"; // Replace with your server URL
    let client = Client::new();

    let mut handles = vec![];

    let (wallet1, wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    let input_utxos = vec![
        UTXO::new(1000, wallet1.clone()),
        UTXO::new(2000, wallet1.clone()),
    ];
    let output_utxos = vec![
        UTXO::new(2500, wallet2.clone()),
        UTXO::new(500, wallet2.clone()),
    ];
    let transaction_info: TransactionInfo = TransactionInfo::new(input_utxos, output_utxos);

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
    // Channel to kill thread

    // Run server thread
    run_server_thread();

    // 10.000 breaks the os (client), but the server seems fine
    // Error accepting connection: Too many open files (os error 24)
    thread_get(10);
    thread_post(10);

    kill_server();
}
