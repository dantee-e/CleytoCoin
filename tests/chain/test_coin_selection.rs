use cleyto_coin::chain::{
    utxo::{OutPoint, TransactionOutput, UTXO},
    wallet::Wallet,
};

/// Coin with a made up, unique outpoint
fn coin(i: u8, value: u64, owner: &Wallet) -> UTXO {
    UTXO::new(
        OutPoint {
            txid: [i; 32],
            vout: 0,
        },
        TransactionOutput::new(value, owner),
    )
}

#[test]
fn test_get_utxo_wallet() {
    let (mut wallet1, _) = Wallet::new();

    let utxos = vec![
        // Large UTXOs - good for covering big amounts efficiently
        coin(0, 50000, &wallet1),
        coin(1, 25000, &wallet1),
        // Medium UTXOs - typical transaction amounts
        coin(2, 10000, &wallet1),
        coin(3, 5000, &wallet1),
        // Small UTXOs - test efficiency vs dust management
        coin(4, 3000, &wallet1),
        coin(5, 1200, &wallet1),
        coin(6, 1000, &wallet1),
        // Very small UTXOs - potential dust scenarios
        coin(7, 300, &wallet1),
    ];

    /// The selection must cover the amount, using each of the wallet's coins
    /// at most once
    fn check_selection(selected: &[UTXO], available: &[UTXO], amount: u64) {
        for utxo in selected {
            println!("utxo: ({})", utxo.output.value);
        }
        assert!(
            UTXO::sum(&selected.to_vec()) >= amount,
            "selection doesn't cover {amount}"
        );
        assert!(selected.iter().all(|u| available.contains(u)));
        let mut outpoints: Vec<_> = selected.iter().map(|u| u.outpoint).collect();
        outpoints.sort();
        outpoints.dedup();
        assert_eq!(outpoints.len(), selected.len(), "a coin was selected twice");
    }

    wallet1.add_utxos(utxos.clone());

    // exact match
    assert_eq!(wallet1.get_utxos(50000).unwrap(), vec![coin(0, 50000, &wallet1)]);

    // more than the whole balance
    assert!(wallet1.get_utxos(100000000).is_err());

    for amount in [30000, 40000, 60000] {
        check_selection(&wallet1.get_utxos(amount).unwrap(), &utxos, amount);
    }
}
