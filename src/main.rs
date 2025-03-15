mod chain;
use chain::{block, transaction::{Transaction, TransactionInfo}};
use chrono::Utc;
use chain::wallet::Wallet;

fn main() {
    let (wallet1, mut wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    

    let transaction_info = TransactionInfo::new(10.5, Utc::now());

    let signature = match wallet1_pk.sign_transaction(&transaction_info) {
        Ok(value) => value,
        Err(e) => panic!("Error creating signed message: {e}"),
    };

    let new_transaction = Transaction::new(wallet1, wallet2, transaction_info, signature);


    let mut chain = chain::Chain::new();

    let block = block::Block::new(&mut chain, vec![new_transaction]);

    chain.add_block(block);

    
}
