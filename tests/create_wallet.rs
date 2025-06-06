use cleyto_coin::chain::wallet::Wallet;

#[test] //mark a function as a test.
fn test_wallet_creation() {
    let (wallet, wallet_pk) = Wallet::new();
    println!("wallet.to_string: {:?}", wallet.to_vec());
    println!("{:#?}", wallet);
    println!("{:#?}", wallet_pk);
}
