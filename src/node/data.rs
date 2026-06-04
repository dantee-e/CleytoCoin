use std::path::PathBuf;

use crate::{
    chain::block::Block,
    configs::{Config, ConfigPaths},
    error_handling::{CleytoResult, CleytonError},
};

pub fn write_block(block: Block) -> CleytoResult<()> {
    let last_block = Config::get().last_block();

    let serialized_block =
        serde_json::to_string(&block).map_err(CleytonError::BlockSerializationError)?;

    let block_path = PathBuf::from(format!(
        "{}/block_{}.blk",
        ConfigPaths::get().block_dir,
        last_block
    ));
    Ok(std::fs::write(block_path, serialized_block)?)
}

pub fn read_block(block_number: u32) -> CleytoResult<Block> {
    let block_path = PathBuf::from(format!(
        "{}/block_{}.blk",
        ConfigPaths::get().block_dir,
        block_number,
    ));

    let serialized_block =
        std::fs::read_to_string(block_path).map_err(|_| CleytonError::BlockNotFound)?;

    serde_json::from_str(&serialized_block).map_err(CleytonError::BlockDeserializationError)
}

pub fn check_block_is_registered(block_number: u32, hash: String) -> bool {
    let block = match read_block(block_number) {
        Ok(v) => v,
        Err(_) => return false,
    };

    true
}
