use chrono::Utc;
use cleyto_coin::chain::transaction::{Transaction, TransactionInfo};
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn create_transaction() {
    let (wallet_sender, mut walletpk_sender) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();
    let transactioninfo: TransactionInfo = TransactionInfo::new(12345 as f32, Utc::now());

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
        == true
    {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }

    let transaction: Transaction =
        Transaction::new(wallet_sender, wallet_receiver, transactioninfo, signature).unwrap();

    println!("transaction.to_string(): {}", transaction.to_string());
}
