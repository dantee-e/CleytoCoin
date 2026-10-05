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
            TransactionDeserializeError::InsufficientFunds => {
                write!(f, "Insufficient funds")
            }
            TransactionDeserializeError::MalformedTransaction => {
                write!(f, "Malformed transaction")
            }
            TransactionDeserializeError::SerdeError(value) => {
                write!(f, "{}", value)
            }
        }
    }
}
impl std::error::Error for TransactionDeserializeError {}

#[derive(Debug)]
pub enum TransactionError {
    OpenSSLError(ErrorStack),
    InsufficientInputs,
    ValidationError,
    UnsignedInput,
    InsufficientFunds,
    ConnectionError(String),
    NoInputs,
    DuplicateInput,
    UtxoNotFound,
    ValueOverflow,
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
            TransactionError::UnsignedInput => {
                write!(
                            f,
                            "It wasn't possible to validate the transaction due to a missing signature for one of the inputs"
                        )
            }
            TransactionError::NoInputs => {
                write!(
                    f,
                    "The transaction has no inputs. Only coinbase transactions, created as part \
                    of a block, can have no inputs."
                )
            }
            TransactionError::DuplicateInput => {
                write!(f, "The transaction spends the same output more than once.")
            }
            TransactionError::UtxoNotFound => {
                write!(
                    f,
                    "One of the inputs spends an output that doesn't exist or was already spent."
                )
            }
            TransactionError::ValueOverflow => {
                write!(f, "The values of the transaction overflow a u64.")
            }
        }
    }
}
impl std::error::Error for TransactionError {}
