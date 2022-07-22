use {
    anyhow::anyhow,
    serde_json::Value,
    sha2::{Digest, Sha256},
    std::str::from_utf8,
    worker::Url,
};

use futures::join;

use crate::constants::{
    HASH_PREFIX, MAINNET_URL, NAME_RECORD_HEADER_LEN, RECORDS, RECORDS_LOWER_CASE,
    ROOT_DOMAIN_ACCOUNT, SPL_NAME_SERVICE_ID,
};

/// Fetch and deseriealize the URL value stored in the SNS domain names data
pub async fn get_name_url(sns_name: &str) -> anyhow::Result<Url> {
    let mut splitted_names: Vec<&str> = sns_name.split('.').collect();
    let mut first_name = splitted_names[0].to_owned();

    first_name.make_ascii_lowercase();

    // If a record was specified, get its correct name and strip it from the input
    let record = RECORDS_LOWER_CASE
        .iter()
        .position(|rec| rec == &first_name)
        .map(|idx| {
            splitted_names.remove(0);
            RECORDS[idx]
        });

    let parent_key = if splitted_names.len() == 2 {
        find_name_key(splitted_names[1], &ROOT_DOMAIN_ACCOUNT)
    } else {
        ROOT_DOMAIN_ACCOUNT
    };

    let prefix = if splitted_names.len() == 2 { "\0" } else { "" };

    let domain_key = find_name_key(&format!("{}{}", prefix, splitted_names[0]), &parent_key);

    let mut result = match record {
        None => {
            // No record specified, default to URL then IPFS, do it in parallel
            let url_record = find_name_key("\x01url", &domain_key);
            let ipfs_record = find_name_key("\x01IPFS", &domain_key);
            let res_tuple = join!(fetch_record(&url_record), fetch_record(&ipfs_record));
            let res = res_tuple.0.map_or(res_tuple.1, Ok);
            res?
        }
        Some(r) => {
            let record_key = find_name_key(&format!("\x01{}", r), &domain_key);
            fetch_record(&record_key).await?
        }
    };

    if result.starts_with("ipfs://") {
        let cid = &result[7..];
        result = format!("https://ipfs.infura.io/ipfs/{}", cid);
    }

    if result.starts_with("arwv://") {
        let arwv_hash = &result[7..];
        result = format!("https://arweave.net/{}", arwv_hash);
    }

    Url::parse(&result).map_err(|_| anyhow!("Error parsing URL"))
}

pub async fn fetch_record(record_key: &[u8; 32]) -> anyhow::Result<String> {
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

    let decoded = base64::decode(url_str)?;
    let result = from_utf8(&decoded)?.to_string();

    Ok(result)
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

#[tokio::test]
async fn test_resolve() {
    const INPUT_OUTPUT: [(&str, &str); 4] = [
        (
            "boston",
            "https://ipfs.infura.io/ipfs/QmZk9uh2mqmXJFKu2Hq7kFRh93pA8GDpSZ6ReNqubfRKKQ",
        ),
        (
            "ARWV.boston",
            "https://arweave.net/KuB5jmew87_M2flH9f6ZpB9jlDv8hZSHPrmGUY8KqEk",
        ),
        (
            "sub.boston",
            "https://ipfs.infura.io/ipfs/QmeHUsLEdoEzTVuRxHcYxx6mXDqs9RhEawCS3a3AQTFFeM",
        ),
        (
            "ARWV.sub.boston",
            "https://arweave.net/VE2zcstYZ9ptHWQcQBrb4gOe6j162c7NdO8xy4OcWiE",
        ),
    ];
    // let name = "sub.boston";
    // let res = get_name_url(name).await.unwrap();
    // println!("{:?}", res.as_str());

    for (input, output) in INPUT_OUTPUT {
        let res = get_name_url(input).await.unwrap();
        assert_eq!(res.as_str(), output);
    }
}
