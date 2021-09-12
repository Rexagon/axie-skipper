use std::convert::TryInto;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignRequest {
    pub account_id: u16,
    pub message: String,
}

#[derive(Serialize)]
struct SignResponse {
    pub owner: String,
    pub message: String,
    pub signature: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let super_secret = std::env::var("SECRET")?;

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Content-Type",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
        ])
        .allow_methods(vec!["POST", "OPTIONS"]);

    let server = warp::post()
        .and(warp::path("sign"))
        .and(warp::body::json())
        .map(move |request| match post_sign(&super_secret, request) {
            Ok(res) => {
                warp::reply::with_status(serde_json::to_string(&res).unwrap(), http::StatusCode::OK)
            }
            Err(e) => warp::reply::with_status(e.to_string(), http::StatusCode::BAD_REQUEST),
        });

    warp::serve(server.with(cors))
        .run(([0, 0, 0, 0], 3030))
        .await;

    Ok(())
}

fn post_sign(secret: &str, req: SignRequest) -> Result<SignResponse> {
    let secp256k1 = secp256k1::Secp256k1::new();

    let path = format!("m/44'/60'/0'/0/{}", req.account_id);
    let secret_key = secp256k1::SecretKey::from_slice(&derive_secret_from_phrase(secret, &path)?)?;
    let public_key = secp256k1::PublicKey::from_secret_key(&secp256k1, &secret_key);

    // sign data
    // let data_hash = keccak256(req.message.as_bytes());
    let mut eth_data: Vec<u8> =
        format!("\x19Ethereum Signed Message:\n{}", req.message.len()).into_bytes();
    eth_data.extend_from_slice(&req.message.as_bytes());

    let hash = keccak256(&eth_data);
    let message = secp256k1::Message::from_slice(&hash).expect("Shouldn't fail");

    let (id, signature) = secp256k1
        .sign_recoverable(&message, &secret_key)
        .serialize_compact();

    let mut ex_sign = [0u8; 65];
    ex_sign[..64].copy_from_slice(&signature);
    ex_sign[64] = id.to_i32() as u8 + 27;

    // compute address
    let address = compute_eth_address(&public_key);

    Ok(SignResponse {
        owner: format!("0x{}", hex::encode(&address)),
        message: req.message,
        signature: format!("0x{}", hex::encode(&ex_sign)),
    })
}

pub fn compute_eth_address(public_key: &secp256k1::PublicKey) -> [u8; 20] {
    let pub_key = &public_key.serialize_uncompressed()[1..];
    keccak256(pub_key)[32 - 20..].try_into().unwrap()
}

pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

fn derive_secret_from_phrase(phrase: &str, path: &str) -> Result<[u8; 32]> {
    let mnemonic = bip39::Mnemonic::from_phrase(phrase, bip39::Language::English)?;
    let hd = bip39::Seed::new(&mnemonic, "");
    let seed_bytes = hd.as_bytes();

    let derived = tiny_hderive::bip32::ExtendedPrivKey::derive(seed_bytes, path)
        .map_err(|_| anyhow!("Failed to derive key"))?;
    Ok(derived.secret())
}
