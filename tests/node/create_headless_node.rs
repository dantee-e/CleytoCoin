use cleyto_coin::{kill_node, new_server_name, run_server_thread};
use serial_test::serial;

// Serial because every node binds the same port (Node::DEFAULT_PORT)
#[test]
#[serial(node_port)]
fn run_and_kill_node() {
    let server_name = new_server_name();
    run_server_thread(server_name.clone());
    kill_node(server_name).unwrap();
}
