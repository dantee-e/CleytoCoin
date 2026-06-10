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
    pub blocks: Vec<block::Block>,
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

pub mod testing {
    use super::Chain;
    use crate::chain::block::Block;
    use crate::chain::{
        transaction::{Transaction, TransactionInfo},
        utxo::UTXO,
        wallet::Wallet,
    };

    pub fn test_chain() -> Chain {
        let wallet_1 = Wallet::new();
        let wallet_2 = Wallet::new();
        let wallet_3 = Wallet::new();
        let wallet_4 = Wallet::new();
        let wallet_5 = Wallet::new();

        // --- Block 1: wallet_1 splits 100000 evenly to itself and wallet_2 ---
        let utxos_1 = vec![UTXO::new(100000, wallet_1.0.clone())];
        let utxos_1_output = vec![
            UTXO::new(50000, wallet_1.0.clone()),
            UTXO::new(50000, wallet_2.0.clone()),
        ];
        let transaction_info_1 = TransactionInfo::new(utxos_1, utxos_1_output);
        let signature_1 = wallet_1.1.sign_transaction(&transaction_info_1).unwrap();
        let transaction_1 = Transaction::new(
            wallet_1.0.clone(),
            wallet_2.0.clone(),
            transaction_info_1,
            signature_1,
        )
        .unwrap();

        let mut chain = Chain::new();
        let block_1 = Block::new(&mut chain, vec![transaction_1]);
        chain.add_block(block_1);

        // --- Block 2: wallet_1 sends 50000 to wallet_3,
        //              wallet_2 sends 50000 split to wallet_3 and wallet_4 ---
        let utxos_2 = vec![UTXO::new(50000, wallet_1.0.clone())];
        let utxos_2_output = vec![UTXO::new(50000, wallet_3.0.clone())];
        let transaction_info_2 = TransactionInfo::new(utxos_2, utxos_2_output);
        let signature_2 = wallet_1.1.sign_transaction(&transaction_info_2).unwrap();
        let transaction_2 = Transaction::new(
            wallet_1.0.clone(),
            wallet_3.0.clone(),
            transaction_info_2,
            signature_2,
        )
        .unwrap();

        let utxos_3 = vec![UTXO::new(50000, wallet_2.0.clone())];
        let utxos_3_output = vec![
            UTXO::new(25000, wallet_3.0.clone()),
            UTXO::new(25000, wallet_4.0.clone()),
        ];
        let transaction_info_3 = TransactionInfo::new(utxos_3, utxos_3_output);
        let signature_3 = wallet_2.1.sign_transaction(&transaction_info_3).unwrap();
        let transaction_3 = Transaction::new(
            wallet_2.0.clone(),
            wallet_4.0.clone(),
            transaction_info_3,
            signature_3,
        )
        .unwrap();

        let block_2 = Block::new(&mut chain, vec![transaction_2, transaction_3]);
        chain.add_block(block_2);

        // --- Block 3: wallet_3 consolidates its 75000 and sends it all to wallet_5,
        //              wallet_4 sends 25000 split to wallet_1 and wallet_5 ---
        let utxos_4 = vec![
            UTXO::new(50000, wallet_3.0.clone()),
            UTXO::new(25000, wallet_3.0.clone()),
        ];
        let utxos_4_output = vec![UTXO::new(75000, wallet_5.0.clone())];
        let transaction_info_4 = TransactionInfo::new(utxos_4, utxos_4_output);
        let signature_4 = wallet_3.1.sign_transaction(&transaction_info_4).unwrap();
        let transaction_4 = Transaction::new(
            wallet_3.0.clone(),
            wallet_5.0.clone(),
            transaction_info_4,
            signature_4,
        )
        .unwrap();

        let utxos_5 = vec![UTXO::new(25000, wallet_4.0.clone())];
        let utxos_5_output = vec![
            UTXO::new(10000, wallet_1.0.clone()),
            UTXO::new(15000, wallet_5.0.clone()),
        ];
        let transaction_info_5 = TransactionInfo::new(utxos_5, utxos_5_output);
        let signature_5 = wallet_4.1.sign_transaction(&transaction_info_5).unwrap();
        let transaction_5 = Transaction::new(
            wallet_4.0.clone(),
            wallet_5.0.clone(),
            transaction_info_5,
            signature_5,
        )
        .unwrap();

        let block_3 = Block::new(&mut chain, vec![transaction_4, transaction_5]);
        chain.add_block(block_3);

        // --- Block 4: wallet_5 distributes its 90000 back to everyone ---
        let utxos_6 = vec![
            UTXO::new(75000, wallet_5.0.clone()),
            UTXO::new(15000, wallet_5.0.clone()),
        ];
        let utxos_6_output = vec![
            UTXO::new(20000, wallet_1.0.clone()),
            UTXO::new(20000, wallet_2.0.clone()),
            UTXO::new(20000, wallet_3.0.clone()),
            UTXO::new(20000, wallet_4.0.clone()),
            UTXO::new(10000, wallet_5.0.clone()),
        ];
        let transaction_info_6 = TransactionInfo::new(utxos_6, utxos_6_output);
        let signature_6 = wallet_5.1.sign_transaction(&transaction_info_6).unwrap();
        let transaction_6 = Transaction::new(
            wallet_5.0.clone(),
            wallet_1.0.clone(),
            transaction_info_6,
            signature_6,
        )
        .unwrap();

        let block_4 = Block::new(&mut chain, vec![transaction_6]);
        chain.add_block(block_4);

        chain
    }
}
