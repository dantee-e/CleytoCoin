use std::{collections::HashSet, io::Write, net::TcpStream};

use crate::{
    chain::{block::Block, transaction::Transaction},
    error_handling::{CleytoResult, CleytonError},
    node::{resolve_requests::messages::Message, ConnectedNodeInfo},
};

fn send_message(
    message: Message,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> Vec<CleytoResult<()>> {
    let source_pkey = if let Some(pkey) = source {
        pkey
    } else {
        &vec![]
    };

    connected_nodes
        .iter()
        .map(|node| {
            if node.public_key == source_pkey {
                return Ok(());
            }
            let mut stream = TcpStream::connect(node.address).unwrap();
            if let Err(e) =
                stream.write_all(serde_json::to_string(&message)?.as_bytes())
            {
                Err(CleytonError::SendMessageError(e.to_string()))
            } else {
                Ok(())
            }
        })
        .collect()
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
