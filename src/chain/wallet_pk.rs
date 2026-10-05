use super::transaction::Transaction;
use crate::chain::ordered_vector::OrderedVec;
use crate::chain::utxo::{OutPoint, UTXO};
use crate::chain::wallet::Wallet;
use crate::configs::ConfigPaths;
use crate::error_handling::CleytoResult;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::sign::Signer;
use openssl::symm::Cipher;
use std::collections::HashSet;
// ---------------------------------------------- WalletPK definition ----------------------------------------------
#[derive(Debug)]
pub struct WalletPK {
    pub(crate) private_key: PKey<Private>,
}

impl WalletPK {
    /// Signs, in place, every input of the transaction that spends one of `coins` owned by this
    /// key. `coins` are the UTXOs this wallet selected (or any list containing them); inputs of
    /// other owners are left untouched, so several owners can sign the same transaction in any
    /// order. Returns how many inputs were signed.
    ///
    /// Every input signs the same message (the whole unsigned transaction), so it is signed once.
    pub fn sign_all_owned_inputs_in_transaction(
        &self,
        transaction: &mut Transaction,
        coins: &[UTXO],
    ) -> Result<usize, ErrorStack> {
        let me = self.public_wallet().to_public_key();
        let owned: HashSet<OutPoint> = coins
            .iter()
            .filter(|coin| coin.output.owner == me)
            .map(|coin| coin.outpoint)
            .collect();

        let mut signer =
            Signer::new(MessageDigest::sha256(), &self.private_key)?;
        let signature = signer
            .sign_oneshot_to_vec(&transaction.get_ordered_bytes_for_signing())?;

        let mut signed = 0;
        for input in transaction
            .inputs
            .iter_mut()
            .filter(|input| owned.contains(&input.prev))
        {
            input.signature = Some(signature.clone());
            signed += 1;
        }

        Ok(signed)
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
            available_utxos: OrderedVec::new(),
        }
    }
}
impl From<PKey<Private>> for WalletPK {
    fn from(private_key: PKey<Private>) -> Self {
        Self { private_key }
    }
}

// -----------------------------------------------------------------------------------------------------------------
