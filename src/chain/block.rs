use chrono::{DateTime, Utc};

use super::utils::HashedData;
use super::transaction::Transaction;

pub struct Block {
    previous_hash: HashedData,
    transactions: Vec<Transaction>,
    index: i64,
    timestamp: DateTime<Utc>,
    hash: HashedData,
}


