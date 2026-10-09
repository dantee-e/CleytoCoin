use cleyto_coin::{
    chain::{block::Block, Chain},
    node::Node,
};

#[test]
fn serialize_and_deserialize_node() {
    let mut chain = Chain::new(vec![]);

    chain.add_block(Block::test_block(&chain));
    chain.add_block(Block::test_block(&chain));
    chain.add_block(Block::test_block(&chain));
    chain.add_block(Block::test_block(&chain));

    let hashes: Vec<String> =
        chain.blocks.iter().map(|block| block.hash()).collect();

    let (node1, _) = Node::new(chain, "test".to_string());
    let node_json =
        serde_json::to_string(&node1).expect("Could not serialize node");
    println!("{}", node_json);
    let node2: Node =
        serde_json::from_str(&node_json).expect("Could not deserialize node");

    assert_eq!(node2.name, node1.name);
    let restored: Vec<String> = node2
        .state
        .lock()
        .unwrap()
        .chain()
        .blocks
        .iter()
        .map(|block| block.hash())
        .collect();
    assert_eq!(restored, hashes);
}
