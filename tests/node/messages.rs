use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use cleyto_coin::chain::block::Block;
use cleyto_coin::chain::testing::spend_all;
use cleyto_coin::chain::utxo::{TransactionOutput, UTXO};
use cleyto_coin::chain::wallet::Wallet;
use cleyto_coin::chain::Chain;
use cleyto_coin::node::{Message, NodeState};

use crate::mock_stream::request;

fn state_with(chain: Chain) -> Arc<Mutex<NodeState>> {
    Arc::new(Mutex::new(NodeState {
        status: true,
        chain,
        transactions_pool: Vec::new(),
        connected_nodes: HashSet::new(),
    }))
}

/// A node whose genesis gives 1000 to `sender`, and the next block for that
/// node: `sender` pays 600 to `receiver` and keeps 400 as change
fn node_and_next_block() -> (Arc<Mutex<NodeState>>, Block, Wallet, Wallet) {
    let sender = Wallet::new();
    let (receiver, _) = Wallet::new();
    let chain = Chain::new(vec![TransactionOutput::new(1000, &sender.0)]);

    let transaction = spend_all(
        &chain,
        &sender,
        vec![
            TransactionOutput::new(600, &receiver),
            TransactionOutput::new(400, &sender.0),
        ],
    );

    // Built on a copy, so the node's chain doesn't have the block yet
    let mut builder = chain.clone();
    let block = Block::new(&mut builder, vec![transaction]);

    (state_with(chain), block, sender.0, receiver)
}

fn balance(state: &Arc<Mutex<NodeState>>, wallet: &Wallet) -> u64 {
    UTXO::sum(
        &state
            .lock()
            .unwrap()
            .chain
            .utxos_owned_by(&wallet.to_public_key()),
    )
}

#[tokio::test]
async fn send_new_block() {
    let (state, block, sender, receiver) = node_and_next_block();
    let body = serde_json::to_string(&Message::Block(block.clone())).unwrap();

    let result =
        request("POST", "/messages", Some(&body), state.clone()).await;
    assert!(result.is_ok(), "expected Ok, got {:?}", result);

    {
        let state = state.lock().unwrap();
        assert_eq!(state.chain.blocks.len(), 2);
        assert_eq!(state.chain.get_last_hash(), block.hash());
    }

    // The block's transaction was applied to the UTXO set
    assert_eq!(balance(&state, &receiver), 600);
    assert_eq!(balance(&state, &sender), 400);
}

#[tokio::test]
async fn send_same_block_twice() {
    let (state, block, sender, receiver) = node_and_next_block();
    let body = serde_json::to_string(&Message::Block(block)).unwrap();

    for _ in 0..2 {
        let result =
            request("POST", "/messages", Some(&body), state.clone()).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    // Added, and its transaction applied, only once
    assert_eq!(state.lock().unwrap().chain.blocks.len(), 2);
    assert_eq!(balance(&state, &receiver), 600);
    assert_eq!(balance(&state, &sender), 400);
}
