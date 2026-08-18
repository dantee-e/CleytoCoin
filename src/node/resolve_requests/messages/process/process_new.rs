use std::collections::HashSet;

use serde_json::json;

use crate::{
    chain::{block::Block, transaction::Transaction, Chain},
    error_handling::{TransactionDeserializeError, TransactionError},
    node::{
        data::check_block_is_registered_by_hash,
        resolve_requests::{
            errors::HTTPResponseError,
            helpers::HTTPResult,
            messages::send::{notify_new_block, notify_new_transaction},
            methods::{Content, HTTPResponse},
        },
        ConnectedNodeInfo,
    },
};

pub fn process_new_block(
    block: Block,
    chain: &mut Chain,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> HTTPResult {
    if check_block_is_registered_by_hash(&block.hash()) {
        return Ok(HTTPResponse::OK(None));
    }
    if block.previous_hash() == chain.get_last_hash() {
        notify_new_block(&block, connected_nodes, source);
        chain.add_block(block);
        Ok(HTTPResponse::OK(None))
    } else {
        unimplemented!("Chain conflict!");
    }
}

pub fn process_new_transaction(
    transaction: Transaction,
    transaction_pool: &mut Vec<Transaction>,
    connected_nodes: &HashSet<ConnectedNodeInfo>,
    source: Option<&[u8]>,
) -> HTTPResult {
    println!("Inside process_new_transaction");
    if transaction_pool.contains(&transaction) {
        return Ok(HTTPResponse::OK(None));
    }

    match Transaction::check_transaction(&transaction) {
        Ok(tx) => tx,
        Err(e) => {
            return match e {
                TransactionDeserializeError::InsufficientFunds => {
                    Err(HTTPResponseError::InvalidBody(None))
                }
                TransactionDeserializeError::MalformedTransaction => {
                    Err(HTTPResponseError::InvalidBody(None))
                }
                TransactionDeserializeError::SerdeError(_) => {
                    Err(HTTPResponseError::InvalidBody(None))
                }
            }
        }
    }
    match transaction.verify_signature() {
        Ok(()) => {}
        Err(e) => {
            return match e {
                TransactionError::OpenSSLError(_) => Err(HTTPResponseError::InternalServerError(
                    Some("Error in the OpenSSL library when verifying a transaction".to_string()),
                )),
                TransactionError::ValidationError => Err(HTTPResponseError::BadRequest(Some(
                    "Transaction submitted with \
                    invalid signature"
                        .to_string(),
                ))),
                TransactionError::InsufficientInputs => Err(HTTPResponseError::BadRequest(Some(
                    "Transaction's outputs are bigger that its inputs".to_string(),
                ))),
                // TODO Should move both of those to another error enum, maybe client and server errors
                TransactionError::InsufficientFunds => panic!("Not the server's problem"),
                TransactionError::ConnectionError(_) => panic!("Not the server's problem"),
            };
        }
    };

    notify_new_transaction(&transaction, connected_nodes, source);
    transaction_pool.push(transaction);

    Ok(HTTPResponse::OK(Some(Content::JSON(json!({
        "msg": "The transaction was added to the pool.",
        "status_code": "200"
    })))))
}
