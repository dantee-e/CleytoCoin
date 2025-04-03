pub mod utils;
pub mod transaction;
pub mod block;
pub mod wallet;
use block::Block;
use chrono::{DateTime, Utc};
use sha2::Sha256;
use transaction::Transaction as Transaction;

pub struct Chain {
    blocks: Vec<block::Block>
}

impl Chain {

    pub fn new() -> Self {
        Self{
            blocks: Vec::new()
        }
    }

    pub fn add_block(&mut self, block: Block){
        self.blocks.push(block);
    }

    pub fn create_genesis_block(&mut self) -> Block {
        let genesis = Block::genesis_block();
        self.add_block(genesis.clone());
        genesis
    }

    fn get_last_hash(&mut self) -> String {
        match self.blocks.last() {
            Some(block) => block.get_hash(),
            None => {
                let genesis_block = self.create_genesis_block();
                genesis_block.get_hash()
            },
        }
    }

    fn get_last_index(&mut self) -> u64 {
        match self.blocks.last() {
            Some(block) => block.get_index(),
            None =>{
                let genesis_block = self.create_genesis_block();
                genesis_block.get_index()
            },
        }
    }
}
