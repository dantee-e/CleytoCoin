use cleyto_coin::{
    chain::{
        transaction::Transaction,
        utxo::{TransactionInput, TransactionOutput},
        wallet::{Wallet, WalletPK},
        Chain,
    },
    node::{Message, Node},
};

use crate::mock_stream::request;

fn create_multiple_nodes(n: usize) -> (Vec<Node>, Wallet, WalletPK) {
    let (mut first_receiver, first_receiver_pk) = Wallet::new();
    let chain = Chain::new(vec![TransactionOutput::new(1000, &first_receiver)]);
    chain.load_wallet_utxos(&mut first_receiver);
    (
        (0..n)
            .map(|i| Node::new(chain.clone(), format!("node{i}")).0)
            .collect(),
        first_receiver,
        first_receiver_pk,
    )
}

#[tokio::test]
#[ignore = "aaaareason"]
async fn use_multiple_nodes() {
    let (mut nodes, fr_wallet, fr_wallet_pk) = create_multiple_nodes(3);

    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();
    // let wallet3 = Wallet::new();
    // let wallet4 = Wallet::new();
    // let wallet5 = Wallet::new();

    let node1 = nodes.first_mut().unwrap();
    let state1 = node1.state.clone();

    let coins = fr_wallet.get_utxos(400).unwrap();
    for coin in &coins {
        println!("UTXO = {}", coin);
    }

    let mut transaction = Transaction::new(
        TransactionInput::from_utxos(&coins),
        vec![
            TransactionOutput::new(200, &wallet1.0),
            TransactionOutput::new(200, &wallet2.0),
        ],
    )
    .unwrap();

    fr_wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
        .unwrap();

    let message_str =
        serde_json::to_string(&Message::Transaction(transaction)).unwrap();

    let result1 =
        request("POST", "/messages", Some(&message_str), state1.clone())
            .await
            .unwrap();

    if let Some(str) = result1 {
        println!("Result is {str}");
    }

    assert_eq!(state1.lock().unwrap().transactions_pool().len(), 1);
    // TODO: once nodes are connected to each other, check that the
    // transaction reached the other nodes' pools too
}
