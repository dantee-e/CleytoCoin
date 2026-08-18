use cleyto_coin::{
    chain::{
        transaction::{Transaction, TransactionInfo},
        utxo::UTXO,
        wallet::{Wallet, WalletPK},
        Chain,
    },
    node::{Message, Node},
};

use crate::mock_stream::request;

fn create_multiple_nodes(n: usize) -> (Vec<Node>, Wallet, WalletPK) {
    let first_receiver = Wallet::new();
    let chain = Chain::new(
        first_receiver.0.clone(),
        vec![UTXO::new(1000, first_receiver.0.clone())],
    );
    (
        (0..n)
            .into_iter()
            .map(|i| Node::new(chain.clone(), format!("node{i}")).0)
            .collect(),
        first_receiver.0,
        first_receiver.1,
    )
}

#[tokio::test]
async fn use_multiple_nodes() {
    let (mut nodes, fr_wallet, fr_wallet_pk) = create_multiple_nodes(3);

    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();
    // let wallet3 = Wallet::new();
    // let wallet4 = Wallet::new();
    // let wallet5 = Wallet::new();

    let node1 = nodes.first_mut().unwrap();
    let state1 = node1.state.clone();

    let input_utxos = fr_wallet.get_utxos(400).unwrap();
    for i in &input_utxos {
        println!("UTXO = {}", i);
    }

    let transaction_info = TransactionInfo::new(
        input_utxos,
        vec![UTXO::new(200, wallet1.0.clone()), UTXO::new(200, wallet2.0)],
    );

    let signature = fr_wallet_pk.sign_transaction(&transaction_info).unwrap();

    let transaction =
        Transaction::new(fr_wallet, wallet1.0, transaction_info, signature)
            .unwrap();

    let message_str =
        serde_json::to_string(&Message::Transaction(transaction)).unwrap();

    let result1 = request("POST", "/messages", Some(&message_str), state1)
        .await
        .unwrap();

    if let Some(str) = result1 {
        println!("Result is {str}");
    }
}
