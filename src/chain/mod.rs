pub mod block;
pub mod ordered_vector;
pub mod transaction;
pub mod utils;
pub mod utxo;
pub mod wallet;
mod wallet_pk;

use block::Block;
use serde::{Deserialize, Serialize};

use crate::chain::{
    block::BlockHeader, utxo::TransactionOutput, wallet::Wallet,
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Chain {
    pub blocks: Vec<block::Block>,
    pub wallets: Vec<Wallet>,
}

impl Chain {
    pub fn new(
        first_receiver: Wallet,
        utxo_original: Vec<TransactionOutput>,
    ) -> Self {
        let mut chain = Self {
            blocks: Vec::new(),
            wallets: Vec::new(),
        };
        chain.create_genesis_block(first_receiver, utxo_original);
        chain
    }

    fn register_block_wallets(&mut self, block: &Block) {
        println!("Registering block");
        for tx in block.transactions.clone() {
            // <- exact accessor TBD
            // Outputs: this owner now has a new spendable UTXO.
            for output in &tx.outputs {
                println!("output");
                match self.wallets.iter_mut().find(|w| *w == &output.owner) {
                    Some(existing) => existing.add_utxos(vec![output.clone()]),
                    None => {
                        let mut w = output.owner.clone();
                        w.add_utxos(vec![output.clone()]);
                        println!("Registering wallet");
                        self.wallets.push(w);
                    }
                }
            }
            // Inputs: the referenced UTXO is now spent, so remove it from
            // the owner's available set.
            for input in &tx.inputs {
                if let Some(owner) =
                    self.wallets.iter_mut().find(|w| *w == &input.owner)
                {
                    owner.remove_utxo(input); // <- needs to exist, see below
                }
            }
        }
    }

    /// Adds block without checking anything
    pub fn add_block(&mut self, block: Block) {
        self.register_block_wallets(&block);
        self.blocks.push(block);
    }

    pub fn create_genesis_block(
        &mut self,
        first_receiver: Wallet,
        utxo_original: Vec<TransactionOutput>,
    ) -> Block {
        let genesis = Block::genesis_block(first_receiver, utxo_original);
        self.add_block(genesis.clone());
        genesis
    }

    pub fn get_last_hash(&self) -> String {
        self.blocks
            .last()
            .expect("Chain was created without genesis_block")
            .hash()
    }

    pub fn get_last_index(&self) -> u64 {
        self.blocks
            .last()
            .expect("Chain was created without genesis_block")
            .index()
    }

    pub fn find_block_from_header(
        &self,
        header: BlockHeader,
    ) -> Option<&Block> {
        self.blocks.iter().find(|b| b.to_header() == header)
    }
}

pub mod testing {
    use super::Chain;
    use crate::chain::block::Block;
    use crate::chain::utxo::{TransactionInput, TransactionOutput};
    use crate::chain::{transaction::Transaction, wallet::Wallet};

    pub fn test_chain() -> Chain {
        let wallet_1 = Wallet::new();
        let wallet_2 = Wallet::new();
        let wallet_3 = Wallet::new();
        let wallet_4 = Wallet::new();
        let wallet_5 = Wallet::new();

        // --- Block 1: wallet_1 splits 100000 evenly to itself and wallet_2 ---
        let utxos_1 = vec![TransactionInput::new(100000, wallet_1.0.clone())];
        let utxos_1_output = vec![
            TransactionOutput::new(50000, wallet_1.0.clone()),
            TransactionOutput::new(50000, wallet_2.0.clone()),
        ];
        let mut transaction_1 =
            Transaction::new(utxos_1, utxos_1_output).unwrap();

        wallet_1
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction_1)
            .unwrap();

        let mut chain = Chain::new(Wallet::null_wallet(), vec![]);
        let block_1 = Block::new(&mut chain, vec![transaction_1]);
        chain.add_block(block_1);

        // --- Block 2: wallet_1 sends 50000 to wallet_3,
        //              wallet_2 sends 50000 split to wallet_3 and wallet_4 ---
        let utxos_2_input =
            vec![TransactionInput::new(50000, wallet_1.0.clone())];
        let utxos_2_output =
            vec![TransactionOutput::new(50000, wallet_3.0.clone())];
        let mut transaction_2 =
            Transaction::new(utxos_2_input, utxos_2_output).unwrap();

        wallet_1
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction_2)
            .unwrap();

        let utxos_3 = vec![TransactionInput::new(50000, wallet_2.0.clone())];
        let utxos_3_output = vec![
            TransactionOutput::new(25000, wallet_3.0.clone()),
            TransactionOutput::new(25000, wallet_4.0.clone()),
        ];
        let mut transaction_3 =
            Transaction::new(utxos_3, utxos_3_output).unwrap();
        wallet_2
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction_3)
            .unwrap();

        let block_2 =
            Block::new(&mut chain, vec![transaction_2, transaction_3]);
        chain.add_block(block_2);

        // --- Block 3: wallet_3 consolidates its 75000 and sends it all to wallet_5,
        //              wallet_4 sends 25000 split to wallet_1 and wallet_5 ---
        let utxos_4 = vec![
            TransactionInput::new(50000, wallet_3.0.clone()),
            TransactionInput::new(25000, wallet_3.0.clone()),
        ];
        let utxos_4_output =
            vec![TransactionOutput::new(75000, wallet_5.0.clone())];
        let mut transaction_4 =
            Transaction::new(utxos_4, utxos_4_output).unwrap();
        wallet_3
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction_4)
            .unwrap();

        let utxos_5 = vec![TransactionInput::new(25000, wallet_4.0.clone())];
        let utxos_5_output = vec![
            TransactionOutput::new(10000, wallet_1.0.clone()),
            TransactionOutput::new(15000, wallet_5.0.clone()),
        ];
        let mut transaction_5 =
            Transaction::new(utxos_5, utxos_5_output).unwrap();

        wallet_4
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction_5)
            .unwrap();

        let block_3 =
            Block::new(&mut chain, vec![transaction_4, transaction_5]);
        chain.add_block(block_3);

        // --- Block 4: wallet_5 distributes its 90000 back to everyone ---
        let utxos_6 = vec![
            TransactionInput::new(75000, wallet_5.0.clone()),
            TransactionInput::new(15000, wallet_5.0.clone()),
        ];
        let utxos_6_output = vec![
            TransactionOutput::new(20000, wallet_1.0.clone()),
            TransactionOutput::new(20000, wallet_2.0.clone()),
            TransactionOutput::new(20000, wallet_3.0.clone()),
            TransactionOutput::new(20000, wallet_4.0.clone()),
            TransactionOutput::new(10000, wallet_5.0.clone()),
        ];
        let mut transaction_6 =
            Transaction::new(utxos_6, utxos_6_output).unwrap();
        wallet_5
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction_6)
            .unwrap();

        let block_4 = Block::new(&mut chain, vec![transaction_6]);
        chain.add_block(block_4);

        chain
    }
}
