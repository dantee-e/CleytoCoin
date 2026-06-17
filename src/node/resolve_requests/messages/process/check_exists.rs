use crate::{
    chain::{
        block::BlockHeader,
        transaction::{Transaction, TransactionHeader},
        Chain,
    },
    node::resolve_requests::{
        helpers::HTTPResult,
        messages::{message_struct::GetDataMessage, Message},
        methods::{Content, HTTPResponse},
    },
};

pub fn check_transaction_by_transaction_header(
    transaction_header: TransactionHeader,
    transaction_pool: &[Transaction],
) -> HTTPResult {
    if transaction_pool
        .iter()
        .any(|transaction| transaction.to_header() == transaction_header)
    {
        Ok(HTTPResponse::OK(None))
    } else {
        let value = serde_json::to_value(Message::GetData(GetDataMessage::Transaction(
            transaction_header,
        )))
        .unwrap();
        Ok(HTTPResponse::OK(Some(Content::JSON(value))))
    }
}

pub fn check_block_by_block_header(block_header: BlockHeader, chain: &Chain) -> HTTPResult {
    if chain
        .blocks
        .iter()
        .any(|block| block.to_header() == block_header)
    {
        Ok(HTTPResponse::OK(None))
    } else {
        let value =
            serde_json::to_value(Message::GetData(GetDataMessage::Block(block_header))).unwrap();
        Ok(HTTPResponse::OK(Some(Content::JSON(value))))
    }
}
