pub mod utils;
pub mod transaction;
pub mod block;
pub mod wallet;
use transaction::Transaction as Transaction;

struct Chain {
    blocks: Vec<block::Block>
}

impl Chain {
    fn new_block(previous_hash: utils::HashedData, transactions: Vec<Transaction>, timestamp: utils::Date){
        println!("{}", previous_hash.hash_as_string());
        
    }
}