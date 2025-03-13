use super::utils::{HashedData};
use super::wallet::Wallet;
use rsa::pkcs1v15::{Signature};
use chrono::{DateTime, Utc};
use sha2::Sha256;





pub struct TransactionInfo {
    value: f32,
    date: DateTime<Utc>
}

impl TransactionInfo {
    pub fn new(value: f32, date: DateTime<Utc>) -> TransactionInfo {
        Self {
            value,
            date
        }
    }
 
    pub fn to_hashed_data(&self) -> HashedData {
        let str = self.value.to_string() + &self.date.to_string();
        HashedData::from_string(&str)
    }
}


pub struct Transaction {
    pub sender: Wallet,
    pub receiver: Wallet,
    pub signature: Signature,
    pub ammount: f32,
}

impl Transaction {
    pub fn new(sender: Wallet, receiver: Wallet, data: TransactionInfo, signature: Signature, ammount:f32) -> Self{
        
        let verify_signature = sender.verify_signature(&data.to_hashed_data().get_hash(), signature.clone());
        
        if verify_signature {
            Self{
                sender,
                receiver,
                signature,
                ammount
            }
        }
        /* verifying_key.; */

        else {
            panic!("Signature couldn't be verified");
        }
        
    }
}