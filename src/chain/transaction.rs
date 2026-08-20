use super::utxo::UTXO;
use super::wallet::Wallet;
use crate::chain::ordered_vector::OrderedVec;
use crate::chain::utxo::OutPoint;
use crate::chain::utxo::TransactionInput;
use crate::chain::utxo::TransactionOutput;
use crate::error_handling::CleytoResult;
use crate::error_handling::TransactionDeserializeError;
use crate::error_handling::TransactionError;
use chrono::{DateTime, Utc};
use openssl::sha::Sha256;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fmt::Display;

// ---------------------------------------------- TransactionInfo definition -----------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub date: DateTime<Utc>,
    pub txid: [u8; 32],
}

impl Transaction {
    /// Use this function to sign the transaction output + the individual input utxo
    pub fn get_ordered_bytes_for_signing(
        &self,
        input: &TransactionInput,
    ) -> Vec<u8> {
        let mut bytes = input.to_string().into_bytes();

        let mut bytes2: Vec<u8> = OrderedVec::from(self.outputs.clone())
            .into_iter()
            .map(|output| output.to_string().into_bytes())
            .flatten()
            .collect();

        bytes.append(&mut bytes2);
        bytes
    }

    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> CleytoResult<Self> {
        let input_sum = TransactionInput::sum(&inputs);
        let output_sum = UTXO::sum(&outputs);

        let change: i64 = input_sum as i64 - output_sum as i64;

        if change < 0 {
            return Err(TransactionError::InsufficientInputs)?;
        }

        let mut transaction = Self {
            inputs,
            outputs,
            txid: [0; 32],
            date: Utc::now(),
        };

        let to_hash = transaction.to_string();
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash.as_bytes());
        transaction.txid = hasher.finish().to_owned();

        match transaction.verify_signatures() {
            Ok(()) => Ok(transaction),
            Err(error) => Err(error)?,
        }
    }

    pub fn to_header(&self) -> TransactionHeader {
        TransactionHeader { txid: self.txid }
    }

    pub(crate) fn verify_signatures(&self) -> CleytoResult<()> {
        for input in &self.inputs {
            if !input.owner.verify_transaction_info(self)? {
                return Err(TransactionError::ValidationError)?;
            }
        }

        Ok(())
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn check_sufficient_funds(
        tx: &Transaction,
    ) -> Result<(), TransactionDeserializeError> {
        let input_sum = UTXO::sum(&tx.inputs);
        println!("Input sum is {input_sum}");
        let output_sum = UTXO::sum(&tx.outputs);
        println!("Output sum is {output_sum}");
        let change = input_sum as i64 - output_sum as i64;

        if change < 0 {
            return Err(TransactionDeserializeError::InsufficientFunds);
        }

        Ok(())
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.txid == other.txid
    }
}

impl Default for Transaction {
    fn default() -> Self {
        let (sender, sender_pk) = Wallet::new();
        let (receiver, _) = Wallet::new();

        let value: u64 = rand::random();

        let input = TransactionInput::new(
            value,
            sender.clone(),
            OutPoint::new(Some([0; 32]), 0),
        );
        let output = TransactionOutput::new(
            value,
            receiver.clone(),
            OutPoint::new(Some([1; 32]), 0),
        );

        let mut transaction_info =
            Transaction::new(vec![input], vec![output]).unwrap();

        sender_pk.sign_all_owned_inputs_in_transaction(&mut transaction_info);

        transaction_info
    }
}

impl Display for Transaction {
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

        write!(
            f,
            "TXID::{:?}::DATE::{}::INPUTS::{}:OUTPUTS::{}",
            self.txid, self.date, inputs, outputs
        )
    }
}

// -------------------------------------------------------------------------------------------------

// ------------------------------------- Transaction definition ------------------------------------

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionHeader {
    txid: [u8; 32],
}
