use std::{
    collections::HashSet,
    io::Write,
    net::{SocketAddr, TcpStream},
};

use crate::{
    chain::{
        block::{Block, BlockHeader},
        transaction::{Transaction, TransactionHeader},
        Chain,
    },
    error_handling::{CleytoResult, CleytonError},
    node::{
        resolve_requests::{
            helpers::HTTPResult,
            messages::{message_struct::GetDataMessage, Message},
            methods::{Content, HTTPResponse},
        },
        ConnectedNodeInfo,
    },
};

// TODO maybe these two could avoid sending messages back to wherever they came from in the first place
pub fn notify_new_transaction(
    transaction: Transaction,
    connected_nodes: HashSet<ConnectedNodeInfo>,
) -> Vec<CleytoResult<()>> {
    let header = transaction.to_header();

    let message = Message::CheckTransaction(header);

    connected_nodes
        .iter()
        .map(|node| {
            let mut stream = TcpStream::connect(node.address).unwrap();
            if let Err(e) = stream
                .write_all(serde_json::to_string(&message).unwrap().as_bytes())
            {
                Err(CleytonError::SendMessageError(e.to_string()))
            } else {
                Ok(())
            }
        })
        .collect()
}

// TODO maybe these two could avoid sending messages back to wherever they came from in the first place
pub fn notify_new_block(
    block: Block,
    connected_nodes: HashSet<ConnectedNodeInfo>,
) -> Vec<CleytoResult<()>> {
    let header = block.to_header();

    let message = Message::CheckBlock(header);

    connected_nodes
        .iter()
        .map(|node| {
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
