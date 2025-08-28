use cleyto_coin::chain::transaction::{Transaction, TransactionInfo};
use cleyto_coin::chain::wallet::Wallet;
use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() {
    post_json().await.expect("TODO: panic message");
}

// fn create_transaction_json() -> String {
//     let (wallet1, mut wallet1_pk) = Wallet::new();
//     let (wallet2, _) = Wallet::new();
//     let transaction_info = TransactionInfo::new(0.3, Utc::now());
//     let signature = wallet1_pk.sign_transaction(&transaction_info).unwrap();
//     let transaction = match Transaction::new(wallet1, wallet2, transaction_info, signature) {
//         Ok(tx) => tx,
//         Err(e) => match e {
//             TransactionValidationError::OpenSSLError(_) => panic!("{e}"),
//             TransactionValidationError::ValidationError => panic!("Validation error"),
//         },
//     };
//
//     transaction.serialize()
// }

async fn post_json() -> Result<(), Box<dyn Error>> {
    // Initialize the HTTP client
    let client = Client::new();

    let (wallet_sender, walletpk_sender) = Wallet::new();
    let (wallet_receiver, _) = Wallet::new();
    let transactioninfo: TransactionInfo = TransactionInfo::new(12345);

    let signature = match walletpk_sender.sign_transaction(&transactioninfo) {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };
    println!(
        "Transaction signature (signed using the wallet_pk):\n{:?}",
        signature
    );

    let transaction: Transaction =
        Transaction::new(wallet_sender, wallet_receiver, transactioninfo, signature).unwrap();
    let transaction_json = transaction.serialize();

    // Send the POST request
    let response = client
        .post("http://localhost:9473/submit-transaction")
        .header("Content-Type", "application/json")
        .body(transaction_json)
        .send()
        .await?;

    // Check the response status
    let status = response.status();
    let response_body = response.text().await?;

    // Print the response
    println!("Response Status: {}", status);
    println!("Response Body: {}", response_body);

    Ok(())
}
