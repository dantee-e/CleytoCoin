mod chain;
use chain::transaction::{Transaction, TransactionInfo};
use chrono::{DateTime, Utc};
use chain::wallet::Wallet;

fn main() {
    let (wallet1, mut wallet1_pk) = Wallet::new();
    let (wallet2, mut wallet2_pk) = Wallet::new();

    let transaction_info = TransactionInfo::new(10.5, Utc::now());

    let signature = match wallet1_pk.sign_transaction(&transaction_info) {
        Ok(value) => value,
        Err(e) => panic!("Error creating signed message: {e}"),
    };

    println!("Signature: {signature}");

    /* println!("This is the signature: {}", signed_message); */

    /* let verify_message = signed_message */

    let verified = wallet1.verify_signature(transaction_info.to_string(), signature);

    match verified {
        true => println!("Sucesso nan verificacao"),
        false => println!("Deu ruim na verificacao")
    }

    /* let mut transactions: Vec<Transaction> = Vec::new();

    let new_transaction = Transaction::new(wallet1, wallet2, transaction_info, signature, 10.0);

    println!("Created new transaction, adding to chain...");
    transactions.push(new_transaction); */
}
