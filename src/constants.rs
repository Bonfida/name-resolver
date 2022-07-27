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

pub const ERROR_URL: &str = "https://sol-domain.org";

pub const HOME_MSG: &str = "Visit https://bonfida.org";

pub const RECORDS_LOWER_CASE: [&str; 14] = [
    "ipfs", "arwv", "eth", "btc", "ltc", "doge", "email", "url", "discord", "github", "reddit",
    "twitter", "telegram", "shdw",
];

pub const RECORDS: [&str; 14] = [
    "IPFS", "ARWV", "ETH", "BTC", "LTC", "DOGE", "email", "url", "discord", "github", "reddit",
    "twitter", "telegram", "SHDW",
];
