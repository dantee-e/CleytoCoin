use cleyto_coin::{
    chain::{block::Block, Chain},
    node::Node,
};

fn create_multiple_nodes(n: usize) -> Vec<Node> {
    let mut chain = Chain::new();
    chain.add_block(Block::genesis_block());
    (0..n)
        .into_iter()
        .map(|i| Node::new(chain.clone(), format!("node{i}")).0)
        .collect()
}

#[test]
fn use_multiple_nodes() {
    let mut nodes = create_multiple_nodes(3);

    let node1 = nodes.first_mut().unwrap();
    Node::handle_connection(node1, stream)
}
