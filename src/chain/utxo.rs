use std::{cmp::Ordering, fmt::Display};

use serde::{Deserialize, Serialize};
use sha2::digest::Output;

use super::wallet::Wallet;

pub trait UTXO:
    Display + for<'de> Deserialize<'de> + Serialize + Clone
{
    fn new(value: u64, owner: Wallet, outpoint: OutPoint) -> Self;

    fn value(&self) -> u64;

    fn owner(&self) -> Wallet;

    fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone;

    fn outpoint(&self) -> OutPoint;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub index: u32,
}

impl Display for OutPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TXID::{:?}::INDEX::{}", self.txid, self.index)
    }
}

// ----------------------- TRANSACTION INPUT -----------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionInput {
    signature: Vec<u8>,
    value: u64,
    owner: Wallet,
    outpoint: OutPoint,
}

impl UTXO for TransactionInput {
    fn new(value: u64, owner: Wallet, outpoint: OutPoint) -> Self {
        Self {
            value,
            owner,
            signature: Vec::new(),
            outpoint,
        }
    }
    fn value(&self) -> u64 {
        self.value
    }
    fn owner(&self) -> Wallet {
        self.owner.clone()
    }
    fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.value).sum()
    }

    fn outpoint(&self) -> OutPoint {
        self.outpoint.clone()
    }
}

impl Display for TransactionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner_pem = self.owner.to_pem();
        let owner = if let Ok(val) = String::from_utf8(owner_pem) {
            val
        } else {
            panic!("Invalid UTF-8 when getting UTXO owner")
        };
        write!(
            f,
            "VALUE::{}::OWNER::{}::SIGNATURE::{:?}",
            self.value, owner, self.signature
        )
    }
}

impl PartialEq for TransactionInput {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.owner == other.owner
    }
}
impl PartialOrd for TransactionInput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for TransactionInput {}
impl Ord for TransactionInput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if other.value < self.value {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if other.value < self.value {
            other
        } else {
            self
        }
    }
}

// ----------------------- TRANSACTION INPUT -----------------------

// ----------------------- TRANSACTION OUTPUT ----------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionOutput {
    value: u64,
    owner: Wallet,
    outpoint: OutPoint,
}

impl UTXO for TransactionOutput {
    fn new(value: u64, owner: Wallet, outpoint: OutPoint) -> Self {
        Self {
            value,
            owner,
            outpoint,
        }
    }
    fn value(&self) -> u64 {
        self.value
    }
    fn owner(&self) -> Wallet {
        self.owner.clone()
    }
    fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.value).sum()
    }
    fn outpoint(&self) -> OutPoint {
        self.outpoint.clone()
    }
}

impl Display for TransactionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner_pem = self.owner.to_pem();
        let owner = if let Ok(val) = String::from_utf8(owner_pem) {
            val
        } else {
            panic!("Invalid UTF-8 when getting UTXO owner")
        };
        write!(f, "VALUE::{}::OWNER::{}", self.value, owner)
    }
}

impl PartialEq for TransactionOutput {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.owner == other.owner
    }
}
impl PartialOrd for TransactionOutput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for TransactionOutput {}
impl Ord for TransactionOutput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if other.value < self.value {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if other.value < self.value {
            other
        } else {
            self
        }
    }
}

// If you want to use sum for a collection of UTXOs, use the func UTXO::sum()
