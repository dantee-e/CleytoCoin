use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, Verifier};
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::sha2::Sha256;
use super::transaction::TransactionInfo;


// ---------------------------------------------- WalletPK definition ----------------------------------------------
#[derive(Debug)]
pub struct WalletPK{
    #[allow(unused)]
    private_key: RsaPrivateKey,
    signing_key: SigningKey<Sha256>
}

impl WalletPK {
    pub fn sign_transaction(&mut self, transaction_info: &TransactionInfo) -> Result<rsa::pkcs1v15::Signature, rsa::Error>{        
        let signed_hashed_message = self.signing_key.sign_with_rng(&mut rand::thread_rng(), transaction_info.to_string().as_bytes());

        Ok(signed_hashed_message)
    }
}
// -----------------------------------------------------------------------------------------------------------------

// ---------------------------------------------- Wallet definition ------------------------------------------------
#[derive(Clone)]
#[derive(Debug)]
pub struct Wallet{
    public_key: RsaPublicKey,
    verifying_key: VerifyingKey<Sha256>
}

impl Wallet {
    pub fn new() -> (Self, WalletPK) {
        let bits: usize = 2048;
        let private_key: RsaPrivateKey = RsaPrivateKey::new(&mut rand::thread_rng(), bits).expect("failed to generate a key");
        let public_key: RsaPublicKey = RsaPublicKey::from(&private_key);
        let signing_key: SigningKey<Sha256> = SigningKey::<Sha256>::new(private_key.clone());
        let verifying_key = signing_key.verifying_key();
        
        (
            Wallet{
                public_key, 
                verifying_key
            },
            WalletPK{
                private_key,
                signing_key, }
        )
    }

    pub fn verify_transaction_info(&self, data: &TransactionInfo, signature: &rsa::pkcs1v15::Signature) -> bool {
        let verified = self.verifying_key.verify(data.to_string().as_bytes(), &signature);
        match verified {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    pub fn to_string(&self) -> String {
        self.public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap().to_string()
    }

    #[allow(unused)]
    pub fn get_public_key(&self) -> RsaPublicKey {
        self.public_key.clone()
    }
}
// -----------------------------------------------------------------------------------------------------------------

// ---------------------------------------------- UNIT TESTS -------------------------------------------------------
#[cfg(test)] //ensures that the tests module is only included when running tests.
mod tests {
    use crate::chain::wallet::Wallet;

    #[test] //mark a function as a test.
    fn test_wallet_creation() {
        let (wallet, wallet_pk) = Wallet::new();
        println!("wallet.to_string: {}", wallet.to_string());
        println!("{:#?}", wallet);
        println!("{:#?}", wallet_pk);
    }
}
// -----------------------------------------------------------------------------------------------------------------
