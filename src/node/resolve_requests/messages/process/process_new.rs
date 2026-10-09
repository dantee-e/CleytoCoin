use crate::{
    chain::{block::Block, transaction::Transaction, utxo::OutPoint, Chain},
    error_handling::{CleytonError, TransactionError},
    node::{
        resolve_requests::{
            errors::HTTPResponseError,
            helpers::HTTPResult,
            messages::send::{notify_new_block, notify_new_transaction},
            methods::{Content, HTTPResponse},
        },
        ConnectedNodeInfo, NodeState,
    },
};
use serde_json::json;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

fn chain_conflict(chain: &Chain, challenging_block: Block) -> HTTPResult {
    let my_last_block = chain.blocks.last().expect("Node has an empty chain");
    if my_last_block.height() > challenging_block.height() {
        return Ok(HTTPResponse::OK(None));
    }

    // if competing chain is longer

    Ok(HTTPResponse::OK(None))
}

pub fn process_new_block(
    block: Block,
    chain: &mut Chain,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> HTTPResult {
    // Already have it. Checked against the chain in memory: blocks received
    // from other nodes are not written to disk
    if chain.blocks.iter().any(|b| b.hash() == block.hash()) {
        return Ok(HTTPResponse::OK(None));
    }

    // chain conflicst are handled when receiving block headers
    if block.previous_hash() == chain.get_last_hash() {
        chain.add_block(block);
    }

    Ok(HTTPResponse::OK(None))
}

fn validation_error_to_http(error: CleytonError) -> HTTPResponseError {
    match error {
        CleytonError::TransactionError(e) => match e {
            TransactionError::OpenSSLError(_) => {
                HTTPResponseError::InternalServerError(Some(
                    "Error in the OpenSSL library when verifying a transaction"
                        .to_string(),
                ))
            }
            // TODO Should move both of those to another error enum, maybe client and server errors
            e @ (TransactionError::InsufficientFunds
            | TransactionError::ConnectionError(_)) => {
                HTTPResponseError::InternalServerError(Some(e.to_string()))
            }
            e => HTTPResponseError::BadRequest(Some(e.to_string())),
        },
        CleytonError::OpenSslErrorStack(_) => {
            HTTPResponseError::InternalServerError(Some(
                "Error in the OpenSSL library when verifying a transaction"
                    .to_string(),
            ))
        }
        _ => HTTPResponseError::InternalServerError(None),
    }
}

pub fn process_new_transaction(
    transaction: Transaction,
    source: Option<&[u8]>,
    state: Arc<Mutex<NodeState>>,
) -> HTTPResult {
    let connected_nodes = {
        let mut state = state.lock().unwrap();

        if state.transactions_pool.contains(&transaction) {
            return Ok(HTTPResponse::OK(None));
        }

        // Inputs exist and are unspent, signatures are valid, inputs cover outputs
        state
            .chain
            .validate_transaction(&transaction)
            .map_err(validation_error_to_http)?;

        // Inputs must not be spent already by a transaction waiting in the pool
        let spent_in_pool: HashSet<OutPoint> = state
            .transactions_pool
            .iter()
            .flat_map(|t| t.inputs.iter().map(|input| input.prev))
            .collect();
        if transaction
            .inputs
            .iter()
            .any(|input| spent_in_pool.contains(&input.prev))
        {
            return Err(HTTPResponseError::BadRequest(Some(String::from(
                "Transaction spends an output already spent by a transaction in the pool",
            ))));
        }

        state.transactions_pool.push(transaction.clone());
        state.connected_nodes.clone()
    };

    // The lock is released before talking to the network
    notify_new_transaction(&transaction, &connected_nodes, source);

    Ok(HTTPResponse::OK(Some(Content::JSON(json!({
        "msg": "The transaction was added to the pool.",
        "status_code": "200"
    })))))
}
