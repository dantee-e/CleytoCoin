use cleyto_coin::node;
use std::{sync::{mpsc, Arc, Mutex}, thread};

use reqwest::blocking::Client;
use serde::Serialize;
use rand::Rng;


#[derive(Serialize)]
struct RandomData {
    message: String,
    value: i32,
}

#[test]
fn post_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let data = RandomData {
        message: (0..10).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect(),
        value: rng.gen_range(1..=1000),
    };

    let client = Client::new();
    let res = client
        .post(format!("http://localhost:{}", node::Node::DEFAULT_PORT))
        .json(&data)
        .send()?;

    assert_eq!(res.status(), 200);
    
    Ok(())
}

#[test]
fn get_request() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();
    let res = client
        .get(format!("http://localhost:{}", node::Node::DEFAULT_PORT))
        .send()?;

    assert_eq!(res.status(), 200);
    let body = res.text()?;

    Ok(())
}

#[test]
fn post_and_get_request(){
    /* let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);
        node::Node::run(true, rx, 0);
    }); */

    for _ in 0..30 {
        match post_request() {
            Ok(_) => println!("Post successful"),
            Err(_) => println!("Post failed"),
        };
        match get_request() {
            Ok(_) => println!("Get successful"),
            Err(_) => println!("Get failed"),
        };
    }
    

    /* tx.send(()).expect("Failed to send termination signal.");

    // Wait for the server thread to finish (this will block until the server thread terminates)
    server.join().expect("Server thread panicked."); */
}




