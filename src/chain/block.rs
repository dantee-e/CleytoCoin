use core::hash;
use std::ptr::hash;
use serde::{Serialize, Deserialize};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use super::transaction::Transaction;
use super::Chain;


#[derive(Clone)]
pub struct Block {
    previous_hash: String,
    transactions: Vec<Transaction>,
    index: i64,
    timestamp: DateTime<Utc>,
    hash: String,
}

impl Block {
    pub fn get_hash(&self) -> String{
        self.hash.clone()
    }

    pub fn get_index(&self) -> i64 {
        self.index.clone()
    }

    pub fn calculate_hash(&self) -> String {
        let transactions_string = self.transactions
            .iter()
            .map(|t| t.to_string()) // Calls the `to_string()` method of `Transaction`
            .collect::<Vec<String>>() // Collects into a Vec<String>
            .join("::END_OF_TRANSACTION::BEGIN_OF_TRANSACTION::"); // Joins all elements with "; " as separator
        
        let serialized = serde_json::to_string
        (&(
            "BEGIN::BEGIN_PREVIOUS_HASH::",
            &self.previous_hash,
            "::END_PREVIOUS_HASH::BEGIN_TRANSACTIONS::",
            "BEGIN_OF_TRANSACTION::",
            transactions_string,
            "::END_OF_TRANSACTION",
            "::END_TRANSACTIONS::BEGIN_INDEX::",
            &self.index,
            "::END_INDEX::BEGIN_TIMESTAMP::",
            &self.timestamp.to_string(),
            "::END_TIMESTAMP::END::",
        )).expect("Coudn't serialize the block to create the hash");

        println!("behold the serialized block:\n{serialized}");

        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let result = hasher.finalize();

        hex::encode(result) // Converts bytes to a hex string
    }

    pub fn new(chain: &mut Chain, transactions: Vec<Transaction>) -> Block {
        let previous_hash = chain.get_last_hash();
        let index = chain.get_last_index() + 1;
        let timestamp = Utc::now();

        
        let mut block = Self {
            previous_hash,
            transactions,
            index,
            timestamp,
            hash: String::new() // temporary so that we can calculate hash
        };

        block.hash = block.calculate_hash();

        block

    }

    pub fn genesis_block() -> Self{
        Self {
            previous_hash: String::from("Foguete nao da re"),
            transactions: Vec::new(),
            index: 0,
            timestamp: Utc::now(),
            hash: String::from("The Times 03/Jan/2009 Chancellor on brink of second bailout for banks")
        }
    }
}


