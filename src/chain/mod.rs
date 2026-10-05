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
    block::BlockHeader,
    ordered_vector::OrderedVec,
    transaction::Transaction,
    utxo::{PublicKey, TransactionOutput, UtxoSet, UTXO},
    wallet::Wallet,
};
use crate::error_handling::CleytoResult;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Chain {
    pub blocks: Vec<block::Block>,
    /// Every unspent output, updated as blocks are added
    pub utxos: UtxoSet,
}

impl Chain {
    pub fn new(genesis_outputs: Vec<TransactionOutput>) -> Self {
        let mut chain = Self {
            blocks: Vec::new(),
            utxos: UtxoSet::default(),
        };
        chain.create_genesis_block(genesis_outputs);
        chain
    }

    /// Adds block without checking anything
    pub fn add_block(&mut self, block: Block) {
        for transaction in &block.transactions {
            self.utxos.apply(transaction);
        }
        self.blocks.push(block);
    }

    pub fn create_genesis_block(
        &mut self,
        genesis_outputs: Vec<TransactionOutput>,
    ) -> Block {
        let genesis = Block::genesis_block(genesis_outputs);
        self.add_block(genesis.clone());
        genesis
    }

    /// Checks the transaction against the current UTXO set. Returns its fee.
    pub fn validate_transaction(
        &self,
        transaction: &Transaction,
    ) -> CleytoResult<u64> {
        transaction.verify(&self.utxos)
    }

    pub fn utxos_owned_by(&self, owner: &PublicKey) -> Vec<UTXO> {
        self.utxos.owned_by(owner)
    }

    /// Replaces the wallet's available UTXOs with the ones it owns on this chain
    pub fn load_wallet_utxos(&self, wallet: &mut Wallet) {
        wallet.available_utxos =
            OrderedVec::from(self.utxos_owned_by(&wallet.to_public_key()));
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
    use crate::chain::{
        transaction::Transaction,
        wallet::{Wallet, WalletPK},
    };

    /// Builds a transaction spending every coin `owner` has on `chain` into `outputs`, signs it
    /// and checks it is valid against the chain.
    pub fn spend_all(
        chain: &Chain,
        owner: &(Wallet, WalletPK),
        outputs: Vec<TransactionOutput>,
    ) -> Transaction {
        let coins = chain.utxos_owned_by(&owner.0.to_public_key());
        let mut transaction =
            Transaction::new(TransactionInput::from_utxos(&coins), outputs)
                .unwrap();

        owner
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
            .unwrap();

        chain.validate_transaction(&transaction).unwrap();
        transaction
    }

    pub fn test_chain() -> Chain {
        let wallet_1 = Wallet::new();
        let wallet_2 = Wallet::new();
        let wallet_3 = Wallet::new();
        let wallet_4 = Wallet::new();
        let wallet_5 = Wallet::new();

        // --- Genesis: wallet_1 receives 100000 ---
        let mut chain =
            Chain::new(vec![TransactionOutput::new(100000, &wallet_1.0)]);

        // --- Block 1: wallet_1 splits 100000 evenly to itself and wallet_2 ---
        let transaction_1 = spend_all(
            &chain,
            &wallet_1,
            vec![
                TransactionOutput::new(50000, &wallet_1.0),
                TransactionOutput::new(50000, &wallet_2.0),
            ],
        );
        let block_1 = Block::new(&mut chain, vec![transaction_1]);
        chain.add_block(block_1);

        // --- Block 2: wallet_1 sends 50000 to wallet_3,
        //              wallet_2 sends 50000 split to wallet_3 and wallet_4 ---
        let transaction_2 = spend_all(
            &chain,
            &wallet_1,
            vec![TransactionOutput::new(50000, &wallet_3.0)],
        );
        let transaction_3 = spend_all(
            &chain,
            &wallet_2,
            vec![
                TransactionOutput::new(25000, &wallet_3.0),
                TransactionOutput::new(25000, &wallet_4.0),
            ],
        );
        let block_2 =
            Block::new(&mut chain, vec![transaction_2, transaction_3]);
        chain.add_block(block_2);

        // --- Block 3: wallet_3 consolidates its 75000 and sends it all to wallet_5,
        //              wallet_4 sends 25000 split to wallet_1 and wallet_5 ---
        let transaction_4 = spend_all(
            &chain,
            &wallet_3,
            vec![TransactionOutput::new(75000, &wallet_5.0)],
        );
        let transaction_5 = spend_all(
            &chain,
            &wallet_4,
            vec![
                TransactionOutput::new(10000, &wallet_1.0),
                TransactionOutput::new(15000, &wallet_5.0),
            ],
        );
        let block_3 =
            Block::new(&mut chain, vec![transaction_4, transaction_5]);
        chain.add_block(block_3);

        // --- Block 4: wallet_5 distributes its 90000 back to everyone ---
        let transaction_6 = spend_all(
            &chain,
            &wallet_5,
            vec![
                TransactionOutput::new(20000, &wallet_1.0),
                TransactionOutput::new(20000, &wallet_2.0),
                TransactionOutput::new(20000, &wallet_3.0),
                TransactionOutput::new(20000, &wallet_4.0),
                TransactionOutput::new(10000, &wallet_5.0),
            ],
        );
        let block_4 = Block::new(&mut chain, vec![transaction_6]);
        chain.add_block(block_4);

        chain
    }
}
