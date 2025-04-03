use cleyto_coin::chain::{transaction::TransactionInfo, wallet::Wallet};
use chrono::Utc;

#[test]
fn sign_and_verify_transactioninfo() {
    let (wallet, mut wallet_pk) = Wallet::new();
    let transactioninfo: TransactionInfo = TransactionInfo::new(12345 as f32, Utc::now());

    let signature = match wallet_pk.sign_transaction(&transactioninfo) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!("Transaction signature (signed using the wallet_pk):\n{:?}", signature);

    if wallet.verify_transaction_info(&transactioninfo, &signature) == true {
        println!("transaction verified (by the wallet)");
    } else {
        println!("transaction not verified");
    }
}
