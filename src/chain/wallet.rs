use super::transaction::TransactionInfo;
use openssl::error::ErrorStack;

use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use openssl::symm::Cipher;
use serde::{Deserialize, Serialize};

// ---------------------------------------------- WalletPK definition ----------------------------------------------
#[derive(Debug)]
pub struct WalletPK {
    private_key: PKey<Private>,
}

impl WalletPK {
    pub fn sign_transaction(
        &self,
        transaction_info: &TransactionInfo,
    ) -> Result<Vec<u8>, ErrorStack> {
        let mut signer = Signer::new(MessageDigest::sha256(), &self.private_key)?;
        signer.sign_oneshot_to_vec(transaction_info.to_string().as_bytes())
    }
    pub fn to_pem_with_password(&self, password: String) -> Vec<u8> {
        self.private_key
            .private_key_to_pem_pkcs8_passphrase(Cipher::aes_256_cbc(), password.as_bytes())
            .unwrap()
    }
    pub fn to_pem(&self) -> Vec<u8> {
        self.private_key.private_key_to_pem_pkcs8().unwrap()
    }
    pub fn public_wallet(&self) -> Wallet {
        let public_key = PKey::public_key_from_pem(
            &self
                .private_key
                .public_key_to_pem()
                .expect("Could not extract Publick Key from Private Key"),
        )
        .unwrap();

        Wallet { public_key }
    }
}
impl From<PKey<Private>> for WalletPK {
    fn from(private_key: PKey<Private>) -> Self {
        Self { private_key }
    }
}

// -----------------------------------------------------------------------------------------------------------------

// ---------------------------------------------- Wallet definition ------------------------------------------------
#[derive(Clone, Debug)]
pub struct Wallet {
    public_key: PKey<Public>, // Should I store this as PEM or as PKey<Public>?
}

impl<'de> Deserialize<'de> for Wallet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        let public_key = PKey::public_key_from_pem(&bytes).map_err(serde::de::Error::custom)?;
        Ok(Wallet { public_key })
    }
}

impl Serialize for Wallet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let pem = self
            .public_key
            .public_key_to_pem()
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&pem)
    }
}

impl From<String> for Wallet {
    fn from(value: String) -> Self {
        let public_rsa = openssl::rsa::Rsa::public_key_from_pem(value.as_bytes())
            .expect("Could not read the public key");
        let public_key =
            PKey::from_rsa(public_rsa).expect("Error converting from RSA to PKey<Public>");
        Self { public_key }
    }
}
impl From<PKey<Public>> for Wallet {
    fn from(public_key: PKey<Public>) -> Self {
        Self { public_key }
    }
}
impl Wallet {
    pub fn new() -> (Self, WalletPK) {
        let bits: u32 = 2048;
        let rsa = Rsa::generate(bits).unwrap();
        let private_key = PKey::from_rsa(rsa).unwrap();

        let public_key = PKey::public_key_from_pem(
            &private_key
                .public_key_to_pem()
                .expect("Could not extract Publick Key from Private Key"),
        )
        .unwrap();

        (Wallet { public_key }, WalletPK { private_key })
    }

    pub fn verify_transaction_info(
        &self,
        transaction_info: &TransactionInfo,
        signature: &[u8],
    ) -> Result<bool, ErrorStack> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.public_key)?;
        verifier.update(transaction_info.to_string().as_bytes())?;
        verifier.verify(signature)
    }

    pub fn to_pem(&self) -> Vec<u8> {
        self.public_key.public_key_to_pem().unwrap()
    }
}
// -----------------------------------------------------------------------------------------------------------------
