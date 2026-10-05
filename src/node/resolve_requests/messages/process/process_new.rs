use crate::{
    chain::{
        block::Block,
        transaction::Transaction,
        utxo::{TransactionInput, UTXO},
        Chain,
    },
    error_handling::{
        CleytonError, TransactionDeserializeError, TransactionError,
    },
    node::{
        data::check_block_is_registered_by_hash,
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
    source: Option<&[u8]>,
    state: Arc<Mutex<NodeState>>,
) -> HTTPResult {
    let transaction_pool = &mut state.lock().unwrap().transactions_pool.clone();
    let connected_nodes = &state.lock().unwrap().connected_nodes.clone();
    let wallets = &state.lock().unwrap().chain.wallets.clone();

    if transaction_pool.contains(&transaction) {
        return Ok(HTTPResponse::OK(None));
    }

    match transaction.verify_signatures() {
        Ok(_) => (),
        Err(e) => {
            return match e {
                CleytonError::TransactionDeserializeError(e) => match e {
                    TransactionDeserializeError::InsufficientFunds => {
                        Err(HTTPResponseError::InvalidBody(None))
                    }
                    TransactionDeserializeError::MalformedTransaction => {
                        Err(HTTPResponseError::InvalidBody(None))
                    }
                    TransactionDeserializeError::SerdeError(_) => {
                        Err(HTTPResponseError::InvalidBody(None))
                    }
                },
                CleytonError::TransactionError(e) => match e {
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
                        TransactionError::UnsignedInput => todo!(),
                }
                _ => {
                    return Err(HTTPResponseError::InternalServerError(None))
                }
            };
        }
    }

    // check if the wallets have the utxos that they claim to have
    let mut seen: Vec<(_, _)> = Vec::new();

    for input in transaction.inputs.iter() {
        let key = (input.txid, input.index);
        if seen.contains(&key) {
            return Err(HTTPResponseError::InvalidBody(Some(String::from(
                "Duplicate input in transaction",
            ))));
        }
        seen.push(key);

        println!("Wallets length is {}", wallets.len());

        let wallet =
            wallets.iter().find(|w| **w == input.owner).ok_or_else(|| {
                HTTPResponseError::InvalidBody(Some(String::from(
                    "Wallet owning the input not found",
                )))
            })?;

        println!(
            "available_utxos len is {}, utxo is {:#?}",
            wallet.available_utxos.iter().len(),
            wallet.available_utxos.get(0)
        );
        println!("Searching for utxo {:#?}", input.txid);

        let utxo = wallet
            .available_utxos
            .iter()
            .find(|u| u.txid == input.txid && u.index == input.index)
            .ok_or_else(|| {
                HTTPResponseError::InvalidBody(Some(String::from(
                    "UTXO not found in wallet",
                )))
            })?;

        // The claimed data must match the real UTXO
        if utxo.value != input.value || utxo.owner != input.owner {
            return Err(HTTPResponseError::InvalidBody(Some(String::from(
                "Input does not match the UTXO it references",
            ))));
        }
    }
    notify_new_transaction(&transaction, connected_nodes, source);
    transaction_pool.push(transaction);

    Ok(HTTPResponse::OK(Some(Content::JSON(json!({
        "msg": "The transaction was added to the pool.",
        "status_code": "200"
    })))))
}
