use crypto::{
  blake2b::Blake2b,
  digest::Digest,
  ed25519,
  hmac::Hmac,
  pbkdf2::pbkdf2,
  sha2::Sha256,
};
use rand::prelude::*;

#[derive(Debug)]
pub struct KeyPair {
  pub public_key: [u8; 32],
  pub private_key: [u8; 64],
}

fn make_seed(length: u32) -> Vec<u8> {
  let mut rng = rand::thread_rng();
  let mut seed: Vec<u8> = Vec::new();
  for _ in 0..length {
  	seed.push(rng.gen());
  }
  seed
}

pub fn gen_keypair_and_seed(length: u32) -> (KeyPair, Vec<u8>) {
  let seed = make_seed(length);
  (gen_keypair_from_seed(&seed), seed)
}

pub fn gen_keypair_from_seed(seed: &[u8]) -> KeyPair {
  let mut mac = Hmac::new(Sha256::new(), seed);
  let mut derived_key = [0; 32];
  pbkdf2(&mut mac, &[], 1000, &mut derived_key);

  let (private_key, public_key) = ed25519::keypair(&derived_key);

  KeyPair {
  	public_key,
  	private_key,
  }
}

pub fn derive_child_seed(master: &[u8], seed: &[u8]) -> Vec<u8> {
  let mut child_seed = vec![0; master.len()];
  let mut hasher = Blake2b::new(master.len());
  Digest::input(&mut hasher, master);
  Digest::input(&mut hasher, seed);
  Digest::result(&mut hasher, &mut child_seed);
  child_seed
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_keypair() {
    let (keypair, seed) = gen_keypair_and_seed(64);
    let keypair2 = gen_keypair_from_seed(&seed);
    assert_eq!(keypair.public_key, keypair2.public_key);
    assert_eq!(keypair.private_key, keypair2.private_key);
    let child_seed = derive_child_seed(&seed, b"foo");
    assert_eq!(child_seed.len(), seed.len());
  }
}
