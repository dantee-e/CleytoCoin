//use std::fmt;

use super::wallet::Wallet;
use rsa::pkcs1v15::Signature;
use chrono::{DateTime, Utc};




#[derive(Clone)]
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

    pub fn to_string(&self) -> String {
        format!(
            "VALUE::{}::TIME::{}",
            self.value.to_string(),
            self.date.to_string()
        )
    }

}

#[derive(Clone)]
pub struct Transaction {
    pub sender: Wallet,
    pub receiver: Wallet,
    pub signature: Signature,
    pub transaction_info: TransactionInfo,
}

impl Transaction {
    pub fn new(sender: Wallet, receiver: Wallet, transaction_info: TransactionInfo, signature: Signature) -> Self{
        
        let verify_signature = sender.verify_transaction_info(&transaction_info, &signature);
        
        if verify_signature {
            println!("Assinatura verificada com sucesso");
            Self{
                sender,
                receiver,
                signature,
                transaction_info
            }
        }
        /* verifying_key.; */

        else {
            panic!("Signature couldn't be verified");
        }
        
    }

    pub fn to_string(&self) -> String {
        format!(
            "SENDER::{}::RECEIVER::{}::{}::SIGNATURE::{}",
            self.sender.to_string(),
            self.receiver.to_string(),
            self.transaction_info.to_string(),
            self.signature.to_string()
        )
    }
}
