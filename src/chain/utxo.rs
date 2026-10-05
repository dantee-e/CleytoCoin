use super::transaction::Transaction;
use super::wallet::Wallet;
use crate::error_handling::CleytoResult;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sha::sha256;
use openssl::sign::Verifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{cmp::Ordering, fmt::Display};

// ----------------------- PUBLIC KEY -----------------------

/// DER encoded public key. This is what owns an output, instead of a whole `Wallet`, so that
/// transactions serialize deterministically and don't carry anyone's UTXO list around.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct PublicKey(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl PublicKey {
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> CleytoResult<bool> {
        let pkey = PKey::public_key_from_der(&self.0)?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey)?;
        verifier.update(message)?;
        // A malformed signature is just an invalid one, not an internal error
        Ok(verifier.verify(signature).unwrap_or(false))
    }
}

impl From<&Wallet> for PublicKey {
    fn from(wallet: &Wallet) -> Self {
        wallet.to_public_key()
    }
}
impl From<Wallet> for PublicKey {
    fn from(wallet: Wallet) -> Self {
        wallet.to_public_key()
    }
}

impl Display for PublicKey {
    /// Short fingerprint of the key, the full DER is too long to be readable
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&sha256(&self.0)[..8]))
    }
}

// ----------------------- OUTPOINT -----------------------

/// Address of an output: "output number `vout` of the transaction `txid`".
/// It is the key of the UTXO set.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

impl Display for OutPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", hex::encode(self.txid), self.vout)
    }
}

// ----------------------- TRANSACTION INPUT -----------------------

/// A spend: points to the output being spent plus the proof that the owner allowed it.
/// The value and owner are NOT here on purpose, they must be looked up in the UTXO set.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct TransactionInput {
    pub prev: OutPoint,
    pub signature: Option<Vec<u8>>,
}

impl TransactionInput {
    pub fn new(prev: OutPoint) -> Self {
        Self {
            prev,
            signature: None,
        }
    }

    pub fn from_utxos(utxos: &[UTXO]) -> Vec<Self> {
        utxos.iter().map(|utxo| Self::new(utxo.outpoint)).collect()
    }
}

impl Display for TransactionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PREV::{}::SIGNED::{}",
            self.prev,
            self.signature.is_some()
        )
    }
}

// ----------------------- TRANSACTION OUTPUT -----------------------

/// What an output is. Its address is `(txid of the transaction, index in outputs)`.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct TransactionOutput {
    pub value: u64,
    pub owner: PublicKey,
}

impl TransactionOutput {
    pub fn new(value: u64, owner: impl Into<PublicKey>) -> Self {
        Self {
            value,
            owner: owner.into(),
        }
    }

    /// Returns None on overflow
    pub fn checked_sum<'a, T>(outputs: T) -> Option<u64>
    where
        T: IntoIterator<Item = &'a Self>,
    {
        outputs
            .into_iter()
            .try_fold(0u64, |acc, output| acc.checked_add(output.value))
    }
}

impl Display for TransactionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VALUE::{}::OWNER::{}", self.value, self.owner)
    }
}

// ----------------------- UTXO -----------------------

/// An unspent output together with its address. This is a view used by wallets (coin
/// selection, signing) and by the UTXO set's serialization; it never goes inside a transaction.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UTXO {
    pub outpoint: OutPoint,
    pub output: TransactionOutput,
}

impl UTXO {
    pub fn new(outpoint: OutPoint, output: TransactionOutput) -> Self {
        Self { outpoint, output }
    }

    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.output.value).sum()
    }
}

/// Ordered by value (used by coin selection). Ties are broken by outpoint and owner so that the
/// ordering agrees with `Eq`, otherwise `OrderedVec` would drop different coins of equal value.
impl Ord for UTXO {
    fn cmp(&self, other: &Self) -> Ordering {
        self.output
            .value
            .cmp(&other.output.value)
            .then_with(|| self.outpoint.cmp(&other.outpoint))
            .then_with(|| self.output.owner.cmp(&other.output.owner))
    }
}
impl PartialOrd for UTXO {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for UTXO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::OUTPOINT::{}", self.output, self.outpoint)
    }
}

// ----------------------- UTXO SET -----------------------

/// Every output that exists and has not been spent yet. This is the source of truth for
/// validating inputs. Serialized as a list because JSON map keys must be strings.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(from = "Vec<UTXO>", into = "Vec<UTXO>")]
pub struct UtxoSet(HashMap<OutPoint, TransactionOutput>);

impl UtxoSet {
    pub fn get(&self, outpoint: &OutPoint) -> Option<&TransactionOutput> {
        self.0.get(outpoint)
    }

    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.0.contains_key(outpoint)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn owned_by(&self, owner: &PublicKey) -> Vec<UTXO> {
        self.0
            .iter()
            .filter(|(_, output)| &output.owner == owner)
            .map(|(outpoint, output)| UTXO::new(*outpoint, output.clone()))
            .collect()
    }

    /// Removes the outputs spent by the transaction and adds the ones it creates.
    /// Does not validate anything, use `Transaction::verify` before.
    pub fn apply(&mut self, transaction: &Transaction) {
        for input in &transaction.inputs {
            self.0.remove(&input.prev);
        }
        for utxo in transaction.new_utxos() {
            self.0.insert(utxo.outpoint, utxo.output);
        }
    }
}

impl From<Vec<UTXO>> for UtxoSet {
    fn from(vec: Vec<UTXO>) -> Self {
        Self(
            vec.into_iter()
                .map(|utxo| (utxo.outpoint, utxo.output))
                .collect(),
        )
    }
}

impl From<UtxoSet> for Vec<UTXO> {
    fn from(set: UtxoSet) -> Self {
        set.0
            .into_iter()
            .map(|(outpoint, output)| UTXO::new(outpoint, output))
            .collect()
    }
}
