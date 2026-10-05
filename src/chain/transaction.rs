use super::wallet::Wallet;
use crate::chain::utxo::OutPoint;
use crate::chain::utxo::TransactionInput;
use crate::chain::utxo::TransactionOutput;
use crate::chain::utxo::UtxoSet;
use crate::chain::utxo::UTXO;
use crate::error_handling::CleytoResult;
use crate::error_handling::TransactionError;
use chrono::{DateTime, Utc};
use openssl::sha::Sha256;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;

// ---------------------------------------------- Transaction definition ---------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u8,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    /// Only set on coinbase transactions. Makes two coinbases paying the same amount to the
    /// same key have different txids (otherwise the second would overwrite the first's outputs)
    pub coinbase_height: Option<u64>,
    /// Informative only, not part of the txid
    pub date: DateTime<Utc>,
}

impl Transaction {
    pub const VERSION: u8 = 1;

    /// Bytes that are hashed into the txid and signed by every input owner. Signatures are left
    /// out, so the txid (and therefore the outpoints of the outputs) is known before signing and
    /// adding signatures can't change it.
    pub fn get_ordered_bytes_for_signing(&self) -> Vec<u8> {
        fn push_field(bytes: &mut Vec<u8>, field: &[u8]) {
            bytes.extend((field.len() as u64).to_be_bytes());
            bytes.extend(field);
        }

        let mut bytes = Vec::new();
        push_field(&mut bytes, b"CLEYTOCOIN_TX_V1");
        bytes.push(self.version);

        match self.coinbase_height {
            Some(height) => {
                bytes.push(1);
                bytes.extend(height.to_be_bytes());
            }
            None => bytes.push(0),
        }

        bytes.extend((self.inputs.len() as u64).to_be_bytes());
        for input in &self.inputs {
            bytes.extend(input.prev.txid); // fixed 32 bytes
            bytes.extend(input.prev.vout.to_be_bytes());
        }

        bytes.extend((self.outputs.len() as u64).to_be_bytes());
        for output in &self.outputs {
            bytes.extend(output.value.to_be_bytes());
            push_field(&mut bytes, &output.owner.0);
        }

        bytes
    }

    /// Creates an unsigned transaction spending `inputs`. Whether the inputs exist and cover the
    /// outputs can only be checked against the UTXO set, see `verify`.
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> CleytoResult<Self> {
        if inputs.is_empty() {
            Err(TransactionError::NoInputs)?
        }

        let mut seen = HashSet::new();
        if !inputs.iter().all(|input| seen.insert(input.prev)) {
            Err(TransactionError::DuplicateInput)?
        }

        Ok(Self {
            version: Self::VERSION,
            inputs,
            outputs,
            coinbase_height: None,
            date: Utc::now(),
        })
    }

    /// Transaction that creates coins out of nothing. Only valid as the first transaction of a
    /// block, `height` being that block's index.
    pub fn coinbase(height: u64, outputs: Vec<TransactionOutput>) -> Self {
        Self {
            version: Self::VERSION,
            inputs: Vec::new(),
            outputs,
            coinbase_height: Some(height),
            date: Utc::now(),
        }
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty() && self.coinbase_height.is_some()
    }

    pub fn txid(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.get_ordered_bytes_for_signing());
        hasher.finish()
    }

    /// The outputs of this transaction with their outpoints
    pub fn new_utxos(&self) -> Vec<UTXO> {
        let txid = self.txid();
        self.outputs
            .iter()
            .enumerate()
            .map(|(vout, output)| {
                UTXO::new(
                    OutPoint {
                        txid,
                        vout: vout as u32,
                    },
                    output.clone(),
                )
            })
            .collect()
    }

    pub fn to_header(&self) -> TransactionHeader {
        TransactionHeader { txid: self.txid() }
    }

    /// Validates a non coinbase transaction against the UTXO set: every input must spend an
    /// existing unspent output, at most once, signed by that output's owner, and the inputs must
    /// cover the outputs. Returns the fee (inputs - outputs).
    pub fn verify(&self, utxos: &UtxoSet) -> CleytoResult<u64> {
        if self.inputs.is_empty() {
            Err(TransactionError::NoInputs)?
        }

        let message = self.get_ordered_bytes_for_signing();
        let mut seen = HashSet::new();
        let mut input_sum: u64 = 0;

        for input in &self.inputs {
            if !seen.insert(input.prev) {
                Err(TransactionError::DuplicateInput)?
            }

            let spent = utxos
                .get(&input.prev)
                .ok_or(TransactionError::UtxoNotFound)?;

            let signature = input
                .signature
                .as_ref()
                .ok_or(TransactionError::UnsignedInput)?;

            if !spent.owner.verify(&message, signature)? {
                Err(TransactionError::ValidationError)?
            }

            input_sum = input_sum
                .checked_add(spent.value)
                .ok_or(TransactionError::ValueOverflow)?;
        }

        let output_sum = TransactionOutput::checked_sum(&self.outputs)
            .ok_or(TransactionError::ValueOverflow)?;

        Ok(input_sum
            .checked_sub(output_sum)
            .ok_or(TransactionError::InsufficientInputs)?)
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.txid() == other.txid()
    }
}

impl Default for Transaction {
    /// A signed transaction spending a made up coin. Structurally valid, but it won't pass
    /// `verify` since the coin isn't in any UTXO set.
    fn default() -> Self {
        let (sender, sender_pk) = Wallet::new();
        let (receiver, _) = Wallet::new();

        let value = rand::random::<u32>() as u64;

        let coin = UTXO::new(
            OutPoint {
                txid: rand::random(),
                vout: 0,
            },
            TransactionOutput::new(value, &sender),
        );

        let mut transaction = Transaction::new(
            TransactionInput::from_utxos(std::slice::from_ref(&coin)),
            vec![TransactionOutput::new(value, &receiver)],
        )
        .unwrap();

        sender_pk
            .sign_all_owned_inputs_in_transaction(&mut transaction, &[coin])
            .unwrap();

        transaction
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inputs: String = self
            .inputs
            .iter()
            .map(|input| input.to_string())
            .collect::<Vec<String>>()
            .join("::");

        let outputs: String = self
            .outputs
            .iter()
            .map(|output| output.to_string())
            .collect::<Vec<String>>()
            .join("::");

        write!(
            f,
            "TXID::{}::DATE::{}::INPUTS::{}:OUTPUTS::{}",
            hex::encode(self.txid()),
            self.date,
            inputs,
            outputs
        )
    }
}

// -------------------------------------------------------------------------------------------------

// ------------------------------------- TransactionHeader definition ------------------------------

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionHeader {
    txid: [u8; 32],
}
