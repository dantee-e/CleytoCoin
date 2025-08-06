use cleyto_coin::node;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use cleyto_coin::chain::Chain;
use cleyto_coin::node::logger::Logger;
use reqwest::blocking::Client;

fn thread_post() {
    let url = "http://localhost:9473/"; // Replace with your server URL
    let client = Client::new();

    let mut handles = vec![];

    for i in 0..10000 {
        let client = client.clone();
        let url = url.to_string();

        let handle = thread::spawn(move || {
            let body = format!(r#"{{"message": "Hello #{}"}}"#, i);

            match client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body)
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

fn thread_get() {
    let url = "http://localhost:9473/get-transaction-pool"; // Replace with your server URL
    let client = Client::new();

    let mut handles = vec![];

    for i in 0..10000 {
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

#[test]
fn main() {
    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);
        let mut node = node::Node::new(Chain::new(), Arc::new(Logger::new()));
        node.run(true, rx, 0);
    });

    // thread_post();
    thread_get();

    tx.send(()).expect("Failed to send termination signal.");

    // Wait for the server thread to finish (this will block until the server thread terminates)
    server.join().expect("Server thread panicked.");
}
