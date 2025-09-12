use crate::chain::ordered_vector::OrderedVec;
use crate::chain::utxo::UTXO;

use super::transaction::TransactionInfo;
use openssl::error::ErrorStack;

pub use super::wallet_pk::WalletPK;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::rsa::Rsa;
use openssl::sign::Verifier;
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

// ------------------------------------------------- Serde stuff ---------------------------------------------------
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
// -----------------------------------------------------------------------------------------------------------------

// ------------------------------------------------- Traits stuff --------------------------------------------------
impl PartialEq for Wallet {
    fn eq(&self, other: &Self) -> bool {
        self.to_pem() == other.to_pem()
    }
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
// -----------------------------------------------------------------------------------------------------------------

// ---------------------------------------------- Wallet definition ------------------------------------------------
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(
        serialize_with = "serialize_public_key",
        deserialize_with = "deserialize_public_key"
    )]
    pub(crate) public_key: PKey<Public>,
    pub(crate) available_utxos: Option<OrderedVec<UTXO>>,
}

const MAX_UTXO_SEARCH_DEPTH: usize = 100;
const UTXO_WEIGHT: u64 = 64; // typical weight of a P2PKH output - replace with real weight
const LONG_TERM_FEE_RATE: u64 = 1; // placeholder - replace with real rate

impl Wallet {
    /* --------------------------------------------------------------------- *
     *                         Construction & Utilities                     *
     * --------------------------------------------------------------------- */

    /// Creates a fresh key pair and returns `(wallet, private_key_wrapper)`.
    pub fn new() -> (Self, WalletPK) {
        // 2048‑bit RSA key – adjust size if you need stronger security.
        let rsa = Rsa::generate(2048).expect("RSA generation failed");
        let private_key = PKey::from_rsa(rsa).expect("Invalid RSA key");

        // Export the public part as PEM and immediately parse it back.
        let public_key = PKey::public_key_from_pem(
            &private_key
                .public_key_to_pem()
                .expect("Could not extract public key from private key"),
        )
        .expect("Failed to parse PEM public key");

        (
            Wallet {
                public_key,
                available_utxos: None,
            },
            WalletPK { private_key },
        )
    }

