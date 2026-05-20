use crate::error_handling::TransactionDeserializeError;
use crate::error_handling::TransactionError;

use super::utxo::UTXO;
use super::wallet::Wallet;
use chrono::{DateTime, Utc};
use openssl::sha::Sha256;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
// ---------------------------------------------- TransactionInfo definition -----------------------
pub struct TransactionInfo {
    pub inputs: Vec<UTXO>,
    pub outputs: Vec<UTXO>,
    pub date: DateTime<Utc>,
}

impl TransactionInfo {
    pub fn new(inputs: Vec<UTXO>, outputs: Vec<UTXO>) -> TransactionInfo {
        let date = Utc::now();
        Self {
            inputs,
            outputs,
            date,
        }
    }
}

impl Display for TransactionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inputs: String = self
            .inputs
            .clone()
            .into_iter()
            .map(|input| input.to_string())
            .collect::<Vec<String>>()
            .join("::");

        let outputs: String = self
            .outputs
            .clone()
            .into_iter()
            .map(|output| output.to_string())
            .collect::<Vec<String>>()
            .join("::");

        write!(f, "INPUTS::{}:OUTPUTS::{}", inputs, outputs)
    }
}
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

// TODO eventually, I want to make the transactions not need to have the sender adress
impl Transaction {
    pub fn new(
        sender: Wallet,
        receiver: Wallet,
        transaction_info: TransactionInfo,
        signature: Vec<u8>,
    ) -> Result<Self, TransactionError> {
        let mut transaction = Self {
            sender,
            receiver,
            signature,
            transaction_info,
            txid: [0; 32], // This could be optimized by avoiding the creation of this Vec, which
                           // serves no function on its own, but I don't really see that being a problem
        };

        let input_sum = UTXO::sum(&transaction.transaction_info.inputs);
        let output_sum = UTXO::sum(&transaction.transaction_info.outputs);
        let change: i64 = input_sum as i64 - output_sum as i64;

        if change < 0 {
            return Err(TransactionError::InsufficientInputs);
        }

        let to_hash = transaction.to_string();
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash.as_bytes());
        transaction.txid = hasher.finish().to_owned();

        match transaction.verify() {
            Ok(()) => Ok(transaction),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn verify(&self) -> Result<(), TransactionError> {
        match self
            .sender
            .verify_transaction_info(&self.transaction_info, &self.signature)
        {
            Ok(value) => match value {
                true => Ok(()),
                false => Err(TransactionError::ValidationError),
            },
            Err(stack) => Err(TransactionError::OpenSSLError(stack)),
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

        let input_sum = UTXO::sum(&tx.transaction_info.inputs);
        let output_sum = UTXO::sum(&tx.transaction_info.outputs);
        let change = input_sum - output_sum;

        if change < 1 {
            return Err(TransactionDeserializeError::InsufficientFunds);
        }

        Ok(tx)
    }
}

impl Default for Transaction {
    fn default() -> Self {
        let (sender, sender_pk) = Wallet::new();
        let (receiver, _) = Wallet::new();

        let value: u32 = rand::random();

        let transaction_info = TransactionInfo::new(
            vec![UTXO::new(value as u64, sender.clone())],
            vec![UTXO::new(value as u64, receiver.clone())],
        );

        let signature = sender_pk.sign_transaction(&transaction_info).unwrap();
        Transaction::new(sender, receiver, transaction_info, signature).unwrap()
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
