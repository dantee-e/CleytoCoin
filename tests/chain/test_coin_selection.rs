use cleyto_coin::chain::{utxo::UTXO, wallet::Wallet};

#[test]
fn test_get_utxo_wallet() {
    let (mut wallet1, _) = Wallet::new();

    let input_utxos = vec![
        // Large UTXOs - good for covering big amounts efficiently
        UTXO::new(50000, wallet1.clone()),
        UTXO::new(25000, wallet1.clone()),
        // Medium UTXOs - typical transaction amounts
        UTXO::new(10000, wallet1.clone()),
        UTXO::new(5000, wallet1.clone()),
        // Small UTXOs - test efficiency vs dust management
        UTXO::new(3000, wallet1.clone()),
        UTXO::new(1200, wallet1.clone()),
        UTXO::new(1000, wallet1.clone()),
        // Very small UTXOs - potential dust scenarios
        UTXO::new(300, wallet1.clone()),
    ];

    fn print_utxo_vec(input_utxos: Vec<UTXO>) {
        for utxo in input_utxos {
            println!("utxo: ({})", utxo.value());
        }
    }

    wallet1.add_utxos(input_utxos);
    println!("Checkpoint 1");

    assert_eq!(
        wallet1.get_utxos(50000).unwrap(),
        vec![UTXO::new(50000, wallet1.clone())]
    );
    println!("Checkpoint 2");

    assert!(wallet1.get_utxos(100000000).is_err());
    println!("Checkpoint 3");

    print_utxo_vec(wallet1.get_utxos(30000).unwrap());
    println!("Checkpoint 4");

    print_utxo_vec(wallet1.get_utxos(40000).unwrap());
    println!("Checkpoint 5");

    print_utxo_vec(wallet1.get_utxos(60000).unwrap());
    println!("Checkpoint 6");
}
