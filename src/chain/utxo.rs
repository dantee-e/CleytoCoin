use super::wallet::Wallet;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

// pub trait UTXO:
//     Display
//     + for<'de> Deserialize<'de>
//     + Serialize
//     + Clone
//     + PartialEq
//     + Eq
//     + Ord
//     + PartialOrd
// {
//     fn new(value: u64, owner: Wallet) -> Self;
//
//     fn sum<T>(vec: &T) -> u64
//     where
//         T: IntoIterator<Item = Self>,
//         T: Clone;
// }

// ----------------------- TRANSACTION INPUT -----------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionInput {
    pub signature: Option<Vec<u8>>,
    pub value: u64,
    pub owner: Wallet,
    pub index: Option<u32>,
    pub txid: Option<[u8; 32]>,
}

impl TransactionInput {
    pub fn new(value: u64, owner: Wallet) -> Self {
        Self {
            value,
            owner,
            signature: None,
            index: None,
            txid: None,
        }
    }

    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.value).sum()
    }
}

impl From<TransactionOutput> for TransactionInput {
    fn from(value: TransactionOutput) -> Self {
        Self {
            value: value.value,
            owner: value.owner,
            index: value.index,
            txid: value.txid,
            signature: None,
        }
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
    pub value: u64,
    pub owner: Wallet,
    pub index: Option<u32>,
    pub txid: Option<[u8; 32]>,
}

impl TransactionOutput {
    pub fn new(value: u64, owner: Wallet) -> Self {
        Self {
            value,
            owner,
            index: None,
            txid: None,
        }
    }

    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.value).sum()
    }
}

impl Display for TransactionOutput {
    /// This does not serialize the txid
    /// This will panic if index is None
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner_pem = self.owner.to_pem();
        let owner = if let Ok(val) = String::from_utf8(owner_pem) {
            val
        } else {
            panic!("Invalid UTF-8 when getting UTXO owner")
        };

        write!(
            f,
            "VALUE::{}::OWNER::{}::INDEX::{}",
            self.value,
            owner,
            self.index.unwrap()
        )
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
