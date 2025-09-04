use crate::chain::ordered_vector::OrderedVec;
use crate::chain::utxo::UTXO;

use super::transaction::TransactionInfo;
use openssl::error::ErrorStack;

use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use openssl::symm::Cipher;
use serde::de::{self, Error};
use serde::{Deserialize, Serialize};

// ------------------------------------------- Wallet errors definition --------------------------------------------
#[derive(Debug)]
pub enum WalletError {
    InsufficientFunds,
}
impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Insufficient funds on the wallet to conduct operation")
    }
}
impl std::error::Error for WalletError {}
// -----------------------------------------------------------------------------------------------------------------

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

// ---------------------------------------------- Wallet definition ------------------------------------------------
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(
        serialize_with = "serialize_public_key",
        deserialize_with = "deserialize_public_key"
    )]
    public_key: PKey<Public>, // Should I store this as PEM or as PKey<Public>?
    available_utxos: Option<OrderedVec<UTXO>>,
}

impl PartialEq for Wallet {
    fn eq(&self, other: &Self) -> bool {
        self.to_pem() == other.to_pem()
    }
}

fn serialize_public_key<S>(key: &PKey<Public>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let processed: Vec<u8> = key.public_key_to_pem().map_err(serde::ser::Error::custom)?;
    processed.serialize(serializer)
}

fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PKey<Public>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringVisitor;
    impl<'de> de::Visitor<'de> for StringVisitor {
        type Value = PKey<Public>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("The PEM string as a vector of u8")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            PKey::public_key_from_pem(v).map_err(E::custom)
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            PKey::public_key_from_pem(v.as_bytes()).map_err(E::custom)
        }
    }

    // use our visitor to deserialize an `ActualValue`
    deserializer.deserialize_any(StringVisitor)
}

impl From<String> for Wallet {
    fn from(value: String) -> Self {
        let public_rsa = openssl::rsa::Rsa::public_key_from_pem(value.as_bytes())
            .expect("Could not read the public key");
        let public_key =
            PKey::from_rsa(public_rsa).expect("Error converting from RSA to PKey<Public>");
        Self {
            public_key,
            available_utxos: None,
        }
    }
}
impl From<PKey<Public>> for Wallet {
    fn from(public_key: PKey<Public>) -> Self {
        Self {
            public_key,
            available_utxos: None,
        }
    }
}
const MAX_UTXO_SEARCH_DEPTH: usize = 100;
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

        (
            Wallet {
                public_key,
                available_utxos: None,
            },
            WalletPK { private_key },
        )
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

    fn estimate_fee_per_utxo() -> u64 {
        0
    }

    // TODO write some tests for this bad boy
    pub fn get_utxos(&self, amount: u64) -> Result<Vec<UTXO>, WalletError> {
        let available_utxos = self.available_utxos.clone().expect("Found no UTXOs");
        if UTXO::sum(&available_utxos) < amount {
            return Err(WalletError::InsufficientFunds);
        }
        let mut solutions: Vec<Vec<UTXO>> = Vec::new();

        // Gets the first UTXO (if there is such a value) that's is bigger than the requested
        // amount
        let index = match available_utxos
            .clone()
            .into_iter()
            .enumerate()
            .find(|(_, utxo)| utxo.value() > amount)
        {
            Some((index, value)) => {
                solutions.push(vec![value]);
                index
            }
            None => available_utxos.len(),
        };

        // https://bitcoin.stackexchange.com/questions/1077/what-is-the-coin-selection-algorithm
        // if the sum of all yout UTXO smaller than the target happens to match the target,they
        // will be used

        fn calculate_recursion_depth(
            max_depth: usize,
            elements_tested: usize,
            total_elements: usize,
        ) -> u32 {
            if total_elements == 0 {
                return 0;
            }
            let fraction_used = elements_tested as f64 / total_elements as f64;
            let new_depth = (max_depth as f64 * (1.0 - fraction_used)).ceil() as u32;
            std::cmp::max(new_depth, 1)
        }

        fn recursive(
            slice: &[UTXO],
            solutions: &mut Vec<Vec<UTXO>>,
            target: u64,
            x: &mut Option<u32>,
        ) {
            if slice.is_empty() {
                return;
            }
            if let Some(ref mut depth) = x {
                if *depth == 0 {
                    return;
                }
                *depth -= 1;
            }
            let mut sum = 0;
            let k = slice.len() / 2;
            let mut elements: Vec<UTXO> = vec![slice[k].clone()];
            sum += slice[k].value();

            for i in 0..k {
                sum += slice[k - i].value();
                elements.push(slice[k - i].clone());
                if sum > target {
                    // if there's no x yet, we calculate it here. The x is a function of the number
                    // of elements necessary in the first iteration. If many elements are needed in the
                    // first iteration, that means that if I continue for too many times there will be a
                    // lot of overlap. Therefore, we reduce the size of x
                    if x.is_none() {
                        *x = Some(calculate_recursion_depth(
                            MAX_UTXO_SEARCH_DEPTH,
                            i,
                            slice.len(),
                        ));
                    }

                    solutions.push(elements);
                    break;
                }
            }
            recursive(&slice[0..k], solutions, target, x);
            recursive(&slice[k..], solutions, target, x);
        }

        let dust_threshold = Self::estimate_fee_per_utxo() * 3;
        let target = amount + dust_threshold;
        let utxos_smaller_than_target = available_utxos.get_slice(0..index).to_vec();
        let sum: u64 = UTXO::sum(&utxos_smaller_than_target);

        if sum == amount {
            return Ok(available_utxos.get_slice(0..index).to_vec());
        }
        if sum > target {
            // I'll just use the shit algorithm I invented. Goes like this:
            // Inputs are target amount and the ordered vector V that contains all available UTXOs
            // target = target_amount + dust threshold
            // First, we get the smallest element bigger than the target on the vector with index i
            // smallest elements vector = s = [0..i]
            //
            // We select the middle element of s (index k) and sum it with the following element (k+1).
            // If it's still smaller than target, sum it with k-1.
            // If still smaller than target, sum with k+2, and then k-2, k+3, and so on, until we find a solution
            // and save it as an option in a solutions vector
            //
            // We do the same now, but starting the element in the middle of the slice s[k..],
            // and again with the element in the middle of s[0..k]. This creates kind of a binary search through the vector.
            //
            // We repeat the process recursively x times, x being arbitrarily defined
            recursive(
                &utxos_smaller_than_target,
                &mut solutions,
                target,
                &mut None,
            );
        }

        let best_solution = solutions.into_iter().fold(None, |acc, new_vec| match acc {
            None => Some((UTXO::sum(&new_vec), new_vec)),
            Some((old_sum, old_vec)) => {
                let new_sum = UTXO::sum(&new_vec);
                if new_sum > old_sum {
                    Some((new_sum, new_vec))
                } else {
                    Some((old_sum, old_vec))
                }
            }
        });

        match best_solution {
            Some((_, vec)) => Ok(vec),
            // This is probably safe for an unwrap but fuck it
            None => Err(WalletError::InsufficientFunds),
        }
    }
}

// -----------------------------------------------------------------------------------------------------------------
