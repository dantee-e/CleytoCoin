use cleyto_coin::chain::testing::test_chain;
use cleyto_coin::node::data::{remove_block_by_hash, write_chain_blocks};

#[test]
fn creating_test_chain() {
    let _ = test_chain();
}

#[test]
fn writing_test_chain() {
    use cleyto_coin::chain::testing::test_chain;
    let chain = test_chain();

    let hashes: Vec<String> = chain.blocks.iter().map(|block| block.get_hash()).collect();

    write_chain_blocks(&chain).unwrap();

    for hash in hashes {
        remove_block_by_hash(hash).unwrap();
    }
}
