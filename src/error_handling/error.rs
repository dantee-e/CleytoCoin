use super::transaction_error::{TransactionDeserializeError, TransactionError};

#[derive(Debug)]
pub enum CleytonError {
    KillServerError(String),
    TransactionError(TransactionError),
    TransactionDeserializeError(TransactionDeserializeError),
}
