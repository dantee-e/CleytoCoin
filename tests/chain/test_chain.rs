use cleyto_coin::chain::testing::test_chain;
use cleyto_coin::chain::utxo::UTXO;
use cleyto_coin::chain::Chain;
use cleyto_coin::node::data::{remove_block_by_hash, write_chain_blocks};

#[test]
fn creating_test_chain() {
    let chain = test_chain();

    // genesis + 4 blocks
    assert_eq!(chain.blocks.len(), 5);

    // Every transaction in test_chain has no fee, so all the money from the
    // genesis is still there. Unspent: wallet_1's 10000 from block 3 plus the
    // 5 outputs of block 4.
    let unspent: Vec<UTXO> = chain.utxos.clone().into();
    assert_eq!(unspent.len(), 6);
    assert_eq!(UTXO::sum(&unspent), 100000);
}

#[test]
#[ignore = "nao quero lidar com isso agr"]
fn writing_test_chain() {
    use cleyto_coin::chain::testing::test_chain;
    let chain = test_chain();

    let hashes: Vec<String> =
        chain.blocks.iter().map(|block| block.hash()).collect();

    write_chain_blocks(&chain).unwrap();

    for hash in hashes {
        remove_block_by_hash(hash).unwrap();
    }
}

#[test]
fn serialize_and_deserialize_chain() {
    let chain = test_chain();

    let serialized = serde_json::to_string(&chain).unwrap();
    let deserialized: Chain = serde_json::from_str(&serialized).unwrap();

    let hashes = |chain: &Chain| -> Vec<String> {
        chain.blocks.iter().map(|block| block.hash()).collect()
    };
    assert_eq!(hashes(&deserialized), hashes(&chain));

    let mut original: Vec<UTXO> = chain.utxos.clone().into();
    let mut restored: Vec<UTXO> = deserialized.utxos.clone().into();
    original.sort();
    restored.sort();
    assert_eq!(restored, original);
}
