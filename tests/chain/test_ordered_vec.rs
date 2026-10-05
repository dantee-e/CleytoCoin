use cleyto_coin::chain::ordered_vector::OrderedVec;
use cleyto_coin::chain::utxo::{OutPoint, TransactionOutput, UTXO};
use cleyto_coin::chain::wallet::Wallet;

#[test]
fn test_ordered_vec() {
    let (wallet1, _) = Wallet::new();

    let values = [
        50000, 32000, 25000, 15000, 12000, 10000, 8500, 7200, 6000, 5500, 3000,
        2500, 2000, 1500, 1200, 1000, 800, 600, 400, 300,
        // repeated values in different coins must not be deduplicated
        1000, 1000,
    ];

    let input_utxos: Vec<UTXO> = values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            UTXO::new(
                OutPoint {
                    txid: [i as u8; 32],
                    vout: 0,
                },
                TransactionOutput::new(*value, &wallet1),
            )
        })
        .collect();

    let vec = OrderedVec::from(input_utxos);
    assert_eq!(vec.len(), values.len());

    // Ordered from the biggest value to the smallest
    let ordered: Vec<u64> = vec.into_iter().map(|u| u.output.value).collect();
    println!("values: {ordered:?}");
    assert!(ordered.windows(2).all(|pair| pair[0] >= pair[1]));
}
