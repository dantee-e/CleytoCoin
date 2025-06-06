use super::wallet::Wallet;
use chrono::{DateTime, Utc};
use openssl::error::ErrorStack;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
// ---------------------------------------------- TransactionInfo definition -----------------------
pub struct TransactionInfo {
    pub value: f32,
    pub date: DateTime<Utc>,
}

impl TransactionInfo {
    pub fn new(value: f32, date: DateTime<Utc>) -> TransactionInfo {
        Self { value, date }
    }

    pub fn to_string(&self) -> String {
        format!(
            "VALUE::{}::TIME::{}",
            self.value.to_string(),
            self.date.to_string()
        )
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
}

impl Transaction {
    pub fn new(
        sender: Wallet,
        receiver: Wallet,
        transaction_info: TransactionInfo,
        signature: Vec<u8>,
    ) -> Result<Self, TransactionValidationError> {
        let transaction = Self {
            sender,
            receiver,
            signature,
            transaction_info,
        };

        match transaction.verify() {
            Ok(()) => Ok(transaction),
            Err(error) => return Err(error),
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

    pub fn to_string(&self) -> String {
        format!(
            "SENDER::{:?}::RECEIVER::{:?}::{}::SIGNATURE::{:?}",
            self.sender,
            self.receiver.to_vec(),
            self.transaction_info.to_string(),
            self.signature
        )
    }

    pub fn serialize(&self) -> String {
        let value = serde_json::to_string(self).unwrap();
        // println!("serialized transaction: {value}");
        value
    }
    pub fn deserialize(json: String) -> Result<Transaction, TransactionDeserializeError> {
        let tx: Transaction = match serde_json::from_str(&json) {
            Ok(tx) => tx,
            Err(e) => return Err(TransactionDeserializeError::SerdeError(e)),
        };

        if tx.transaction_info.value <= 0.0 {
            return Err(TransactionDeserializeError::InsufficientFunds);
        }

        Ok(tx)
    }
}
