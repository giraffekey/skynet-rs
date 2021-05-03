use crate::{SkynetClient, SkynetResult, SkynetError::*, util::make_uri};
use std::{collections::HashMap, str};
use crypto::{
  blake2b::Blake2b,
  digest::Digest,
  ed25519,
};
use hex::{FromHex, ToHex};
use hyper::{body, Body, Request};
use serde::Deserialize;
use serde_json::json;

const DEFAULT_GET_ENTRY_TIMEOUT: u32 = 5;

#[derive(Debug)]
pub struct RegistryEntry {
  pub data_key: String,
  pub data: Vec<u8>,
  pub revision: u64,
}

#[derive(Debug)]
pub struct SignedRegistryEntry {
  pub entry: RegistryEntry,
  pub signature: Vec<u8>,
}

#[derive(Debug)]
pub struct EntryOptions {
  pub endpoint_path: String,
  pub api_key: Option<String>,
  pub custom_user_agent: Option<String>,
  pub hashed_data_key_hex: bool,
}

impl Default for EntryOptions {
  fn default() -> Self {
    Self {
      endpoint_path: "/skynet/registry".into(),
      api_key: None,
      custom_user_agent: None,
      hashed_data_key_hex: false,
    }
  }
}

fn hash_data_key(data_key: &str, hashed_data_key_hex: bool) -> String {
  if hashed_data_key_hex {
    data_key.into()
  } else {
    let mut hash = [0; 32];
    let mut hasher = Blake2b::new(32);
    Digest::input(&mut hasher, data_key.as_bytes());
    Digest::result(&mut hasher, &mut hash);
    hash.encode_hex()
  }
}

fn hash_registry_entry(entry: &RegistryEntry, hashed_data_key_hex: bool) -> Vec<u8> {
  let mut hash = [0; 32];
  let mut hasher = Blake2b::new(32);
  Digest::input(&mut hasher, hash_data_key(&entry.data_key, hashed_data_key_hex).as_bytes());
  Digest::input(&mut hasher, &entry.data);
  Digest::input(&mut hasher, entry.revision.to_string().as_bytes());
  Digest::result(&mut hasher, &mut hash);
  hash.to_vec()
}

#[derive(Deserialize)]
struct GetResponse {
  data: String,
  revision: u64,
  signature: String,
}

pub async fn get_registry_entry(
  client: &SkynetClient,
  public_key: &[u8],
  data_key: &str,
  opt: EntryOptions,
) -> SkynetResult<SignedRegistryEntry> {
  let req = Request::builder().method("GET");
  let mut query = HashMap::new();
  
  query.insert("publickey".into(), format!("ed25519:{}", public_key.encode_hex::<String>()));
  query.insert("datakey".into(), hash_data_key(data_key, opt.hashed_data_key_hex));
  query.insert("timeout".into(), DEFAULT_GET_ENTRY_TIMEOUT.to_string());

  let uri = make_uri(
    client.get_portal_url(),
    opt.endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let req = req.body(Body::from("")).map_err(HttpError)?;
  let res = client.http.request(req).await.map_err(HyperError)?;
  let body = body::to_bytes(res.into_body()).await.map_err(HyperError)?;
  let body_str = str::from_utf8(&body).map_err(Utf8Error)?;
  let res: GetResponse = serde_json::from_str(body_str)
    .map_err(|_| PortalResponse(body_str.to_string()))?;

  let entry = SignedRegistryEntry {
  	entry: RegistryEntry {
      data_key: data_key.into(),
      data: FromHex::from_hex(res.data).unwrap(),
      revision: res.revision,
    },
    signature: FromHex::from_hex(res.signature).unwrap(),
  };

  let hash = hash_registry_entry(&entry.entry, opt.hashed_data_key_hex);
  if !ed25519::verify(&hash, public_key, &entry.signature) {
  	return Err(InvalidSignature);
  }

  Ok(entry)
}

pub async fn set_registry_entry(
  client: &SkynetClient,
  public_key: &[u8],
  private_key: &[u8],
  entry: RegistryEntry,
  opt: EntryOptions,
) -> SkynetResult<()> {
  let req = Request::builder().method("POST");
  let query = HashMap::new();
  
  let uri = make_uri(
    client.get_portal_url(),
    opt.endpoint_path,
    opt.api_key,
    None,
    query);

  let mut req = req.uri(uri);

  if let Some(custom_user_agent) = opt.custom_user_agent {
    req = req.header("User-Agent", custom_user_agent);
  }

  let hash = hash_registry_entry(&entry, opt.hashed_data_key_hex);
  let signature = ed25519::signature(&hash, private_key);

  let data = json!({
    "publickey": {
      "algorithm": "ed25519",
      "key": public_key,
    },
    "datakey": hash_data_key(&entry.data_key, opt.hashed_data_key_hex),
    "revision": entry.revision,
    "data": entry.data,
    "signature": signature.to_vec(),
  }).to_string();

  let req = req.body(Body::from(data)).map_err(HttpError)?;
  client.http.request(req).await.map_err(HyperError)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::gen_keypair_and_seed;

  #[tokio::test]
  async fn test_registry_entry() {
	  let (keypair, _) = gen_keypair_and_seed(64);
    let client = SkynetClient::default();
    let res = set_registry_entry(
      &client,
      &keypair.public_key,
      &keypair.private_key,
      RegistryEntry {
        data_key: "data".into(),
        data: b"hello world".to_vec(),
        revision: 0,
      },
      EntryOptions::default(),
    ).await;
    println!("{:?}", res);
    assert!(res.is_ok());
    let res = get_registry_entry(
      &client,
      &keypair.public_key,
      "data",
      EntryOptions::default(),
    ).await;
    println!("{:?}", res);
    assert!(res.is_ok());
    let entry = res.unwrap().entry;
    assert_eq!(entry.data_key, "data".to_string());
    assert_eq!(entry.data, b"hello world".to_vec());
    assert_eq!(entry.revision, 0);
  }
}
