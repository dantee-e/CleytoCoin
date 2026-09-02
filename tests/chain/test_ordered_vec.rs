use cleyto_coin::chain::ordered_vector::OrderedVec;
use cleyto_coin::chain::utxo::TransactionOutput;
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn test_ordered_vec() {
    let (wallet1, _) = Wallet::new();

    let input_utxos = vec![
        TransactionOutput::new(50000, wallet1.clone()),
        TransactionOutput::new(32000, wallet1.clone()),
        TransactionOutput::new(25000, wallet1.clone()),
        TransactionOutput::new(15000, wallet1.clone()),
        TransactionOutput::new(12000, wallet1.clone()),
        TransactionOutput::new(10000, wallet1.clone()),
        TransactionOutput::new(8500, wallet1.clone()),
        TransactionOutput::new(7200, wallet1.clone()),
        TransactionOutput::new(6000, wallet1.clone()),
        TransactionOutput::new(5500, wallet1.clone()),
        TransactionOutput::new(3000, wallet1.clone()),
        TransactionOutput::new(2500, wallet1.clone()),
        TransactionOutput::new(2000, wallet1.clone()),
        TransactionOutput::new(1500, wallet1.clone()),
        TransactionOutput::new(1200, wallet1.clone()),
        TransactionOutput::new(1000, wallet1.clone()),
        TransactionOutput::new(800, wallet1.clone()),
        TransactionOutput::new(600, wallet1.clone()),
        TransactionOutput::new(400, wallet1.clone()),
        TransactionOutput::new(300, wallet1.clone()),
    ];

    let vec = OrderedVec::from(input_utxos);

    for i in vec {
        println!("utxo of value {}", i.value);
    }
}
