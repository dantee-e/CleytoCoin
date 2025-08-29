use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use cleyto_coin::{chain::Chain, kill_server, node};

#[test]
fn run_and_kill_node() {
    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    let (mut node, _) = node::Node::new(Chain::new());

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);

        node.run(true, rx, 0);
    });
    let killer_server = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        kill_server(tx);
    });

    killer_server.join().unwrap();
    server.join().unwrap();
}
