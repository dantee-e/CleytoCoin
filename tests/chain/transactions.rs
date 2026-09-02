use cleyto_coin::chain::transaction::Transaction;
use cleyto_coin::chain::utxo::{TransactionInput, TransactionOutput};
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn create_transaction() {
    let (wallet_sender, walletpk_sender) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();

    let input_utxos = vec![
        TransactionInput::new(1000, wallet_sender.clone()),
        TransactionInput::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        TransactionOutput::new(2500, wallet_receiver.clone()),
        TransactionOutput::new(500, wallet_receiver.clone()),
    ];
    let mut transaction = Transaction::new(input_utxos, output_utxos).unwrap();

    match walletpk_sender.sign_all_owned_inputs_in_transaction(&mut transaction)
    {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };

    // this will also be verified by the Transaction::new();
    if wallet_sender.verify_transaction_info(&transaction).unwrap() {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }

    println!("transaction.to_string(): {}", transaction);
}

#[test]
fn test_transaction_info_creation() {
    let (wallet_sender, _) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();

    let input_utxos = vec![
        TransactionInput::new(1000, wallet_sender.clone()),
        TransactionInput::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        TransactionOutput::new(2500, wallet_receiver.clone()),
        TransactionOutput::new(500, wallet_receiver.clone()),
    ];
    let transaction_info: Transaction =
        Transaction::new(input_utxos, output_utxos).unwrap();

    println!("transaction info:\n{}", transaction_info);
    println!("{:?}", transaction_info);
}

#[test]
fn sign_and_verify_transaction_info() {
    let (wallet_sender, wallet_pk) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();

    let input_utxos = vec![
        TransactionInput::new(1000, wallet_sender.clone()),
        TransactionInput::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        TransactionOutput::new(2500, wallet_receiver.clone()),
        TransactionOutput::new(500, wallet_receiver.clone()),
    ];
    let mut transaction_info: Transaction =
        Transaction::new(input_utxos, output_utxos).unwrap();

    let signature = match wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction_info)
    {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!(
        "Transaction signature (signed using the wallet_pk):\n{:?}",
        signature
    );

    match transaction_info.verify_signatures() {
        Ok(()) => println!("transaction verified (by the wallet)"),
        Err(_) => println!("transaction not verified"),
    }
}

#[test]
fn serialize_and_deserialize_transaction() {
    let (wallet, wallet_pk) = Wallet::new();
    let (mallet, _) = Wallet::new();

    let input_utxos = vec![
        TransactionInput::new(1000, wallet.clone()),
        TransactionInput::new(2000, wallet.clone()),
    ];
    let output_utxos = vec![
        TransactionOutput::new(2500, mallet.clone()),
        TransactionOutput::new(500, mallet.clone()),
    ];
    let mut transaction = Transaction::new(input_utxos, output_utxos).unwrap();

    wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction)
        .expect("Error while signing transaction");

    let serialized_transaction = transaction.serialize();
    println!("serialized_transaction: \n{serialized_transaction}");

    let _: Transaction = serde_json::from_str(&serialized_transaction).unwrap();
}
