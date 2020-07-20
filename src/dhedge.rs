use std::collections::HashMap;
use log::*;

use web3::types::{Address, U256};
use web3::contract::{Contract, Options};

use crate::{Symbol, Asset, Swap, parser};

pub struct DHedge {
    manager: Address,
    contract: Contract<web3::transports::Http>,
    web3: web3::Web3<web3::transports::Http>
}

impl DHedge {
    pub async fn new(endpoint: &str, pool_address: &str) -> DHedge {
        let pool_address: Address = pool_address.parse().unwrap();
        let http = web3::transports::Http::new(endpoint).unwrap();
        let web3 = web3::Web3::new(http);
        let contract = Contract::from_json(
            web3.eth(),
            pool_address,
            include_bytes!("dhedge.abi"),
        ).unwrap();
        
        let accounts = web3.personal().list_accounts().await.unwrap();
        let manager = accounts[0];
        info!("Using manager account: {}", manager);        

        DHedge {
            manager, contract, web3
        }
    }

    pub async fn get_fund_composition(&self) -> HashMap<Symbol, Asset> {
        info!("Reading from blockchain. Calling getFundComposition at {}", self.contract.address());
        let data = self.contract.query("getFundComposition", (), None, Options::default(), None);
        let (symbols, balances, rates): (Vec<[u8; 32]>, Vec<U256>, Vec<U256>) = data.await.unwrap();
        let mut assets: HashMap<Symbol, Asset> = HashMap::new();
        for i in 0..symbols.len() {
            let symbol = parser::to_asset_name(&symbols[i]);
            let balance = parser::u256_to_f64(&balances[i]);
            let rate = parser::u256_to_f64(&rates[i]);
            assets.insert(symbol.clone(), Asset {balance, rate});
            info!("[x] {} x {} at price {}", symbol, balance, rate);
        }
        info!("Reading from blockchain done!");
        assets
    }


    pub async fn exchange(&self, swap: &Swap) {
        self.unlock().await;
        let args = (
            parser::asset_name_to_bytes(swap.from.clone()),
            parser::u256(swap.from_amount),
            parser::asset_name_to_bytes(swap.to.clone())
        );
        let tx = self.contract.call("exchange", args, self.manager, Options::default()).await.unwrap();
        info!("Exchange method called with params:");
        info!("From: {}", swap.from);
        info!("Amount: {}", swap.from_amount);
        info!("To: {}", swap.to);
        info!("TxHash: {}", tx);
    }

    pub async fn unlock(&self) {
        let unlocked: bool = self.web3.personal().unlock_account(self.manager, "matrix11a", None).await.unwrap();
        if !unlocked {
            panic!("Not able to unlock the account");
        };
    }
}