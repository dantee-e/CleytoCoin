use std::path::PathBuf;

use crate::{
    chain::{block::Block, Chain},
    configs::{Config, ConfigPaths},
    error_handling::{CleytoResult, CleytonError},
};

pub fn write_block(block: &Block) -> CleytoResult<()> {
    let last_block = Config::get().last_block();

    let serialized_block =
        serde_json::to_string(&block).map_err(CleytonError::BlockSerializationError)?;

    let block_path = PathBuf::from(format!(
        "{}/block_{}_{}.blk",
        ConfigPaths::get().block_dir,
        last_block,
        block.get_hash(),
    ));

    if !std::fs::exists(ConfigPaths::get().block_dir)? {
        std::fs::create_dir(ConfigPaths::get().block_dir)?;
    }

    // write block
    std::fs::write(block_path, serialized_block)?;
    // update last_block
    std::fs::write(ConfigPaths::get().last_block, (last_block + 1).to_string())?;

    Ok(())
}

pub fn read_block_by_hash(hash: &String) -> CleytoResult<Block> {
    let block_paths =
        std::fs::read_dir(PathBuf::from(format!("{}/", ConfigPaths::get().block_dir,))).unwrap();

    let regex = regex::Regex::new(format!("block_.*_{hash}\\.blk").as_str()).unwrap();

    let block_path = if let Some(v) = block_paths.map(|v| v.unwrap()).find(|file| {
        let file_name = file.file_name().into_string().unwrap();
        regex.is_match(&file_name)
    }) {
        v
    } else {
        return Err(CleytonError::BlockNotFound);
    };

    let serialized_block =
        std::fs::read_to_string(block_path.path()).map_err(|_| CleytonError::BlockNotFound)?;

    serde_json::from_str(&serialized_block).map_err(CleytonError::BlockDeserializationError)
}

pub fn read_block_by_number(block_number: &u32) -> CleytoResult<Block> {
    let block_paths =
        std::fs::read_dir(PathBuf::from(format!("{}/", ConfigPaths::get().block_dir,))).unwrap();

    let regex = regex::Regex::new(format!("block_{block_number}_.*\\.blk").as_str()).unwrap();

    let block_path = if let Some(v) = block_paths.map(|v| v.unwrap()).find(|file| {
        let file_name = file.file_name().into_string().unwrap();
        regex.is_match(&file_name)
    }) {
        v
    } else {
        return Err(CleytonError::BlockNotFound);
    };

    let serialized_block =
        std::fs::read_to_string(block_path.path()).map_err(|_| CleytonError::BlockNotFound)?;

    serde_json::from_str(&serialized_block).map_err(CleytonError::BlockDeserializationError)
}

// I'm leaving those here to maybe implement a better one later on,
// right now they are a bit naive
pub fn check_block_is_registered_by_number(block_number: u32) -> bool {
    read_block_by_number(&block_number).is_ok()
}

pub fn check_block_is_registered_by_hash(hash: &String) -> bool {
    read_block_by_hash(hash).is_ok()
}

pub fn write_chain_blocks(chain: &Chain) -> CleytoResult<()> {
    println!("Checkpoint 2");
    for block in &chain.blocks {
        println!("Checkpoint 3");
        write_block(block)?;
        println!("Checkpoint 4");
    }
    Ok(())
}

pub fn remove_block_by_hash(hash: String) -> CleytoResult<()> {
    let block_paths =
        std::fs::read_dir(PathBuf::from(format!("{}/", ConfigPaths::get().block_dir,))).unwrap();

    let regex = regex::Regex::new(format!("block_(.*)_{hash}\\.blk").as_str()).unwrap();

    let (block_path, block_number) = if let Some(v) =
        block_paths.map(|v| v.unwrap()).find_map(|file| {
            let file_name = file.file_name().into_string().unwrap();
            let caps_opt = regex.captures(&file_name);
            if let Some(caps) = caps_opt {
                let blk_number: u32 = str::parse(&caps[0]).unwrap();
                Some((file, blk_number))
            } else {
                None
            }
        }) {
        v
    } else {
        return Err(CleytonError::BlockNotFound);
    };

    std::fs::remove_file(block_path.path())?;

    if block_number == Config::get().last_block() {
        Config::get().update_last_block(-1)?;
    }

    Ok(())
}

pub fn remove_block_by_number(block_number: u32) -> CleytoResult<()> {
    let block_paths =
        std::fs::read_dir(PathBuf::from(format!("{}/", ConfigPaths::get().block_dir,))).unwrap();

    let regex = regex::Regex::new(format!("block_{block_number}_.*\\.blk").as_str()).unwrap();

    let block_path = if let Some(v) = block_paths.map(|v| v.unwrap()).find(|file| {
        let file_name = file.file_name().into_string().unwrap();
        regex.is_match(&file_name)
    }) {
        v
    } else {
        return Err(CleytonError::BlockNotFound);
    };

    std::fs::remove_file(block_path.path())?;

    Ok(())
}

/// If no hash is provided, reads from block number 0
pub fn read_chain(block_hashes: Option<Vec<String>>) -> CleytoResult<Chain> {
    let mut chain = Chain::new();

    if let Some(block_hashes) = block_hashes {
        for hash in block_hashes {
            let block = read_block_by_hash(&hash)?;
            chain.add_block(block);
        }
    }

    Ok(chain)
}
