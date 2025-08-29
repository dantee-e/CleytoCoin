use std::{thread, time::Duration};

use cleyto_coin::{kill_server, run_server_thread};

#[test]
fn run_and_kill_node() {
    run_server_thread();
    thread::sleep(Duration::from_millis(100));
    kill_server();
}
