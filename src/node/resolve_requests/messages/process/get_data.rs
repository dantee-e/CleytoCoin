use crate::{
    chain::{
        block::BlockHeader,
        transaction::{Transaction, TransactionHeader},
        Chain,
    },
    node::resolve_requests::{
        errors::HTTPResponseError,
        helpers::HTTPResult,
        methods::{Content, HTTPResponse},
    },
};

pub fn process_get_data_block(header: BlockHeader, chain: &Chain) -> HTTPResult {
    let block = chain.find_block_from_header(header);
    if let Some(block) = block {
        let value = serde_json::to_value(block).unwrap();
        Ok(HTTPResponse::OK(Some(Content::JSON(value))))
    } else {
        Err(HTTPResponseError::ResourceNotFound(Some(
            "Could not find requested block".to_string(),
        )))
    }
}

pub fn process_get_data_transaction(
    header: TransactionHeader,
    transaction_pool: &[Transaction],
) -> HTTPResult {
    let transaction = transaction_pool.iter().find(|t| t.to_header() == header);

    if let Some(transaction) = transaction {
        let value = serde_json::to_value(transaction).unwrap();
        Ok(HTTPResponse::OK(Some(Content::JSON(value))))
    } else {
        Err(HTTPResponseError::ResourceNotFound(Some(
            "Could not find requested transaction".to_string(),
        )))
    }
}
