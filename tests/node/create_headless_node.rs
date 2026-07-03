use std::{thread, time::Duration};

use cleyto_coin::{kill_node, new_server_name, run_server_thread};

#[test]
fn run_and_kill_node() {
    let server_name = new_server_name();
    run_server_thread(server_name.clone());
    thread::sleep(Duration::from_millis(100));
    kill_node(server_name).unwrap();
}
