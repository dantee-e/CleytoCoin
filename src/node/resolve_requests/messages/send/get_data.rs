use std::collections::HashSet;

use futures::future::select_ok;
use serde::de::DeserializeOwned;

use crate::{
    chain::{
        block::{Block, BlockHeader},
        transaction::{Transaction, TransactionHeader},
    },
    error_handling::{CleytoResult, CleytonError},
    node::{
        resolve_requests::messages::{GetDataMessage, Message},
        ConnectedNodeInfo,
    },
};

use super::send_message_to_connected_node;

/// Asks every connected node for the data at the same time, and returns the
/// first answer that passes `is_requested`. None if no node had it.
async fn get_data<T: DeserializeOwned>(
    message: Message,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    is_requested: impl Fn(&T) -> bool,
) -> Option<T> {
    let message = &message;
    let is_requested = &is_requested;

    let requests = connected_nodes.iter().map(|node| {
        Box::pin(async move {
            let body = send_message_to_connected_node(node, message).await?;
            let data: T = serde_json::from_str(&body)?;

            // Never trust what a node sends back
            if is_requested(&data) {
                CleytoResult::Ok(data)
            } else {
                Err(CleytonError::SendMessageError(
                    "Connected node sent data that was not requested"
                        .to_string(),
                ))
            }
        })
    });

    // select_ok panics on an empty iterator
    if connected_nodes.is_empty() {
        return None;
    }

    select_ok(requests).await.ok().map(|(data, _)| data)
}

pub async fn get_data_block(
    header: BlockHeader,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
) -> Option<Block> {
    let message = Message::GetData(GetDataMessage::Block(header.clone()));

    get_data(message, connected_nodes, |block: &Block| {
        block.to_header() == header
    })
    .await
}

pub async fn get_data_transaction(
    header: TransactionHeader,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
) -> Option<Transaction> {
    let message =
        Message::GetData(GetDataMessage::Transaction(header.clone()));

    get_data(message, connected_nodes, |transaction: &Transaction| {
        transaction.to_header() == header
    })
    .await
}
