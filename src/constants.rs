pub const NAME_RECORD_HEADER_LEN: usize = 96;

pub const MAINNET_URL: &str = "https://bonfida.genesysgo.net";

pub const HASH_PREFIX: &str = "SPL Name Service";

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

pub const ERROR_URL: &str = "https://sol-domain.org";

pub const HOME_MSG: &str = "Visit https://bonfida.org";
