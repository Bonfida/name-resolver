use std::str::from_utf8;

use serde_json::Value;
use sha2::{Digest, Sha256};
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .post_async("/form/:sns_name", |mut req, ctx| async move {
            if let Some(sns_name) = ctx.param("sns_name") {
                let form = req.form_data().await?;
                match form.get(sns_name) {
                    Some(FormEntry::Field(value)) => {
                        let name_url = get_name_url(&value).await;
                        return Response::redirect(Url::parse(&name_url).unwrap());
                    }
                    Some(FormEntry::File(_)) => {
                        return Response::error(
                            "`sns_name` param in form shouldn't be a File",
                            422,
                        );
                    }
                    None => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}

const NAME_RECORD_HEADER_LEN: usize = 96;
const MAINNET_URL: &str = "https://solana-api.projectserum.com/";
const HASH_PREFIX: &str = "SPL Name Service";
pub const ROOT_DOMAIN_ACCOUNT: [u8; 32] = [
    61, 83, 194, 75, 56, 54, 14, 211, 129, 58, 35, 223, 178, 223, 216, 32, 171, 88, 33, 203, 121,
    41, 163, 141, 46, 170, 178, 82, 232, 56, 37, 149,
];
pub const SPL_NAME_SERVICE_ID: [u8; 32] = [
    11, 173, 81, 244, 19, 193, 243, 169, 148, 96, 217, 0, 216, 191, 46, 214, 146, 126, 202, 52,
    215, 183, 132, 43, 248, 16, 169, 115, 8, 45, 30, 220,
];
/// Hardcoding hashv(&[(HASH_PREFIX.to_owned() + "\0url").as_bytes()])
pub const URL_RECORD_NAME_HASHED: [u8; 32] = [
    88, 96, 166, 135, 118, 236, 203, 137, 158, 127, 28, 22, 133, 67, 93, 29, 194, 166, 137, 96, 68,
    56, 40, 62, 177, 97, 252, 247, 192, 110, 40, 168,
];

/// Fetch and deseriealize the URL value stored in the SNS domain names data
async fn get_name_url(sns_name: &str) -> String {
    let mut name_hasher = Sha256::new();
    name_hasher.update(HASH_PREFIX.to_owned() + sns_name);
    let hashed_name = name_hasher.finalize();

    let name_account_key = find_name_key(&hashed_name, &ROOT_DOMAIN_ACCOUNT);
    println!("NAME acc key {:?}", name_account_key);
    let url_name_record_key = find_name_key(&URL_RECORD_NAME_HASHED, &name_account_key);
    println!("URL acc key {:?}", url_name_record_key);

    let request_data = format!(
        "
    {{
        \"jsonrpc\": \"2.0\",
        \"id\": 1,
        \"method\": \"getAccountInfo\",
        \"params\": [
          \"{:?}\",
          {{
            \"encoding\": \"base64\"
          }}
        ]
      }}",
        url_name_record_key
    );

    let request_json: Value = serde_json::from_str(&request_data).unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(MAINNET_URL)
        .json(&request_json)
        .send()
        .await
        .unwrap();
    if !res.status().is_success() {
        println!(
            "Could not find the domain name account. Error {:?}",
            res.status().as_str()
        );
        panic!();
    }

    let a = res.text().await.unwrap();
    let json_return: Value = serde_json::from_str(&a).unwrap();

    let url_str = &json_return["result"]["value"]["data"][0].as_str().unwrap()
        [NAME_RECORD_HEADER_LEN..]
        .trim_start_matches('A')
        .trim_end_matches('=')
        .trim_end_matches('A');
    let decoded_url_bytes = &base64::decode(url_str).unwrap();

    from_utf8(decoded_url_bytes).unwrap().to_string()
}

fn find_name_key(hashed_name: &[u8], parent_key: &[u8]) -> [u8; 32] {
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
