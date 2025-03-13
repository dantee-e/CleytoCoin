use rsa::RsaPrivateKey;
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, Verifier};
use rsa::sha2::{Digest, Sha256};


fn main(){
    let mut rng = rand::thread_rng(); // rand@0.8

    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let verifying_key = signing_key.verifying_key();

    // Sign
    let data = b"hello world";
    let signature = signing_key.sign_with_rng(&mut rng, data);
    assert_ne!(signature.to_bytes().as_ref(), data.as_slice());

    // Verify
    verifying_key.verify(data, &signature).expect("failed to verify");
}

