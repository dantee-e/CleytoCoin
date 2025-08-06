use chrono::Utc;
use cleyto_coin::chain::transaction::{Transaction, TransactionInfo};
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn create_transaction() {
    let (wallet_sender, mut walletpk_sender) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();
    let transactioninfo: TransactionInfo = TransactionInfo::new(12345, Utc::now());

    let signature = match walletpk_sender.sign_transaction(&transactioninfo) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!(
        "Transaction signature (signed using the wallet_pk):\n{:?}",
        signature
    );

    // this will also be verified by the Transaction::new();
    if wallet_sender
        .verify_transaction_info(&transactioninfo, &signature)
        .unwrap()
    {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }

    let transaction: Transaction =
        Transaction::new(wallet_sender, wallet_receiver, transactioninfo, signature).unwrap();

    println!("transaction.to_string(): {}", transaction);
}

#[test] //mark a function as a test.
fn test_transaction_info_creation() {
    let transaction: TransactionInfo = TransactionInfo::new(123452, Utc::now());
    println!("transaction info:\n{}", transaction);
    println!("{:?}", transaction);
}

#[test]
fn sign_and_verify_transaction_info() {
    let (wallet, mut wallet_pk) = Wallet::new();
    let transactioninfo: TransactionInfo = TransactionInfo::new(1234532, Utc::now());

    let signature = match wallet_pk.sign_transaction(&transactioninfo) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!(
        "Transaction signature (signed using the wallet_pk):\n{:?}",
        signature
    );

    if wallet
        .verify_transaction_info(&transactioninfo, &signature)
        .unwrap()
    {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }
}

#[test]
fn serialize_and_deserialize_transaction() {
    let (wallet, mut wallet_pk) = Wallet::new();
    let (mallet, _) = Wallet::new();
    let transactioninfo: TransactionInfo = TransactionInfo::new(1234552, Utc::now());

    let signature = match wallet_pk.sign_transaction(&transactioninfo) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };

    let transaction = Transaction::new(wallet, mallet, transactioninfo, signature).unwrap();

    let serialized_transaction = transaction.serialize();
    println!("serialized_transaction: \n{serialized_transaction}");

    match Transaction::deserialize(serialized_transaction) {
        Ok(value) => {
            println!("Success deserializing transaction");
            value
        }
        Err(_) => panic!("Error deserializing transaction"),
    };
}
