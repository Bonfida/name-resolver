use {
    serde_json::Value,
    sha2::{Digest, Sha256},
    std::str::from_utf8,
    worker::{console_log, Headers, Response, Url},
};

use futures::join;

use crate::constants::{
    HASH_PREFIX, MAINNET_URL, NAME_RECORD_HEADER_LEN, RECORDS, RECORDS_LOWER_CASE,
    ROOT_DOMAIN_ACCOUNT, SPL_NAME_SERVICE_ID, STATUS_CODE_MOVED_PERMANENTLY,
};

#[derive(Debug)]
pub enum Error {
    Worker(worker::Error),
    InvalidRecord,
    SerdeJson(serde_json::Error),
    Network(reqwest::Error),
    CouldNotFetch,
    Base64(base64::DecodeError),
    Utf8(std::str::Utf8Error),
    ParseUrl,
}

impl From<worker::Error> for Error {
    fn from(value: worker::Error) -> Self {
        Self::Worker(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64(value)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Record {
    Url,
    Ipfs,
    Arwv,
    Shdw,
    A,
    Cname,
}

impl Record {
    pub fn as_str(&self) -> String {
        match self {
            Record::Url => "url".to_owned(),
            Record::Ipfs => "IPFS".to_owned(),
            Record::Arwv => "ARWV".to_owned(),
            Record::Shdw => "SHDW".to_owned(),
            Record::A => "A".to_owned(),
            Record::Cname => "CNAME".to_owned(),
        }
    }

    pub fn as_str_with_prefix(&self) -> String {
        format!("\x01{}", self.as_str())
    }

    pub fn from_str(x: &str) -> Result<Record, Error> {
        match x.to_uppercase().as_str() {
            "URL" => Ok(Record::Url),
            "IPFS" => Ok(Record::Ipfs),
            "ARWV" => Ok(Record::Arwv),
            "SHDW" => Ok(Record::Shdw),
            "A" => Ok(Record::A),
            "CNAME" => Ok(Record::Cname),
            _ => Err(Error::InvalidRecord),
        }
    }
}

pub async fn resolve_domain(sns_name: &str) -> Result<worker::Response, Error> {
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

    match record {
        None => {
            // No record specified, default to URL, IPFS, ARWV then SHDW do it in parallel
            let url_record = find_name_key(&Record::Url.as_str_with_prefix(), &domain_key);
            let ipfs_record = find_name_key(&Record::Ipfs.as_str_with_prefix(), &domain_key);
            let arwv_record = find_name_key(&Record::Arwv.as_str_with_prefix(), &domain_key);
            let shdw_record = find_name_key(&Record::Shdw.as_str_with_prefix(), &domain_key);
            let a_record = find_name_key(&Record::A.as_str_with_prefix(), &domain_key);
            let cname_record = find_name_key(&Record::Cname.as_str_with_prefix(), &domain_key);

            let res_tuple = join!(
                fetch_record(&url_record, Record::Url, sns_name),
                fetch_record(&ipfs_record, Record::Ipfs, sns_name),
                fetch_record(&arwv_record, Record::Arwv, sns_name),
                fetch_record(&shdw_record, Record::Shdw, sns_name),
                fetch_record(&a_record, Record::A, sns_name),
                fetch_record(&cname_record, Record::Cname, sns_name)
            );

            res_tuple
                .0
                .map_or(res_tuple.1, Ok)
                .map_or(res_tuple.2, Ok)
                .map_or(res_tuple.3, Ok)
                .map_or(res_tuple.4, Ok)
                .map_or(res_tuple.5, Ok)
        }
        Some(r) => {
            let record = Record::from_str(r)?;
            let record_key = find_name_key(&record.as_str_with_prefix(), &domain_key);
            fetch_record(&record_key, record, sns_name).await
        }
    }
}

pub async fn fetch_record(
    record_key: &[u8; 32],
    record_type: Record,
    domain: &str,
) -> Result<worker::Response, Error> {
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
        return Err(Error::CouldNotFetch);
    }

    let a = res.text().await?;

    let json_return: Value = serde_json::from_str(&a)?;
    let url_str = json_return["result"]["value"]["data"][0]
        .as_str()
        .ok_or_else(|| Error::CouldNotFetch)?;

    let decoded = base64::decode(url_str)?[NAME_RECORD_HEADER_LEN..].to_vec();
    let result = match record_type {
        Record::A => decoded
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("."),
        _ => from_utf8(&decoded)?.to_string(),
    };

    format_response(&result, record_type, domain)
}

fn format_response(
    result: &str,
    record_type: Record,
    domain: &str,
) -> Result<worker::Response, Error> {
    let (url, is_redirect) = match record_type {
        Record::Ipfs => (
            format!(
                "https://cloudflare-ipfs.com/ipfs/{}",
                result.strip_prefix("ipfs://").unwrap_or(result)
            ),
            true,
        ),
        Record::Arwv => (
            format!(
                "https://arweave.net/{}",
                result.strip_prefix("arwv://").unwrap_or(result)
            ),
            true,
        ),
        Record::Shdw => (
            format!(
                "https://shdw-drive.genesysgo.net/{}",
                result.strip_prefix("shdw://").unwrap_or(result)
            ),
            true,
        ),
        Record::Cname | Record::A => (format!("http://{}", result), false),
        Record::Url => (result.to_string(), true),
    };

    if is_redirect {
        let parsed = Url::parse(&url).map_err(|_| Error::ParseUrl)?;
        Ok(Response::redirect(parsed)?)
    } else {
        let mut headers = Headers::new();
        headers.set("Location", &url)?;
        if matches!(record_type, Record::A) {
            headers.set("Host", &format!("{domain}.sol"))?;
        }

        Ok(Response::empty()?
            .with_headers(headers)
            .with_status(STATUS_CODE_MOVED_PERMANENTLY))
    }
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
            key_hasher.update(SPL_NAME_SERVICE_ID);
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
