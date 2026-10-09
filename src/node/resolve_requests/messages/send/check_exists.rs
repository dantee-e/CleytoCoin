use std::collections::HashSet;

use futures::future::join_all;

use crate::{
    chain::{block::Block, transaction::Transaction},
    error_handling::CleytoResult,
    node::{resolve_requests::messages::Message, ConnectedNodeInfo},
};

use super::send_message_to_connected_node;

/// Sends the message to every connected node at the same time, skipping the
/// one that sent it to us
fn send_message(
    message: Message,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> Vec<CleytoResult<()>> {
    let source_pkey = source.unwrap_or_default();

    let sends = connected_nodes
        .iter()
        .filter(|node| node.public_key != source_pkey)
        .map(|node| async {
            send_message_to_connected_node(node, &message).await.map(|_| ())
        });

    // The callers run in the node's thread pool, outside of any async runtime
    async_std::task::block_on(join_all(sends))
}

/// source is public key of message sender
pub fn notify_new_transaction(
    transaction: &Transaction,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> Vec<CleytoResult<()>> {
    let header = transaction.to_header();

    let message = Message::CheckTransaction(header);

    send_message(message, connected_nodes, source)
}

/// source is public key of message sender
pub fn notify_new_block(
    block: &Block,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> Vec<CleytoResult<()>> {
    let header = block.to_header();

    let message = Message::CheckBlock(header);

    send_message(message, connected_nodes, source)
}

pub fn notify_new_node(
    node: &ConnectedNodeInfo,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> Vec<CleytoResult<()>> {
    let message = Message::NewNode(node.clone());

    send_message(message, connected_nodes, source)
}
