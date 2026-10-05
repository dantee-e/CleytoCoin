use cleyto_coin::chain::transaction::Transaction;
use cleyto_coin::chain::utxo::{TransactionInput, TransactionOutput, UTXO};
use cleyto_coin::chain::wallet::{Wallet, WalletPK};
use cleyto_coin::chain::Chain;

/// Chain whose genesis gives `values` to a new wallet. Returns the chain, the wallet and its coins
fn funded_chain(values: &[u64]) -> (Chain, Wallet, WalletPK, Vec<UTXO>) {
    let (wallet, wallet_pk) = Wallet::new();
    let chain = Chain::new(
        values
            .iter()
            .map(|value| TransactionOutput::new(*value, &wallet))
            .collect(),
    );
    let coins = chain.utxos_owned_by(&wallet.to_public_key());
    (chain, wallet, wallet_pk, coins)
}

#[test]
fn create_transaction() {
    let (chain, _, walletpk_sender, coins) = funded_chain(&[1000, 2000]);
    let (wallet_receiver, _) = Wallet::new();

    let output_utxos = vec![
        TransactionOutput::new(2500, &wallet_receiver),
        TransactionOutput::new(500, &wallet_receiver),
    ];
    let mut transaction =
        Transaction::new(TransactionInput::from_utxos(&coins), output_utxos)
            .unwrap();

    let signed = walletpk_sender
        .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
        .expect("error while signing transaction");
    assert_eq!(signed, 2);

    assert_eq!(chain.validate_transaction(&transaction).unwrap(), 0);

    println!("transaction.to_string(): {}", transaction);
}

#[test]
fn test_transaction_info_creation() {
    let (_, _, wallet_pk, coins) = funded_chain(&[1000, 2000]);
    let (wallet_receiver, _) = Wallet::new();

    let output_utxos = vec![
        TransactionOutput::new(2500, &wallet_receiver),
        TransactionOutput::new(500, &wallet_receiver),
    ];
    let mut transaction_info: Transaction =
        Transaction::new(TransactionInput::from_utxos(&coins), output_utxos)
            .unwrap();

    println!("transaction info:\n{}", transaction_info);
    println!("{:?}", transaction_info);

    // Starts unsigned, and its new outputs are addressed by its txid
    assert!(transaction_info.inputs.iter().all(|i| i.signature.is_none()));
    let txid = transaction_info.txid();
    let new_utxos = transaction_info.new_utxos();
    assert_eq!(new_utxos.len(), 2);
    for (vout, utxo) in new_utxos.iter().enumerate() {
        assert_eq!(utxo.outpoint.txid, txid);
        assert_eq!(utxo.outpoint.vout, vout as u32);
    }

    // Signing doesn't change the txid
    wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction_info, &coins)
        .unwrap();
    assert_eq!(transaction_info.txid(), txid);
}

#[test]
fn sign_and_verify_transaction_info() {
    let (chain, _, wallet_pk, coins) = funded_chain(&[1000, 2000]);
    let (wallet_receiver, _) = Wallet::new();

    // 2500 out of 3000, so 500 is left as fee
    let output_utxos = vec![TransactionOutput::new(2500, &wallet_receiver)];
    let mut transaction_info: Transaction =
        Transaction::new(TransactionInput::from_utxos(&coins), output_utxos)
            .unwrap();

    wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction_info, &coins)
        .expect("error while signing transaction");

    assert_eq!(chain.validate_transaction(&transaction_info).unwrap(), 500);
}

#[test]
fn reject_unsigned_overspent_and_duplicate_inputs() {
    let (chain, _, wallet_pk, coins) = funded_chain(&[1000]);
    let (receiver, _) = Wallet::new();
    let inputs = TransactionInput::from_utxos(&coins);

    // unsigned
    let unsigned = Transaction::new(
        inputs.clone(),
        vec![TransactionOutput::new(1000, &receiver)],
    )
    .unwrap();
    assert!(chain.validate_transaction(&unsigned).is_err());

    // outputs bigger than inputs
    let mut overspent = Transaction::new(
        inputs.clone(),
        vec![TransactionOutput::new(1001, &receiver)],
    )
    .unwrap();
    wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut overspent, &coins)
        .unwrap();
    assert!(chain.validate_transaction(&overspent).is_err());

    // same coin twice
    let duplicated = [inputs.clone(), inputs].concat();
    assert!(Transaction::new(
        duplicated,
        vec![TransactionOutput::new(2000, &receiver)]
    )
    .is_err());
}

#[test]
fn reject_signature_from_wrong_owner() {
    let (chain, _, _, coins) = funded_chain(&[1000]);
    let (thief, thief_pk) = Wallet::new();

    let mut transaction = Transaction::new(
        TransactionInput::from_utxos(&coins),
        vec![TransactionOutput::new(1000, &thief)],
    )
    .unwrap();

    // The thief doesn't own the coin, so nothing gets signed
    let signed = thief_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
        .unwrap();
    assert_eq!(signed, 0);

    // Even forcing the thief's signature in, it doesn't verify
    let mut forged = transaction.clone();
    let fake_coins: Vec<UTXO> = coins
        .iter()
        .map(|c| {
            UTXO::new(c.outpoint, TransactionOutput::new(c.output.value, &thief))
        })
        .collect();
    thief_pk
        .sign_all_owned_inputs_in_transaction(&mut forged, &fake_coins)
        .unwrap();
    assert!(chain.validate_transaction(&forged).is_err());
}

#[test]
fn serialize_and_deserialize_transaction() {
    let (_, _, wallet_pk, coins) = funded_chain(&[1000, 2000]);
    let (mallet, _) = Wallet::new();

    let output_utxos = vec![
        TransactionOutput::new(2500, &mallet),
        TransactionOutput::new(500, &mallet),
    ];
    let mut transaction =
        Transaction::new(TransactionInput::from_utxos(&coins), output_utxos)
            .unwrap();

    wallet_pk
        .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
        .expect("Error while signing transaction");

    let serialized_transaction = transaction.serialize();
    println!("serialized_transaction: \n{serialized_transaction}");

    let deserialized: Transaction =
        serde_json::from_str(&serialized_transaction).unwrap();
    assert_eq!(deserialized.txid(), transaction.txid());
}
