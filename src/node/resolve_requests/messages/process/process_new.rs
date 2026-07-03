use std::collections::HashSet;

use crate::{
    chain::{block::Block, transaction::Transaction, Chain},
    node::{
        data::check_block_is_registered_by_hash,
        resolve_requests::{
            helpers::HTTPResult,
            messages::send::{notify_new_block, notify_new_transaction},
            methods::HTTPResponse,
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
    if transaction_pool.contains(&transaction) {
        return Ok(HTTPResponse::OK(None));
    }

    notify_new_transaction(&transaction, connected_nodes, source);
    transaction_pool.push(transaction);
    Ok(HTTPResponse::OK(None))
}
