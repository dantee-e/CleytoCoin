use super::transaction::TransactionInfo;
use super::wallet::Wallet;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::sign::Signer;
use openssl::symm::Cipher;
// ---------------------------------------------- WalletPK definition ----------------------------------------------
#[derive(Debug)]
pub struct WalletPK {
    pub(crate) private_key: PKey<Private>,
}

impl WalletPK {
    pub fn sign_transaction(
        &self,
        transaction_info: &TransactionInfo,
    ) -> Result<Vec<u8>, ErrorStack> {
        let mut signer = Signer::new(MessageDigest::sha256(), &self.private_key)?;
        signer.sign_oneshot_to_vec(transaction_info.to_string().as_bytes())
    }
    pub fn to_pem_with_password(&self, password: &String) -> Vec<u8> {
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

        Wallet {
            public_key,
            available_utxos: None,
        }
    }
}
impl From<PKey<Private>> for WalletPK {
    fn from(private_key: PKey<Private>) -> Self {
        Self { private_key }
    }
}

// -----------------------------------------------------------------------------------------------------------------
