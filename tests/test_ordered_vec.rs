use cleyto_coin::chain::ordered_vector::OrderedVec;
use cleyto_coin::chain::utxo::UTXO;
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn test_ordered_vec() {
    let (wallet1, _) = Wallet::new();

    let input_utxos = vec![
        UTXO::new(50000, wallet1.clone()),
        UTXO::new(32000, wallet1.clone()),
        UTXO::new(25000, wallet1.clone()),
        UTXO::new(15000, wallet1.clone()),
        UTXO::new(12000, wallet1.clone()),
        UTXO::new(10000, wallet1.clone()),
        UTXO::new(8500, wallet1.clone()),
        UTXO::new(7200, wallet1.clone()),
        UTXO::new(6000, wallet1.clone()),
        UTXO::new(5500, wallet1.clone()),
        UTXO::new(3000, wallet1.clone()),
        UTXO::new(2500, wallet1.clone()),
        UTXO::new(2000, wallet1.clone()),
        UTXO::new(1500, wallet1.clone()),
        UTXO::new(1200, wallet1.clone()),
        UTXO::new(1000, wallet1.clone()),
        UTXO::new(800, wallet1.clone()),
        UTXO::new(600, wallet1.clone()),
        UTXO::new(400, wallet1.clone()),
        UTXO::new(300, wallet1.clone()),
    ];

    let vec = OrderedVec::from(input_utxos);

    for i in vec {
        println!("utxo of value {}", i.value());
    }
}
