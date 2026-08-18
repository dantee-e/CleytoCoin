use crate::configs::ConfigPaths;
use crate::error_handling::CleytoResult;

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
        let mut signer =
            Signer::new(MessageDigest::sha256(), &self.private_key)?;
        signer.sign_oneshot_to_vec(transaction_info.to_string().as_bytes())
    }
    pub fn to_pem_with_password(&self, password: &str) -> Vec<u8> {
        self.private_key
            .private_key_to_pem_pkcs8_passphrase(
                Cipher::aes_256_cbc(),
                password.as_bytes(),
            )
            .unwrap()
    }
    /// Will save wallet to default location
    pub fn save_as(&self, name: &str, password: Option<&str>) {
        let pem_private = if let Some(pwd) = password {
            self.to_pem_with_password(pwd)
        } else {
            self.to_pem()
        };

        let pem_public = self.public_wallet().to_pem();

        // creates dir for keys
        std::fs::create_dir_all(format!(
            "{}/{name}",
            ConfigPaths::get().wallets_path
        ))
        .unwrap();

        std::fs::write(
            format!("{}/{name}/private.pem", ConfigPaths::get().wallets_path),
            pem_private,
        )
        .expect("Could not write private.pem");

        std::fs::write(
            format!("{}/{name}/public.pem", ConfigPaths::get().wallets_path),
            pem_public,
        )
        .expect("Could not write public.pem");
    }

    pub fn to_pem(&self) -> Vec<u8> {
        self.private_key.private_key_to_pem_pkcs8().unwrap()
    }

    /// Reads from default location
    pub fn from_pem(
        name: &String,
        password: Option<String>,
    ) -> CleytoResult<Option<Self>> {
        let path =
            format!("{}/{name}/private.pem", ConfigPaths::get().wallets_path);
        let content = match std::fs::read(path) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        };
        let pkey = if let Some(password) = password {
            PKey::private_key_from_pem_passphrase(&content, password.as_bytes())
        } else {
            PKey::private_key_from_pem(&content)
        }?;

        Ok(Some(pkey.into()))
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
