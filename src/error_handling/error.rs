use std::io;

use super::transaction_error::{TransactionDeserializeError, TransactionError};

#[derive(Debug)]
pub enum CleytonError {
    KillServerError(String),
    TransactionError(TransactionError),
    TransactionDeserializeError(TransactionDeserializeError),
    BlockSerializationError(serde_json::Error),
    BlockDeserializationError(serde_json::Error),
    BlockNotFound,
    ReadWriteError(io::Error),
    LastBlockLessThanZero,
}

impl From<io::Error> for CleytonError {
    fn from(value: io::Error) -> Self {
        CleytonError::ReadWriteError(value)
    }
}
