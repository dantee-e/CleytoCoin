use cleyto_coin::chain::{
    block::Block,
    transaction::Transaction,
    utxo::{TransactionInput, TransactionOutput, UTXO},
    wallet::Wallet,
    Chain,
};

#[test]
fn create_block_and_add_chain() {
    let (wallet1, wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    let mut chain = Chain::new(vec![
        TransactionOutput::new(1000, &wallet1),
        TransactionOutput::new(2000, &wallet1),
    ]);

    let coins = chain.utxos_owned_by(&wallet1.to_public_key());
    let outputs = vec![
        TransactionOutput::new(2500, &wallet2),
        TransactionOutput::new(500, &wallet2),
    ];
    let mut transaction: Transaction =
        Transaction::new(TransactionInput::from_utxos(&coins), outputs)
            .unwrap();

    wallet1_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
        .unwrap();

    chain.validate_transaction(&transaction).unwrap();

    let block = Block::new(&mut chain, vec![transaction]);

    chain.add_block(block);

    assert!(chain.utxos_owned_by(&wallet1.to_public_key()).is_empty());
    assert_eq!(UTXO::sum(&chain.utxos_owned_by(&wallet2.to_public_key())), 3000);
}
