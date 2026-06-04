use core::panic;
use std::{cmp::Ordering, fmt::Display};

use serde::{Deserialize, Serialize};

use super::wallet::Wallet;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UTXO {
    value: u64,
    owner: Wallet,
}
impl UTXO {
    pub fn new(value: u64, owner: Wallet) -> Self {
        Self { value, owner }
    }
    pub fn value(&self) -> u64 {
        self.value
    }
    pub fn owner(&self) -> Wallet {
        self.owner.clone()
    }
    pub fn sum<T>(vec: &T) -> u64
    where
        T: IntoIterator<Item = UTXO>,
        T: Clone,
    {
        vec.clone().into_iter().map(|utxo| utxo.value).sum()
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
        write!(f, "VALUE::{}::OWNER::{}", self.value, owner)
    }
}

impl PartialEq for UTXO {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.owner == other.owner
    }
}
impl PartialOrd for UTXO {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for UTXO {}
impl Ord for UTXO {
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
