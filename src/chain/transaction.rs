use super::wallet::Wallet;
use chrono::{DateTime, Utc};
use openssl::error::ErrorStack;
use openssl::sha::Sha256;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
// ---------------------------------------------- TransactionInfo definition -----------------------
pub struct TransactionInfo {
    pub value: i64,
    pub date: DateTime<Utc>,
}

impl TransactionInfo {
    pub fn new(value: i64) -> TransactionInfo {
        let date = Utc::now();
        Self { value, date }
    }
}

impl Display for TransactionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VALUE::{}::TIME::{}", self.value, self.date)
    }
}
// -------------------------------------------------------------------------------------------------

// --------------------------------------- Transaction Serialization Utils -------------------------

// TODO move this to a errors file
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
pub enum TransactionValidationError {
    OpenSSLError(ErrorStack),
    ValidationError,
}
impl fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionValidationError::OpenSSLError(e) => {
                let mut error = String::new();
                error += "The validation of the transaction was not successful due to some \
                internal OpenSSL error:";
                for i in e.errors() {
                    error += &format!("\n{}", i);
                }
                write!(f, "{}", error)
            }
            TransactionValidationError::ValidationError => {
                write!(
                    f,
                    "The validation of the transaction was not successful, as the signature \
                did not match the provided transaction info."
                )
            }
        }
    }
}
impl std::error::Error for TransactionValidationError {}

// -------------------------------------------------------------------------------------------------

// ------------------------------------- Transaction definition ------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Wallet,
    pub receiver: Wallet,
    pub signature: Vec<u8>,
    pub transaction_info: TransactionInfo,
    pub txid: [u8; 32],
}

impl Transaction {
    pub fn new(
        sender: Wallet,
        receiver: Wallet,
        transaction_info: TransactionInfo,
        signature: Vec<u8>,
    ) -> Result<Self, TransactionValidationError> {
        let mut transaction = Self {
            sender,
            receiver,
            signature,
            transaction_info,
            txid: [0; 32], // This could be optimized by avoiding the creation of this Vec, which
                           // serves no function on its own, but I don't really see that being a problem
        };

        let to_hash = transaction.to_string();
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash.as_bytes());
        transaction.txid = hasher.finish().to_owned();

        match transaction.verify() {
            Ok(()) => Ok(transaction),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn verify(&self) -> Result<(), TransactionValidationError> {
        match self
            .sender
            .verify_transaction_info(&self.transaction_info, &self.signature)
        {
            Ok(value) => match value {
                true => Ok(()),
                false => Err(TransactionValidationError::ValidationError),
            },
            Err(stack) => Err(TransactionValidationError::OpenSSLError(stack)),
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn deserialize(json: String) -> Result<Transaction, TransactionDeserializeError> {
        let tx: Transaction = match serde_json::from_str(&json) {
            Ok(tx) => tx,
            Err(e) => return Err(TransactionDeserializeError::SerdeError(e)),
        };

        if tx.transaction_info.value < 1 {
            return Err(TransactionDeserializeError::InsufficientFunds);
        }

        Ok(tx)
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SENDER::{:?}::RECEIVER::{:?}::{}::SIGNATURE::{:?}",
            self.sender,
            self.receiver.to_pem(),
            self.transaction_info,
            self.signature
        )
    }
}
