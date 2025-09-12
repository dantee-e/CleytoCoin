use cleyto_coin::chain::{utxo::UTXO, wallet::Wallet};

#[test]
fn test_get_utxo_wallet() {
    let (mut wallet1, _) = Wallet::new();

    let input_utxos = vec![
        // Large UTXOs - good for covering big amounts efficiently
        UTXO::new(50000, wallet1.clone()),
        UTXO::new(32000, wallet1.clone()),
        UTXO::new(25000, wallet1.clone()),
        // Medium UTXOs - typical transaction amounts
        UTXO::new(15000, wallet1.clone()),
        UTXO::new(15000, wallet1.clone()),
        UTXO::new(12000, wallet1.clone()),
        UTXO::new(10500, wallet1.clone()),
        UTXO::new(8500, wallet1.clone()),
        UTXO::new(7200, wallet1.clone()),
        UTXO::new(6000, wallet1.clone()),
        UTXO::new(5500, wallet1.clone()),
        UTXO::new(5000, wallet1.clone()),
        // Small UTXOs - test efficiency vs dust management
        UTXO::new(3000, wallet1.clone()),
        UTXO::new(2500, wallet1.clone()),
        UTXO::new(2000, wallet1.clone()),
        UTXO::new(1500, wallet1.clone()),
        UTXO::new(1200, wallet1.clone()),
        UTXO::new(1000, wallet1.clone()),
        // Very small UTXOs - potential dust scenarios
        UTXO::new(800, wallet1.clone()),
        UTXO::new(600, wallet1.clone()),
        UTXO::new(400, wallet1.clone()),
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

    print_utxo_vec(wallet1.get_utxos(25461).unwrap());
    println!("Checkpoint 6");

    // let (wallet2, wallet2_pk) = Wallet::new();
    //
    // let input_utxos = vec![
    //     UTXO::new(1000, wallet1.clone()),
    //     UTXO::new(2000, wallet1.clone()),
    // ];
    // let output_utxos = vec![
    //     UTXO::new(2500, wallet2.clone()),
    //     UTXO::new(500, wallet2.clone()),
    // ];
    // let transaction_info = TransactionInfo::new(input_utxos, output_utxos);
    // let signature = wallet2_pk.sign_transaction(&transaction_info).unwrap();
    // let transaction = Transaction::new(wallet2, wallet1, transaction_info, signature);
}
