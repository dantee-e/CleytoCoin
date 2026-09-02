use cleyto_coin::chain::transaction::Transaction;
use cleyto_coin::chain::utxo::{TransactionInput, TransactionOutput};
use cleyto_coin::chain::wallet::Wallet;
use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() {
    todo!("The post_json function is not operational right now, because it only creates a random transacion without any arguments");
    #[allow(unreachable_code)]
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

    let input_utxos = vec![
        TransactionInput::new(1000, wallet_sender.clone()),
        TransactionInput::new(2000, wallet_sender.clone()),
    ];
    let output_utxos = vec![
        TransactionOutput::new(2500, wallet_receiver.clone()),
        TransactionOutput::new(500, wallet_sender.clone()),
    ];
    let mut transaction: Transaction =
        Transaction::new(input_utxos, output_utxos).unwrap();

    match walletpk_sender.sign_all_owned_inputs_in_transaction(&mut transaction)
    {
        Ok(signed_hashed_message) => signed_hashed_message,
        _ => panic!("error while signing transaction"),
    };

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
