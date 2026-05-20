use cleyto_coin::chain::transaction::{Transaction, TransactionInfo};
use cleyto_coin::chain::utxo::UTXO;
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn create_transaction() {
    let (wallet_sender, walletpk_sender) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();

    let input_utxos = vec![
        UTXO::new(1000, wallet_sender.clone()),
        UTXO::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        UTXO::new(2500, wallet_receiver.clone()),
        UTXO::new(500, wallet_receiver.clone()),
    ];
    let transaction_info: TransactionInfo = TransactionInfo::new(input_utxos, output_utxos);

    let signature = match walletpk_sender.sign_transaction(&transaction_info) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!(
        "Transaction signature (signed using the wallet_pk):\n{:?}",
        signature
    );

    // this will also be verified by the Transaction::new();
    if wallet_sender
        .verify_transaction_info(&transaction_info, &signature)
        .unwrap()
    {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }

    let transaction: Transaction =
        Transaction::new(wallet_sender, wallet_receiver, transaction_info, signature).unwrap();

    println!("transaction.to_string(): {}", transaction);
}

#[test]
fn test_transaction_info_creation() {
    let (wallet_sender, _) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();

    let input_utxos = vec![
        UTXO::new(1000, wallet_sender.clone()),
        UTXO::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        UTXO::new(2500, wallet_receiver.clone()),
        UTXO::new(500, wallet_receiver.clone()),
    ];
    let transaction_info: TransactionInfo = TransactionInfo::new(input_utxos, output_utxos);
    println!("transaction info:\n{}", transaction_info);
    println!("{:?}", transaction_info);
}

#[test]
fn sign_and_verify_transaction_info() {
    let (wallet_sender, wallet_pk) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();

    let input_utxos = vec![
        UTXO::new(1000, wallet_sender.clone()),
        UTXO::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        UTXO::new(2500, wallet_receiver.clone()),
        UTXO::new(500, wallet_receiver.clone()),
    ];
    let transaction_info: TransactionInfo = TransactionInfo::new(input_utxos, output_utxos);

    let signature = match wallet_pk.sign_transaction(&transaction_info) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!(
        "Transaction signature (signed using the wallet_pk):\n{:?}",
        signature
    );

    if wallet_sender
        .verify_transaction_info(&transaction_info, &signature)
        .unwrap()
    {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }
}

#[test]
fn serialize_and_deserialize_transaction() {
    let (wallet, wallet_pk) = Wallet::new();
    let (mallet, _) = Wallet::new();

    let input_utxos = vec![
        UTXO::new(1000, wallet.clone()),
        UTXO::new(2000, wallet.clone()),
    ];
    let output_utxos = vec![
        UTXO::new(2500, mallet.clone()),
        UTXO::new(500, mallet.clone()),
    ];
    let transaction_info: TransactionInfo = TransactionInfo::new(input_utxos, output_utxos);

    let signature = match wallet_pk.sign_transaction(&transaction_info) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };

    let transaction = Transaction::new(wallet, mallet, transaction_info, signature).unwrap();

    let serialized_transaction = transaction.serialize();
    println!("serialized_transaction: \n{serialized_transaction}");

    let _: Transaction = serde_json::from_str(&serialized_transaction).unwrap();
}
