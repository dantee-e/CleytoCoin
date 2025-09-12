pub mod block;
pub mod ordered_vector;
pub mod transaction;
pub mod utils;
pub mod utxo;
pub mod wallet;
mod wallet_pk;
use block::Block;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Chain {
    blocks: Vec<block::Block>,
}

impl Chain {
    pub fn new() -> Self {
        let mut chain = Self { blocks: Vec::new() };
        chain.create_genesis_block();
        chain
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn create_genesis_block(&mut self) -> Block {
        let genesis = Block::genesis_block();
        self.add_block(genesis.clone());
        genesis
    }

    pub fn get_last_hash(&self) -> String {
        self.blocks
            .last()
            .expect("Chain was created without genesis_block")
            .get_hash()
    }

    pub fn get_last_index(&self) -> u64 {
        self.blocks
            .last()
            .expect("Chain was created without genesis_block")
            .get_index()
    }
}
