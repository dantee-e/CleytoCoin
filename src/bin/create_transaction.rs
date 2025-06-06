use chrono::Utc;
use cleyto_coin::chain::transaction::{Transaction, TransactionInfo, TransactionValidationError};
use cleyto_coin::chain::wallet::Wallet;
use reqwest::Client;
use std::error::Error;
use std::fs;

#[tokio::main]
async fn main() {
    post_json().await.expect("TODO: panic message");
}

fn create_transaction_json() -> String {
    let (wallet1, mut wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();
    let transaction_info = TransactionInfo::new(0.3, Utc::now());
    let signature = wallet1_pk.sign_transaction(&transaction_info).unwrap();
    let transaction = match Transaction::new(wallet1, wallet2, transaction_info, signature) {
        Ok(tx) => tx,
        Err(e) => match e {
            TransactionValidationError::OpenSSLError(_) => panic!("{e}"),
            TransactionValidationError::ValidationError => panic!("Validation error"),
        },
    };

    transaction.serialize()
}

async fn post_json() -> Result<(), Box<dyn Error>> {
    // Initialize the HTTP client
    let client = Client::new();

    // Read the JSON file
    let json_content = fs::read_to_string("src/bin/transaction.json")?;

    // Send the POST request
    let response = client
        .post("http://localhost:9473/submit-transaction")
        .header("Content-Type", "application/json")
        .body(json_content)
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
