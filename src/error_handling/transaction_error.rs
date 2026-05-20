use openssl::error::ErrorStack;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub enum TransactionDeserializeError {
    InsufficientFunds,
    MalformedTransaction,
    SerdeError(serde_json::Error),
}
impl fmt::Display for TransactionDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionDeserializeError::InsufficientFunds => write!(f, "Insufficient funds"),
            TransactionDeserializeError::MalformedTransaction => write!(f, "Malformed transaction"),
            TransactionDeserializeError::SerdeError(value) => write!(f, "{}", value),
        }
    }
}
impl std::error::Error for TransactionDeserializeError {}

#[derive(Debug)]
pub enum TransactionError {
    OpenSSLError(ErrorStack),
    InsufficientInputs,
    ValidationError,
    InsufficientFunds,
    ConnectionError(String),
}
impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::OpenSSLError(e) => {
                let mut error = String::new();
                error += "The validation of the transaction was not successful due to some \
                internal OpenSSL error:";
                for i in e.errors() {
                    error += &format!("\n{}", i);
                }
                write!(f, "{}", error)
            }
            TransactionError::ValidationError => {
                write!(
                    f,
                    "The validation of the transaction was not successful, as the signature \
                did not match the provided transaction info."
                )
            }
            TransactionError::InsufficientInputs => {
                write!(
                    f,
                    "The transaction was not validated because the inputed UTXOs where not \
                        sufficient to cover the outuputed UTXOs."
                )
            }
            TransactionError::InsufficientFunds => {
                write!(
                            f,
                            "It wasn't possible to execute the transaction because there weren't enough funds."
                        )
            }
            TransactionError::ConnectionError(_) => {
                write!(
                    f,
                    "The transaction was not sent to the server due to a connection error."
                )
            }
        }
    }
}
impl std::error::Error for TransactionError {}
