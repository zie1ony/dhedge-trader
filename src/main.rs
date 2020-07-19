#[macro_use] extern crate log;

use web3::contract::{Contract, Options};
use web3::types::{Address, U256};

use log::Level;

const HTTP_ENDPOINT: &str = "https://ropsten.infura.io/v3/26fe081e46de47e9b60003d2fc69bbf5";
const POOL_ADDRESS: &str = "8b75a9b0a172f155f15a0f4eded511364e064267";
const MANAGER_ADDRESS: &str = "9A271aAF6d6E1F0752ff591390492d96C3114f02";


#[tokio::main]
async fn main() -> Result<(), ()> {
    let _ = env_logger::try_init();


    // let manager: Address = MANAGER_ADDRESS.parse().unwrap();
    // let pool_address: Address = POOL_ADDRESS.parse().unwrap();
    // let http = web3::transports::Http::new(HTTP_ENDPOINT)?;
    // let web3 = web3::Web3::new(http);
    // let pool = Contract::from_json(
    //     web3.eth(),
    //     pool_address,
    //     include_bytes!("dhedge.abi"),
    // )?;

    // let result = pool.query("getFundComposition", (), None, Options::default(), None);
    // let state: (Vec<[u8; 32]>, Vec<U256>, Vec<U256>) = result.await?;
    // let assets_len = state.0.len();

    Ok(())
}

// Remove trailing zeros
fn to_asset_name(raw: &[u8; 32]) -> String {
    let mut result: Vec<u8> = Vec::new();
    let mut started = false;
    for i in (0..32).rev() {
        if started {
            result.push(raw[i]);
        } else {
            if raw[i] != 0 {
                result.push(raw[i]);
                started = true;
            }
        };
    }
    result = result.into_iter().rev().collect();
    let name = std::str::from_utf8(&result).unwrap();
    name.to_string()
}

fn u256_to_string(value: &U256) -> String {
    let d = U256::exp10(18);
    let (a, b) = value.div_mod(d);
    format!("{}.{}", a.as_u64(), b.as_u64())
}

fn u256_to_f64(value: &U256) -> f64 {
    u256_to_string(value).parse().unwrap()
}

fn u256(val: f64) -> U256 {
    let str_val = val.to_string();
    let vals: Vec<&str> = str_val.split(".").collect();
    let (a, b) = if vals.len() == 1 {
        (vals[0], "0")
    } else {
        (vals[0], vals[1])
    };
    let a = U256::from_dec_str(a).unwrap() * U256::exp10(18);
    let b = U256::from_dec_str(b).unwrap() * U256::exp10(18-b.len());
    a + b
}

