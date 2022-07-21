use {
    anyhow::anyhow,
    serde_json::Value,
    sha2::{Digest, Sha256},
    std::str::from_utf8,
};

use crate::constants::{
    HASH_PREFIX, MAINNET_URL, NAME_RECORD_HEADER_LEN, ROOT_DOMAIN_ACCOUNT, SPL_NAME_SERVICE_ID,
    URL_RECORD_NAME_HASHED,
};

/// Fetch and deseriealize the URL value stored in the SNS domain names data
pub async fn get_name_url(sns_name: &str) -> anyhow::Result<String> {
    let mut name_hasher = Sha256::new();
    name_hasher.update(HASH_PREFIX.to_owned() + sns_name);
    let hashed_name = name_hasher.finalize();

    let name_account_key = find_name_key(&hashed_name, &ROOT_DOMAIN_ACCOUNT);
    let url_name_record_key = find_name_key(&URL_RECORD_NAME_HASHED, &name_account_key);

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
        bs58::encode(url_name_record_key).into_string()
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

    let result = from_utf8(decoded_url_bytes)?.to_string();

    Ok(result)
}

pub fn find_name_key(hashed_name: &[u8], parent_key: &[u8]) -> [u8; 32] {
    const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

    let mut seeds_vec: Vec<&[u8]> = vec![hashed_name];
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
