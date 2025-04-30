use super::wallet::Wallet;
use chrono::{DateTime, Utc};
use core::panic;
use openssl::error::ErrorStack;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
// ---------------------------------------------- TransactionInfo definition ----------------------------------------
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
// -----------------------------------------------------------------------------------------------------------------

// --------------------------------------- Transaction Serialization Utils -----------------------------------------

// TODO move this to a errors file
#[derive(Debug)]
pub enum TransactionDeserializeError {
    InvalidSignature,
    InsufficientFunds,
    MalformedTransaction,
    OpenSSLError(ErrorStack),
    SerdeError(serde_json::Error),
}
impl fmt::Display for TransactionDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionDeserializeError::InvalidSignature => write!(f, "Invalid signature"),
            TransactionDeserializeError::InsufficientFunds => write!(f, "Insufficient funds"),
            TransactionDeserializeError::MalformedTransaction => write!(f, "Malformed transaction"),
            TransactionDeserializeError::OpenSSLError(e) => write!(f, "OpenSSL Error"),
            TransactionDeserializeError::SerdeError(value) => write!(f, "{}", value),
        }
    }
}
impl std::error::Error for TransactionDeserializeError {}

/*
mod signature_def {
    use rsa::BigUint;
    use serde::{Deserialize, Serialize};
    use serde_bytes;
    use smallvec::SmallVec;

    type BigDigit = u64;
    const VEC_SIZE: usize = 4;

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "BigUint")]
    struct BigUintDef {
        data: SmallVec<[BigDigit; VEC_SIZE]>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "rsa::pkcs1v15::Signature")]
    pub struct SignatureDef {
        inner: BigUint,
        len: usize,
    }
} */

// -----------------------------------------------------------------------------------------------------------------

// ---------------------------------------------- Transaction definition -------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Wallet,
    pub receiver: Wallet,
    // #[serde(with = "signature_def::SignatureDef")]
    pub signature: Vec<u8>,
    pub transaction_info: TransactionInfo,
}

impl Transaction {
    pub fn new(
        sender: Wallet,
        receiver: Wallet,
        transaction_info: TransactionInfo,
        signature: Vec<u8>,
    ) -> Result<Self, ErrorStack> {
        let verify_signature = match sender.verify_transaction_info(&transaction_info, &signature) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        if verify_signature {
            Ok(Self {
                sender,
                receiver,
                signature,
                transaction_info,
            })
        } else {
            panic!("Signature couldn't be verified");
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
        println!("serialized transaction: {value}");
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
        let verified = match tx
            .sender
            .verify_transaction_info(&tx.transaction_info, &tx.signature)
        {
            Ok(value) => value,
            Err(e) => return Err(TransactionDeserializeError::OpenSSLError(e)),
        };
        if !verified {
            return Err(TransactionDeserializeError::InvalidSignature);
        }
        // TODO check if wallets exist
        Ok(tx)
    }
}

// ---------------------------------------------- UNIT TESTS -------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::chain::transaction::TransactionInfo;
    use chrono::Utc;

    #[test] //mark a function as a test.
    fn test_transactioninfo_creation() {
        let transaction: TransactionInfo = TransactionInfo::new(12345 as f32, Utc::now());
        println!("transaction info:\n{}", transaction.to_string());
        println!("{:?}", transaction);
    }
}
// -----------------------------------------------------------------------------------------------------------------
