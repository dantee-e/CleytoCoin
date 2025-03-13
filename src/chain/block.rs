use super::utils::{HashedData, Date};
use super::transaction::Transaction;

pub struct Block {
    previous_hash: HashedData,
    transactions: Vec<Transaction>,
    index: i64,
    timestamp: Date,
    hash: HashedData,
}


