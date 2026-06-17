use std::collections::HashSet;

use openssl::pkey::PKey;

use crate::node::{
    resolve_requests::{helpers::HTTPResult, methods::HTTPResponse},
    ConnectedNodeInfo,
};

pub fn process_new_node(
    new_node_message: ConnectedNodeInfo,
    connected_nodes: &mut HashSet<ConnectedNodeInfo>,
) -> HTTPResult {
    // Checks if you can reconstruct the public key
    let _ = PKey::public_key_from_pem(&new_node_message.public_key)?;

    if !connected_nodes.contains(&new_node_message) {
        connected_nodes.insert(new_node_message);
    }

    Ok(HTTPResponse::OK(None))
}
