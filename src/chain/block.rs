use chrono::{DateTime, Utc};
use openssl::hash::{Hasher, MessageDigest};
use serde::{Deserialize, Serialize};

use super::transaction::Transaction;
use super::utils::PROOF_OF_WORK_DIFFICULTY;
use super::Chain;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    version: u8,
    previous_hash: String,
    transactions: Vec<Transaction>,
    index: u64,
    timestamp: DateTime<Utc>,
    hash: String,
    merkle_root: [u8; 32],
    nonce: u64,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    version: u8,
    previous_hash: String,
    timestamp: DateTime<Utc>,
    merkle_root: [u8; 32],
    nonce: u64,
}

impl Block {
    pub fn hash(&self) -> String {
        self.hash.clone()
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn to_header(&self) -> BlockHeader {
        BlockHeader {
            version: self.version,
            previous_hash: self.previous_hash.clone(),
            timestamp: self.timestamp,
            merkle_root: self.merkle_root,
            nonce: self.nonce,
        }
    }

    pub fn previous_hash(&self) -> String {
        self.previous_hash.clone()
    }

    /// This merkle tree does not work the same way as the bitcoin core one. If the number of
    /// leaves is not a power of two, it copies the last transaction's hash until we have enough
    /// leaves for a binary tree, and then it collapses the tree into the root, which is then
    /// returned.
    fn calculate_merkle_root(transactions: &[Transaction]) -> [u8; 32] {
        // This gets the closest bigger power of 2
        let log_2 = f32::log2(transactions.len() as f32);
        let mut closest_log_2: u32 = log_2 as u32;
        if log_2 > closest_log_2 as f32 {
            closest_log_2 += 1;
        }
        let mut closest_pow_2 = 2usize.pow(closest_log_2);

        // Runs first time to populate the hash_vec with the hash of all transactions
        let mut hash_vec = (0..closest_pow_2)
            .map(|i| {
                let transaction_hash = {
                    let transaction = match transactions.get(i) {
                        Some(transaction) => transaction,
                        None => transactions.last().unwrap(),
                    };
                    transaction.txid
                };
                transaction_hash
            })
            .collect::<Vec<[u8; 32]>>();

        // Fuck recursion
        let result: [u8; 32];
        loop {
            if closest_pow_2 == 1 {
                result = hash_vec[0];
                break;
            }

            for i in (0..closest_pow_2).step_by(2) {
                // gets either the hashes in the right index or the last hash on the hash_vec
                let (hash_1, hash_2) = {
                    (
                        match hash_vec.get(i) {
                            Some(hash) => hash,
                            None => hash_vec.last().unwrap(),
                        },
                        match hash_vec.get(i + 1) {
                            Some(hash) => hash,
                            None => hash_vec.last().unwrap(),
                        },
                    )
                };
                let mut hasher = Hasher::new(MessageDigest::sha256()).unwrap();
                hasher.update(hash_1).unwrap();
                hasher.update(hash_2).unwrap();
                hash_vec[i / 2] = hasher.finish().unwrap().as_ref().try_into().unwrap();
            }
            closest_pow_2 /= 2;
        }
        result
    }

    pub fn calculate_hash(&self) -> String {
        //This here should be replaced by the calculation of the merkle tree
        // let transactions_string = self
        //     .transactions
        //     .iter()
        //     .map(|t| t.to_string()) // Calls the `to_string()` method of `Transaction`
        //     .collect::<Vec<String>>() // Collects into a Vec<String>
        //     .join("::END_OF_TRANSACTION::BEGIN_OF_TRANSACTION::"); // Joins all elements with "; " as separator

        let merkle_root = Self::calculate_merkle_root(&self.transactions);

        let serialized = serde_json::to_string(&(
            "BEGIN::BEGIN_PREVIOUS_HASH::",
            &self.previous_hash,
            "::END_PREVIOUS_HASH::BEGIN_TRANSACTIONS::",
            "BEGIN_OF_TRANSACTION::",
            merkle_root,
            "::END_OF_TRANSACTION",
            "::END_TRANSACTIONS::BEGIN_INDEX::",
            &self.index,
            "::END_INDEX::BEGIN_TIMESTAMP::",
            &self.timestamp.to_string(),
            "::END_TIMESTAMP::BEGIN_NONCE::",
            &self.nonce,
            "::END_NONCE::END",
        ))
        .expect("Coudn't serialize the block to create the hash");

        // println!("behold the serialized block:\n{serialized}");

        let mut hasher = Hasher::new(MessageDigest::sha256()).unwrap();
        hasher.update(serialized.as_bytes()).unwrap();
        let result = hasher.finish().unwrap();

        hex::encode(result) // Converts bytes to a hex string
    }

    pub fn new(chain: &mut Chain, transactions: Vec<Transaction>) -> Block {
        let previous_hash = chain.get_last_hash();
        let index = chain.get_last_index() + 1;
        let timestamp = Utc::now();
        let merkle_root = Self::calculate_merkle_root(&transactions);

        let mut block = Self {
            version: 1,
            previous_hash,
            transactions,
            index,
            timestamp,
            hash: String::new(),
            merkle_root,
            nonce: 0, // temporary so that we can calculate hash
        };

        block.hash = block.calculate_hash();

        block
    }

    pub fn genesis_block() -> Self {
        let merkle_root = Block::calculate_merkle_root(&[]);
        Self {
            version: 1,
            previous_hash: String::from("Foguete nao da re"),
            transactions: Vec::new(),
            index: 1,
            timestamp: Utc::now(),
            hash: String::from(
                "The_Times_03_Jan_2009_Chancellor_on_brink_of_second_bailout_for_banks",
            ),
            merkle_root,
            nonce: 0,
        }
    }

    pub fn mine_block(mut self) -> Self {
        let prefix = "0".repeat(PROOF_OF_WORK_DIFFICULTY.into());

        while !self.hash.starts_with(&prefix) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        self
    }

    pub fn test_block(chain: &Chain) -> Self {
        let previous_hash = chain.get_last_hash();
        let index = chain.get_last_index() + 1;
        let timestamp = Utc::now();

        let transactions = vec![
            Transaction::default(),
            Transaction::default(),
            Transaction::default(),
        ];

        let merkle_root = Block::calculate_merkle_root(&transactions);

        let mut block = Self {
            version: 1,
            previous_hash,
            transactions,
            index,
            timestamp,
            hash: String::new(),
            merkle_root,
            nonce: 0, // temporary so that we can calculate hash
        };

        block.hash = block.calculate_hash();

        block
    }
}
