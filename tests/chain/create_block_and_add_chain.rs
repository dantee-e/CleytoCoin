use cleyto_coin::chain::{
    block::Block,
    transaction::Transaction,
    utxo::{TransactionInput, UTXO},
    wallet::Wallet,
    Chain,
};

#[test]
fn create_block_and_add_chain() {
    let (wallet1, wallet1_pk) = Wallet::new();
    let (wallet2, _) = Wallet::new();

    let input_utxos = vec![
        TransactionInput::new(1000, wallet1.clone()),
        TransactionInput::new(2000, wallet1.clone()),
    ];
    let output_utxos = vec![
        UTXO::new(2500, wallet2.clone()),
        UTXO::new(500, wallet2.clone()),
    ];
    let mut transaction: Transaction =
        Transaction::new(input_utxos, output_utxos).unwrap();

    wallet1_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction)
        .unwrap();

    transaction.verify_signatures().unwrap();

    let mut chain = Chain::new(Wallet::null_wallet(), vec![]);

    let block = Block::new(&mut chain, vec![transaction]);

    chain.add_block(block);
}
