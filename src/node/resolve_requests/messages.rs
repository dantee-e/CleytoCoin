use openssl::pkey::{PKey, Public};
use serde::{Deserialize, Serialize};

use crate::chain::block::{Block, BlockHeader};
use crate::chain::wallet::{deserialize_public_key, serialize_public_key};

#[derive(Serialize, Deserialize)]
pub struct NewNodeMessage {
    node_id: String,
    #[serde(
        serialize_with = "serialize_public_key",
        deserialize_with = "deserialize_public_key"
    )]
    public_key: PKey<Public>,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    CheckBlock(BlockHeader),
    Block(Block),
    NewNode(NewNodeMessage), // string is node id
    #[allow(dead_code)]
    KeyRefresh,
    CheckTransaction(),
}
