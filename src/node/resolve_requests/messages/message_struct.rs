use serde::{Deserialize, Serialize};

use crate::chain::block::{Block, BlockHeader};
use crate::chain::transaction::TransactionHeader;
use crate::node::ConnectedNodeInfo;

#[derive(Serialize, Deserialize)]
pub enum GetDataMessage {
    Block(BlockHeader),
    Transaction(TransactionHeader),
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    CheckBlock(BlockHeader),
    Block(Block),
    NewNode(ConnectedNodeInfo),
    #[allow(dead_code)]
    KeyRefresh,
    CheckTransaction(TransactionHeader),
    GetData(GetDataMessage),
}
