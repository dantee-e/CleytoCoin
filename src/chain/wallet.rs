use crate::chain::ordered_vector::OrderedVec;
use crate::chain::utxo::UTXO;

use super::transaction::TransactionInfo;
use openssl::error::ErrorStack;

pub use super::wallet_pk::WalletPK;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::rsa::Rsa;
use openssl::sign::Verifier;
use serde::de;
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
    serializer.serialize_bytes(&processed)
}

fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PKey<Public>, D::Error>
where
    D: de::Deserializer<'de>,
{
    // struct StringVisitor;
    // impl<'de> de::Visitor<'de> for StringVisitor {
    //     type Value = PKey<Public>;
    //
    //     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    //         formatter.write_str("The PEM string as a vector of u8")
    //     }
    //
    //     fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    //     where
    //         E: Error,
    //     {
    //         PKey::public_key_from_pem(v).map_err(E::custom)
    //     }
    //     fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    //     where
    //         E: de::Error,
    //     {
    //         PKey::public_key_from_pem(v.as_bytes()).map_err(E::custom)
    //     }
    // }
    let bytes = Vec::<u8>::deserialize(deserializer).expect("Could not deserialize PEM");
    Ok(PKey::public_key_from_pem(&bytes).expect("Could not deserialize PEM"))
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
const MAX_ITERATIONS_NON_EXACT_COIN_SELECTION: u64 = 10_000;

// Helper for the coin selection algorithms
#[derive(Clone)]
struct UtxoEstimate {
    utxo: UTXO,
    effective_value: u64,
    weight: u64,
}
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

        // 2️⃣  Find the smallest UTXO that already exceeds the target.
        let first_over_idx = utxos
            .clone()
            .into_iter()
            .position(|u| u.value() < amount)
            .unwrap_or(0);

        // 3️⃣  Branch‑and‑bound selection on the remaining (smaller) UTXOs.

        let dust_threshold = Self::estimate_fee_per_utxo(&utxos[0]) * 3;
        let smaller_utxos = &utxos.get_slice(first_over_idx..utxos.len());

        let mut total_sum = 0;
        let estimates: Vec<UtxoEstimate> = smaller_utxos
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
        let (bnb_solution, _) = self.branch_and_bound(
            &estimates,
            amount,
            dust_threshold,
            100_000, // max repetitions
            total_sum,
        );

        if !bnb_solution.is_empty() {
            return Ok(bnb_solution);
        } else {
            println!("bnb solution is empty");
        }

        // 4️⃣  Pick the “best” solution (the one with the lowest waste that still
        //     satisfies the target). If none exists we fall back to the exact‑match
        //     error (already handled at the top).

        // If we found a single “big enough” UTXO, keep it as a candidate solution.
        let mut candidate_solutions = Vec::new();
        if first_over_idx > 0 {
            if let Some(utxo) = utxos.get(first_over_idx - 1) {
                candidate_solutions.push(vec![utxo.clone()]);
            }
        }

        let target = amount + dust_threshold;
        let solution = Self::dantes_crazy_algorithm_entrypoint(smaller_utxos, target);

        if solution.is_empty() {
            return Err(WalletError::InsufficientFunds);
        }
        Ok(solution)
    }

    // Helper: compute “waste” (extra fee paid beyond the long‑term rate).
    fn waste(estimates: &[UtxoEstimate], fee_rate: u64) -> u64 {
        let total_weight: u64 = estimates.iter().map(|e| e.weight).sum();
        // println!(
        //     "waste is {}, total_weight is {}, fee rate is {}",
        //     total_weight * (fee_rate - LONG_TERM_FEE_RATE),
        //     total_weight,
        //     fee_rate
        // );
        total_weight * (fee_rate - LONG_TERM_FEE_RATE)
    }
    /// Core branch‑and‑bound algorithm – returns a tuple `(selected_utxos, total_value)`.
    fn branch_and_bound(
        &self,
        estimates: &[UtxoEstimate],
        target: u64,
        dust_threshold: u64,
        max_reps: usize,
        total_sum: u64,
    ) -> (Vec<UTXO>, u64) {
        // Transform each UTXO into an enriched struct that carries an estimated
        // “effective value” (value minus fee) and its weight.

        // for est in estimates.clone() {
        //     println!(
        //         "UTXOEstimate (effective_value: {}, weight: {} )",
        //         est.utxo.value() - Self::estimate_fee_per_utxo(&est.utxo),
        //         UTXO_WEIGHT
        //     );
        // }

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
            let incl_waste = Wallet::waste(&incl_set, fee_rate);

            if incl_sum >= target_interval.0
                && incl_sum <= target_interval.1
                && incl_waste < *best_waste
            {
                *best_sum = incl_sum;
                *best_set = incl_set.clone();
                *best_waste = incl_waste;
                // Found a feasible solution - we can stop exploring this branch.
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

            // ---------- Omit the head ----------
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
            estimates,
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
        println!("The result of the calculate_recursion_depth func is {new_depth}");
        std::cmp::max(new_depth, 1)
    }

    fn dantes_crazy_algorithm_entrypoint(slice: &[UTXO], target: u64) -> Vec<UTXO> {
        let mut solution: Vec<UtxoEstimate> = Vec::new();
        let mut solution_waste = u64::MAX;

        let slice_estimates: Vec<UtxoEstimate> = slice
            .iter()
            .map(|utxo| UtxoEstimate {
                utxo: utxo.clone(),
                effective_value: utxo.value() - Wallet::estimate_fee_per_utxo(utxo),
                weight: UTXO_WEIGHT,
            })
            .collect();

        Self::dantes_crazy_coin_selection_algorithm(
            &slice_estimates,
            slice_estimates.len() / 2,
            &mut solution,
            &mut solution_waste,
            target,
            -1,
            2, // TODO get the real fee_rate
            1,
        );

        solution.into_iter().map(|estimate| estimate.utxo).collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn dantes_crazy_coin_selection_algorithm(
        slice: &[UtxoEstimate],
        middle_index: usize,
        solution: &mut Vec<UtxoEstimate>,
        solution_waste: &mut u64,
        target: u64,
        mut number_of_iterations: i32, // initialize with -1
        fee_rate: u64,
        position: u64,
    ) {
        if number_of_iterations == -1 {
            println!("number_of_iterations is {number_of_iterations}");
        }
        if position == MAX_ITERATIONS_NON_EXACT_COIN_SELECTION
            || number_of_iterations == 0
            || slice.is_empty()
        {
            return;
        }
        let mut sum = 0;
        let mut elements: Vec<UtxoEstimate> = match slice.get(middle_index) {
            Some(_) => vec![slice[middle_index].clone()],
            None => return,
        };
        sum += slice[middle_index].utxo.value();

        if number_of_iterations == -1 {
            println!("number_of_iterations is still {number_of_iterations}");
        }
        for k in 2..slice.len() + 1 {
            // Gets the middle_index + or - k/2 depending on whether k is even or odd. Also checks
            // if the resulting index is smaller than 0, and if so skips the iteration
            let index = match k % 2 {
                1 => {
                    let x = middle_index as i32 - (k as i32 / 2);
                    if x < 0 {
                        continue;
                    } else {
                        x as usize
                    }
                }
                0 => middle_index + k / 2,
                _ => unreachable!(),
            };

            if let Some(x) = slice.get(index) {
                sum += x.utxo.value();
                elements.push(x.clone());
                // println!(
                //     "Pushing {} to sum {} on position {position} with target {target}",
                //     slice[index].utxo.value(),
                //     sum
                // );
            }
            if sum > target {
                // if there's no x yet, we calculate it here. The x is a function of the number
                // of elements necessary in the first iteration. If many elements are needed in the
                // first iteration, that means that if I continue for too many times there will be a
                // lot of overlap. Therefore, we reduce the size of x
                if number_of_iterations == -1 {
                    number_of_iterations =
                        Self::calculate_recursion_depth(MAX_UTXO_SEARCH_DEPTH, k, slice.len());
                    println!("Caculated the recursion depth to be {number_of_iterations}");
                }
                let new_waste = Self::waste(&elements, fee_rate);
                if new_waste < *solution_waste {
                    *solution = elements.clone();
                    *solution_waste = new_waste;
                }

                break;
            }
        }

        Self::dantes_crazy_coin_selection_algorithm(
            slice,
            middle_index / 2,
            solution,
            solution_waste,
            target,
            number_of_iterations - 1,
            fee_rate,
            position * 2,
        );
        Self::dantes_crazy_coin_selection_algorithm(
            slice,
            middle_index * 3 / 2,
            solution,
            solution_waste,
            target,
            number_of_iterations - 1,
            fee_rate,
            position * 2 + 1,
        );
    }
}

// -----------------------------------------------------------------------------------------------------------------
