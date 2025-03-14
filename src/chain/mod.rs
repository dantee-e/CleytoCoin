pub mod utils;
pub mod transaction;
pub mod block;
pub mod wallet;
use chrono::{DateTime, Utc};
use transaction::Transaction as Transaction;

pub mod testes;

struct Chain {
    blocks: Vec<block::Block>
}

impl Chain {
    fn new_block(previous_hash: utils::HashedData, transactions: Vec<Transaction>, timestamp: DateTime<Utc>){
        println!("{}", previous_hash.hash_as_string());
    }
}