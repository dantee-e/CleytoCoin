use std::sync::{Arc, Mutex};

use openssl::pkey::PKey;

use crate::node::{
    resolve_requests::{
        helpers::HTTPResult, messages::send::notify_new_node,
        methods::HTTPResponse,
    },
    ConnectedNodeInfo, NodeState,
};

pub fn process_new_node(
    new_node_message: ConnectedNodeInfo,
    state: Arc<Mutex<NodeState>>,
) -> HTTPResult {
    // Checks if you can reconstruct the public key
    let _ = PKey::public_key_from_pem(&new_node_message.public_key)?;

    let connected_nodes = {
        let mut state = state.lock().unwrap();
        if state.connected_nodes.contains(&new_node_message) {
            return Ok(HTTPResponse::OK(None));
        }
        let connected_nodes = state.connected_nodes.clone();
        state.connected_nodes.insert(new_node_message.clone());
        connected_nodes
    };

    // The lock is released before talking to the network, since the notified
    // nodes may message this one back
    notify_new_node(&new_node_message, &connected_nodes, None);

    Ok(HTTPResponse::OK(None))
}
