use crate::{
    chain::{block::Block, Chain},
    node::resolve_requests::{helpers::HTTPResult, methods::HTTPResponse},
};

pub fn process_new_block(block: Block, chain: &mut Chain) -> HTTPResult {
    if block.previous_hash() == chain.get_last_hash() {
        chain.add_block(block);
        Ok(HTTPResponse::OK(None))
    } else {
        unimplemented!("Chain conflict!");
    }
}
