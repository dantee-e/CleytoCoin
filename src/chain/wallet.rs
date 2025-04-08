use openssl::error::ErrorStack;
use serde::{Deserialize, Serialize};
use super::transaction::TransactionInfo;

use openssl::sign::{Signer, Verifier};
use openssl::rsa::Rsa;
use openssl::pkey::{PKey, Private, Public};
use openssl::hash::MessageDigest;

fn test_sign() {
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap();

    // Data to be signed (this would normally be provided by the signer)
    let data = b"hello, world!";

    // Step 2: Sign the data with the private key (this would be done by the sender)
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey).unwrap();
    signer.update(data).unwrap();
    let signature = signer.sign_to_vec().unwrap();

    // --- Now, we are at the verification step ---
    // Step 3: Extract the public key from the PKey and use it for verification
    let public_key = pkey.public_key_to_pem().unwrap();  // Extract public key in PEM format
    let rsa_public = Rsa::public_key_from_pem(&public_key).unwrap(); // Convert back to Rsa
    let pkey_public = PKey::from_rsa(rsa_public).unwrap();  // Create a PKey for public key

    // Step 4: Verify the signature using the public key
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey_public).unwrap();
    verifier.update(data).unwrap();
    let is_valid = verifier.verify(&signature).unwrap();

    // Step 5: Check if the signature is valid
    if is_valid {
        println!("Signature is valid!");
    } else {
        println!("Signature is invalid.");
    }
}

// ---------------------------------------------- WalletPK definition ----------------------------------------------
#[derive(Debug)]
pub struct WalletPK{
    private_key: PKey<Private>
}

impl WalletPK {
    pub fn sign_transaction(&mut self, transaction_info: &TransactionInfo) -> Result<Vec<u8>, ErrorStack>{
        let mut signer = Signer::new(MessageDigest::sha256(), &self.private_key)?;
        signer.update(transaction_info.to_string().as_bytes())?;
        Ok(signer.sign_to_vec()?)
    }
}
// -----------------------------------------------------------------------------------------------------------------

// ---------------------------------------------- Wallet definition ------------------------------------------------
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet{
    public_key: Vec<u8>
}

impl Wallet {
    pub fn new() -> (Self, WalletPK) {
        let bits: u32 = 2048;
        let rsa = Rsa::generate(bits).unwrap();
        let private_key = PKey::from_rsa(rsa).unwrap();

        let public_key = private_key.public_key_to_pem().expect("Error extracting public key from private key");

        (
            Wallet{
                public_key
            },
            WalletPK{
                private_key
            }
        )
    }

    pub fn verify_transaction_info(&self, transaction_info: &TransactionInfo, signature: &[u8]) -> Result<bool, ErrorStack> {

        let public_key = self.to_pkey();
        let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)?;
        verifier.update(transaction_info.to_string().as_bytes())?;
        Ok(verifier.verify(&signature)?)
    }

    #[allow(unused)]
    pub fn to_pkey(&self) -> PKey<Public> {
        let rsa_public = Rsa::public_key_from_pem(&self.public_key).expect("Error extracting Rsa<Public> object from public key");
        PKey::from_rsa(rsa_public).expect("Error extracting PKey<Public> object from Rsa<Public> object")
    }

    pub fn to_vec(&self) -> Vec<u8> {
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
        println!("wallet.to_string: {:?}", wallet.to_vec());
        println!("{:#?}", wallet);
        println!("{:#?}", wallet_pk);
    }
}
// -----------------------------------------------------------------------------------------------------------------
