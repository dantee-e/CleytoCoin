use chrono::Utc;
use cleyto_coin::chain::{
    block::Block,
    transaction::{Transaction, TransactionInfo},
    wallet::Wallet,
    Chain,
};

#[test]
fn create_block_and_add_chain() {
    let (wallet1, mut wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    let transaction_info = TransactionInfo::new(10.5, Utc::now());

    let signature = match wallet1_pk.sign_transaction(&transaction_info) {
        Ok(value) => value,
        Err(e) => panic!("Error creating signed message: {e}"),
    };

    let new_transaction = Transaction::new(wallet1, wallet2, transaction_info, signature).unwrap();

    let mut chain = Chain::new();

    let block = Block::new(&mut chain, vec![new_transaction]);

    chain.add_block(block);
}
