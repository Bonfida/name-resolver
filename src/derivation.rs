use {
    anyhow::anyhow,
    serde_json::Value,
    sha2::{Digest, Sha256},
    std::str::from_utf8,
    worker::Url,
};

use crate::constants::{
    HASH_PREFIX, MAINNET_URL, NAME_RECORD_HEADER_LEN, ROOT_DOMAIN_ACCOUNT, SPL_NAME_SERVICE_ID,
};

/// Fetch and deseriealize the URL value stored in the SNS domain names data
pub async fn get_name_url(sns_name: &str) -> anyhow::Result<Url> {
    let splitted: Vec<&str> = sns_name.split('.').collect();

    let record_key = if splitted.len() == 2 {
        let parent_key = find_name_key(&splitted[1], &ROOT_DOMAIN_ACCOUNT);
        find_name_key(&format!("\0{}", splitted[0]), &parent_key)
    } else {
        find_name_key(splitted[0], &ROOT_DOMAIN_ACCOUNT)
    };

    let request_data = format!(
        "
    {{
        \"jsonrpc\": \"2.0\",
        \"id\": 1,
        \"method\": \"getAccountInfo\",
        \"params\": [
          \"{}\",
          {{
            \"encoding\": \"base64\"
          }}
        ]
      }}",
        bs58::encode(record_key).into_string()
    );

    let request_json: Value = serde_json::from_str(&request_data)?;

    let client = reqwest::Client::new();
    let res = client.post(MAINNET_URL).json(&request_json).send().await?;

    if !res.status().is_success() {
        return Err(anyhow!("RPC request failed"));
    }

    let a = res.text().await?;

    let json_return: Value = serde_json::from_str(&a)?;

    let url_str = &json_return["result"]["value"]["data"][0]
        .as_str()
        .ok_or_else(|| anyhow!("Error deserializing account data"))?[NAME_RECORD_HEADER_LEN..]
        .trim_start_matches('A')
        .trim_end_matches('=')
        .trim_end_matches('A');
    let decoded_url_bytes = &base64::decode(url_str)?;

    let mut result = from_utf8(decoded_url_bytes)?.to_string();

    if result.starts_with("ipfs://") {
        let cid = &result[7..];
        result = format!("https://ipfs.infura.io/ipfs/{}", cid);
    }

    Url::parse(&result).map_err(|_| anyhow!("Error parsing URL"))
}

pub fn find_name_key(name: &str, parent_key: &[u8]) -> [u8; 32] {
    let mut name_hasher = Sha256::new();
    name_hasher.update(HASH_PREFIX.to_owned() + name);
    let hashed_name = name_hasher.finalize();

    const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

    let mut seeds_vec: Vec<&[u8]> = vec![&hashed_name];
    let def = [0u8; 32];
    seeds_vec.push(&def);
    seeds_vec.push(parent_key);

    let mut name_account_key = def;
    let mut bump_seed = [std::u8::MAX];
    for _ in 0..std::u8::MAX {
        {
            let mut seeds_with_bump = seeds_vec.clone();
            seeds_with_bump.push(&bump_seed);

            let mut key_hasher = Sha256::new();
            for seed in seeds_with_bump.iter() {
                key_hasher.update(seed);
            }
            key_hasher.update(&SPL_NAME_SERVICE_ID);
            key_hasher.update(PDA_MARKER);
            let hash = key_hasher.finalize();

            if curve25519_dalek::edwards::CompressedEdwardsY::from_slice(hash.as_ref())
                .decompress()
                .is_none()
            {
                name_account_key = *hash.as_ref();
                break;
            }
        }
        bump_seed[0] -= 1;
    }
    name_account_key
}
