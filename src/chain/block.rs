use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use super::transaction::Transaction;
use super::Chain;
use super::utils::PROOF_OF_WORK_DIFFICULTY;


#[derive(Clone)]
pub struct Block {
    previous_hash: String,
    transactions: Vec<Transaction>,
    index: i64,
    timestamp: DateTime<Utc>,
    hash: String,
    nonce: u64
}

impl Block {
    pub fn get_hash(&self) -> String {
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
            "::END_TIMESTAMP::BEGIN_NONCE::",
            &self.nonce,
            "::END_NONCE::END",
        )).expect("Coudn't serialize the block to create the hash");

        println!("behold the serialized block:\n{serialized}");

        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let result = hasher.finalize();
        let encoded_result = hex::encode(result);
        println!("Hash = {encoded_result}");
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
            hash: String::new(),
            nonce: 0 // temporary so that we can calculate hash
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
            hash: String::from("The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"),
            nonce: 0
        }
    }

    pub fn mine_block(mut self) -> Self {
        let prefix = "0".repeat(PROOF_OF_WORK_DIFFICULTY.into());

        while !self.hash.starts_with(&prefix) {
            self.nonce+=1;
            self.hash = self.calculate_hash();
        }

        self
    }
}


