use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, SignerMut, Verifier};
use rsa::sha2::{Digest, Sha256};
use rand::rngs::ThreadRng;
use super::transaction::TransactionInfo;
use super::utils::HashedData;



pub struct WalletPK{
    private_key: RsaPrivateKey,
    signing_key: SigningKey<Sha256>,
    rng: ThreadRng
}

impl WalletPK {
    pub fn sign_transaction(&mut self, transaction_info: &TransactionInfo) -> Result<rsa::pkcs1v15::Signature, rsa::Error>{

        
        let signed_hashed_message = self.signing_key.sign_with_rng(&mut self.rng, transaction_info.to_string().as_bytes());


        Ok(signed_hashed_message)
    }
}


pub struct Wallet{
    rng: ThreadRng,
    public_key: RsaPublicKey,
    verifying_key: VerifyingKey<Sha256>
}

impl Wallet {
    pub fn new() -> (Self, WalletPK) {
        let mut rng = rand::thread_rng(); // rand@0.8
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);
        let signing_key = SigningKey::<Sha256>::new(private_key.clone());
        let verifying_key = signing_key.verifying_key();


        
        (Wallet{public_key, verifying_key, rng: rng.clone()}, WalletPK{private_key, signing_key, rng})
    }

    


    pub fn verify_transaction_info(&self, data: &TransactionInfo, signature: &rsa::pkcs1v15::Signature) -> bool {
        let verified = self.verifying_key.verify(data.to_string().as_bytes(), &signature);
        match verified {
            Ok(()) => {println!("Deu bom");true},
            Err(_) => {println!("Deu ruim");false},
        }
    }

    #[allow(unused)]
    pub fn get_public_key(&self) -> RsaPublicKey {
        self.public_key.clone()
    }

    #[allow(unused)]
    pub fn encrypt(&mut self, msg: &[u8]) -> Option<Vec<u8>> {
        match self.public_key.encrypt(&mut self.rng, Pkcs1v15Encrypt, msg) {
            Ok(value) => Some(value),
            Err(e) => {
                println!("Unable to encrypt the message using the wallet's public_key");
                println!("Error: {}", e);
                None
            },
        }
    }
}