use super::wallet::Wallet;
use rsa::pkcs1v15::Signature;
use chrono::{DateTime, Utc};





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
        self.value.to_string()+ "::" + &self.date.to_string()
    }
}


pub struct Transaction {
    #[allow(unused)]
    pub sender: Wallet,
    #[allow(unused)]
    pub receiver: Wallet,
    #[allow(unused)]
    pub signature: Signature,
    #[allow(unused)]
    pub ammount: f32,
}

impl Transaction {
    pub fn new(sender: Wallet, receiver: Wallet, data: TransactionInfo, signature: Signature, ammount:f32) -> Self{
        
        let verify_signature = sender.verify_transaction_info(&data, &signature);
        
        if verify_signature {
            println!("Assinatura verificada com sucesso");
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