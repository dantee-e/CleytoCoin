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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TransactionInput {
    pub signature: Option<Vec<u8>>,
    pub utxo: UTXO,
    pub inpoint: usize,
}

impl TransactionInput {
    pub fn new(utxo: UTXO, inpoint: usize) -> Self {
        Self {
            utxo,
            signature: None,
            inpoint,
        }
    }

    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|input| input.utxo.value).sum()
    }
}

impl Display for TransactionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner_pem = self.utxo.owner.to_pem();
        let owner = if let Ok(val) = String::from_utf8(owner_pem) {
            val
        } else {
            panic!("Invalid UTF-8 when getting UTXO owner")
        };
        write!(f, "VALUE::{}::OWNER::{}", self.utxo.value, owner)
    }
}

// impl PartialEq for TransactionInput {
//     fn eq(&self, other: &Self) -> bool {
//         self.value == other.value && self.owner == other.owner
//     }
// }
impl PartialOrd for TransactionInput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for TransactionInput {}
impl Ord for TransactionInput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.utxo.value.cmp(&other.utxo.value)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if other.utxo.value < self.utxo.value {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if other.utxo.value < self.utxo.value {
            other
        } else {
            self
        }
    }
}

//
/////
///
///
///
///
///
///
///
///
///
///
///
///
///
///

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TransactionOutput {
    pub value: u64,
    pub owner: Wallet,
}

impl TransactionOutput {
    pub fn new(value: u64, owner: Wallet) -> Self {
        Self { value, owner }
    }

    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|output| output.value).sum()
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

// ----------------------- TRANSACTION OUTPUT -----------------------
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UTXO {
    pub value: u64,
    pub owner: Wallet,
    pub outpoint: usize,
    pub txid: [u8; 32],
}

impl UTXO {
    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = Self>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.value).sum()
    }

    pub fn new(
        value: u64,
        owner: Wallet,
        outpoint: usize,
        txid: [u8; 32],
    ) -> Self {
        Self {
            value,
            owner,
            outpoint,
            txid,
        }
    }
}

impl Display for UTXO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner_pem = self.owner.to_pem();
        let owner = if let Ok(val) = String::from_utf8(owner_pem) {
            val
        } else {
            panic!("Invalid UTF-8 when getting UTXO owner")
        };
        write!(
            f,
            "VALUE::{}::OWNER::{}::OUTPOINT::{}::TXID::{}",
            self.value,
            owner,
            self.outpoint,
            String::from_utf8(self.txid.to_vec()).unwrap()
        )
    }
}
