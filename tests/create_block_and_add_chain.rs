use CleytoCoin::chain::{Chain, wallet::Wallet, block::Block, transaction::{Transaction, TransactionInfo}};
use chrono::Utc;

#[test]
fn create_block_and_add_chain() {
    let (wallet1, mut wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    let transaction_info = TransactionInfo::new(10.5, Utc::now());

    let signature = match wallet1_pk.sign_transaction(&transaction_info) {
        Ok(value) => value,
        Err(e) => panic!("Error creating signed message: {e}"),
    };

    let new_transaction = Transaction::new(wallet1, wallet2, transaction_info, signature);

<<<<<<< HEAD:src/main.rs
    let mut chain = chain::Chain::new();

    let block = block::Block::new(&mut chain, vec![new_transaction]);
=======
    let mut chain = Chain::new();

    let block = Block::new(&mut chain, vec![new_transaction]);
>>>>>>> 68fcfb662ba503f732442252f2097e1882a40c57:tests/create_block_and_add_chain.rs

    chain.add_block(block);
}