    /// Verify a signed `TransactionInfo` using the stored public key.
    pub fn verify_transaction_info(
        &self,
        transaction_info: &TransactionInfo,
        signature: &[u8],
    ) -> Result<bool, ErrorStack> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.public_key)?;
        verifier.update(transaction_info.to_string().as_bytes())?;
        verifier.verify(signature)
    }

    /// Export the public key as PEM bytes.
    pub fn to_pem(&self) -> Vec<u8> {
        self.public_key
            .public_key_to_pem()
            .expect("PEM conversion failed")
    }

    /// Rough fee estimate per UTXO – replace with a real estimator later.
    fn estimate_fee_per_utxo(_utxo: &UTXO) -> u64 {
        100
    }

    /// Insert a batch of new UTXOs, keeping the internal ordering intact.
    pub fn add_utxos(&mut self, new_vec: Vec<UTXO>) {
        match &mut self.available_utxos {
            Some(ord_vec) => {
                for utxo in new_vec {
                    ord_vec.insert(utxo);
                }
            }
            None => self.available_utxos = Some(OrderedVec::from(new_vec)),
        }
    }

    /* --------------------------------------------------------------------- *
     *                         Coin‑Selection Logic                         *
     * --------------------------------------------------------------------- */
    /// Public entry point – selects a set of UTXOs whose summed value covers
    /// `amount`. Returns an error if the wallet does not contain enough funds.
    pub fn get_utxos(&self, amount: u64) -> Result<Vec<UTXO>, WalletError> {
        let utxos = self
            .available_utxos
            .as_ref()
            .ok_or(WalletError::InsufficientFunds)?
            .clone();

        // Quick check – if the total balance is insufficient we can bail early.
        if UTXO::sum(&utxos) < amount {
            return Err(WalletError::InsufficientFunds);
        }

        // 1️⃣  Simple exact‑match shortcut.
        if let Some(single) = utxos.clone().into_iter().find(|u| u.value() == amount) {
            return Ok(vec![single.clone()]);
        }

        for utxo in utxos.clone() {
            println!("UTXO of value {}", utxo.value());
        }
        // 2️⃣  Find the smallest UTXO that already exceeds the target.
        let first_over_idx = utxos
            .clone()
            .into_iter()
            .position(|u| u.value() < amount)
            .unwrap_or(0);

        println!("first_over_idx is {first_over_idx}");

        // If we found a single “big enough” UTXO, keep it as a candidate solution.
        let mut candidate_solutions = Vec::new();
        if let Some(utxo) = utxos.get(first_over_idx - 1) {
            candidate_solutions.push(vec![utxo.clone()]);
        }

        // 3️⃣  Branch‑and‑bound selection on the remaining (smaller) UTXOs.

        let dust_threshold = Self::estimate_fee_per_utxo(&utxos[0]) * 3;
        let smaller_utxos = &utxos.get_slice(first_over_idx..utxos.len());
        println!("len of smaller_utxos is {}", smaller_utxos.len());

        println!("The amount is {amount} and the dust_threshold is {dust_threshold}");
        let (bnb_solution, _) = self.branch_and_bound(
            smaller_utxos,
            amount,
            dust_threshold,
            100_000, // max repetitions
        );
        println!("Finished bnb");

        if !bnb_solution.is_empty() {
            return Ok(bnb_solution);
        } else {
            println!("bnb solution is empty");
            return Err(WalletError::InsufficientFunds);
        }

        #[allow(unreachable_code)]
        // 4️⃣  Pick the “best” solution (the one with the highest sum that still
        //     satisfies the target). If none exists we fall back to the exact‑match
        //     error (already handled at the top).
        let target = amount + dust_threshold;
        Self::dantes_crazy_coin_selection_algorithm(
            smaller_utxos,
            MAX_UTXO_SEARCH_DEPTH,
            &mut candidate_solutions,
            target,
            -1,
        );
        let best = candidate_solutions.into_iter().max_by_key(UTXO::sum);

        best.ok_or(WalletError::InsufficientFunds)
    }

    /// Core branch‑and‑bound algorithm – returns a tuple `(selected_utxos, total_value)`.
    fn branch_and_bound(
        &self,
        slice: &[UTXO],
        target: u64,
        dust_threshold: u64,
        max_reps: usize,
    ) -> (Vec<UTXO>, u64) {
        println!("Starting branch_and_bound");
        // Transform each UTXO into an enriched struct that carries an estimated
        // “effective value” (value minus fee) and its weight.
        #[derive(Clone)]
        struct UtxoEstimate {
            utxo: UTXO,
            effective_value: u64,
            weight: u64,
        }

        let mut total_sum = 0u64;
        let estimates: Vec<UtxoEstimate> = slice
            .iter()
            .map(|u| {
                total_sum += u.value();
                UtxoEstimate {
                    utxo: u.clone(),
                    effective_value: u.value() - Self::estimate_fee_per_utxo(u),
                    weight: UTXO_WEIGHT,
                }
            })
            .collect();

        for est in estimates.clone() {
            println!(
                "UTXOEstimate (effective_value: {}, weight: {} )",
                est.utxo.value() - Self::estimate_fee_per_utxo(&est.utxo),
                UTXO_WEIGHT
            );
        }

        // Helper: compute “waste” (extra fee paid beyond the long‑term rate).
        fn waste(estimates: &[UtxoEstimate], fee_rate: u64) -> u64 {
            let total_weight: u64 = estimates.iter().map(|e| e.weight).sum();
            // println!(
            //     total_weight * (fee_rate - LONG_TERM_FEE_RATE),
            //     total_weight,
            //     fee_rate
            // );
            total_weight * (fee_rate - LONG_TERM_FEE_RATE)
        }

        // Recursive branch‑and‑bound search.
        #[allow(clippy::too_many_arguments)]
        fn recurse(
            remaining: &[UtxoEstimate],
            cur_sum: u64,
            cur_set: Vec<UtxoEstimate>,
            sum_left: u64,
            best_set: &mut Vec<UtxoEstimate>,
            best_sum: &mut u64,
            best_waste: &mut u64,
            target_interval: (u64, u64),
            fee_rate: u64,
            reps_left: usize,
        ) {
            if reps_left == 0 || remaining.is_empty() {
                return;
            }

            // Update the amount left after discarding the current head’s fee.
            let sum_left = sum_left - remaining[0].effective_value;

            // ---------- Include the head ----------
            let mut incl_set = cur_set.clone();
            incl_set.push(remaining[0].clone());
            let incl_sum = cur_sum + remaining[0].effective_value;
            let incl_waste = waste(&incl_set, fee_rate);

            if incl_sum >= target_interval.0
                && incl_sum <= target_interval.1
                && incl_waste < *best_waste
            {
                *best_sum = incl_sum;
                *best_set = incl_set.clone();
                *best_waste = incl_waste;
                // Found a feasible solution – we can stop exploring this branch.
                return;
            }

            if incl_sum < target_interval.1
                && sum_left as i64 > target_interval.0 as i64 - incl_sum as i64
            {
                recurse(
                    &remaining[1..],
                    incl_sum,
                    incl_set,
                    sum_left,
                    best_set,
                    best_sum,
                    best_waste,
                    target_interval,
                    fee_rate,
                    reps_left - 1,
                );
            }

            // ---------- Exclude the head ----------
            if sum_left as i64 > target_interval.0 as i64 - cur_sum as i64 {
                recurse(
                    &remaining[1..],
                    cur_sum,
                    cur_set,
                    sum_left,
                    best_set,
                    best_sum,
                    best_waste,
                    target_interval,
                    fee_rate,
                    reps_left - 1,
                );
            }
        }

        // Initialise the search.
        let mut best_set = Vec::new();
        let mut best_sum = u64::MAX;
        let mut best_waste = u64::MAX;
        recurse(
            &estimates,
            0,
            Vec::new(),
            total_sum,
            &mut best_set,
            &mut best_sum,
            &mut best_waste,
            (target, target + dust_threshold),
            2, // fee_rate placeholder – replace with real rate
            max_reps,
        );

        // Strip the auxiliary data and return plain UTXOs.
        let selected = best_set.into_iter().map(|e| e.utxo).collect::<Vec<_>>();
        let total_selected = UTXO::sum(&selected);
        (selected, total_selected)
    }

    fn calculate_recursion_depth(
        max_depth: usize,
        elements_tested: usize,
        total_elements: usize,
    ) -> i32 {
        if total_elements == 0 {
            return 0;
        }
        let fraction_used = elements_tested as f64 / total_elements as f64;
        let new_depth = (max_depth as f64 * (1.0 - fraction_used)).ceil() as i32;
        std::cmp::max(new_depth, 1)
    }
    fn dantes_crazy_coin_selection_algorithm(
        slice: &[UTXO],
        k: usize,
        solutions: &mut Vec<Vec<UTXO>>,
        target: u64,
        mut x: i32, // initialize with -1
    ) {
        if x == 0 {
            return;
        }
        let mut sum = 0;
        let mut elements: Vec<UTXO> = vec![slice[k].clone()];
        sum += slice[k].value();

        for i in 0..k {
            sum += slice[k - i].value();
            elements.push(slice[k - 1].clone());
            if sum > target {
                // if there's no x yet, we calculate it here. The x is a function of the number
                // of elements necessary in the first iteration. If many elements are needed in the
                // first iteration, that means that if I continue for too many times there will be a
                // lot of overlap. Therefore, we reduce the size of x
                if x == -1 {
                    x = Self::calculate_recursion_depth(MAX_UTXO_SEARCH_DEPTH, i, slice.len());
                }

                solutions.push(elements);
                break;
            }
        }
        Self::dantes_crazy_coin_selection_algorithm(slice, k / 2, solutions, target, x - 1);
        Self::dantes_crazy_coin_selection_algorithm(slice, k * 3 / 2, solutions, target, x - 1);
    }
}

// -----------------------------------------------------------------------------------------------------------------
