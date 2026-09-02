use cleyto_coin::{
    chain::{
        transaction::Transaction,
        utxo::TransactionOutput,
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
        vec![TransactionOutput::new(1000, first_receiver.0.clone())],
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

    let mut transaction = Transaction::new(
        input_utxos,
        vec![
            TransactionOutput::new(200, wallet1.0.clone()),
            TransactionOutput::new(200, wallet2.0),
        ],
    )
    .unwrap();

    fr_wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction)
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
