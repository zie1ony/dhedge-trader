use log::*;
use std::collections::HashMap;

use web3::contract::{Contract, Options};
use web3::types::{Address, U256};

use crate::{parser, Asset, Swap, Symbol};

type Transport = web3::transports::batch::Batch<web3::transports::http::Http>;

pub struct DHedge {
    manager: Address,
    contract: Contract<Transport>,
    web3: web3::Web3<Transport>,
}

impl DHedge {
    pub async fn new(endpoint: &str, pool_address: &str) -> DHedge {
        let pool_address: Address = pool_address.parse().unwrap();
        let http = web3::transports::Http::new(endpoint).unwrap();
        let web3 = web3::Web3::new(web3::transports::Batch::new(http));
        let contract =
            Contract::from_json(web3.eth(), pool_address, include_bytes!("dhedge.abi")).unwrap();

        let accounts = web3.personal().list_accounts();

        let _ = web3.transport().submit_batch().await.unwrap();

        let manager = accounts.await.unwrap()[0];
        info!("Using manager account: {}", manager);

        DHedge {
            manager,
            contract,
            web3,
        }
    }

    pub async fn get_fund_composition(&mut self) -> HashMap<Symbol, Asset> {
        info!(
            "Reading from blockchain. Calling getFundComposition at {}",
            self.contract.address()
        );
        let data = self
            .contract
            .query("getFundComposition", (), None, Options::default(), None);
        self.submit_batch().await;
        let (symbols, balances, rates): (Vec<[u8; 32]>, Vec<U256>, Vec<U256>) = data.await.unwrap();
        let mut assets: HashMap<Symbol, Asset> = HashMap::new();
        for i in 0..symbols.len() {
            let symbol = parser::to_asset_name(&symbols[i]);
            let balance = parser::u256_to_f64(&balances[i]);
            let rate = parser::u256_to_f64(&rates[i]);
            assets.insert(symbol.clone(), Asset { balance, rate });
            info!("[x] {} x {} at price {}", symbol, balance, rate);
        }
        info!("Reading from blockchain done!");
        assets
    }

    pub async fn rebalance(&mut self, swaps: Vec<Swap>) {
        let mut nonce = self.get_nonce().await;
        let mut txs = Vec::new();
        for swap in &swaps {
            let args = (
                parser::asset_name_to_bytes(swap.from.clone()),
                parser::u256(swap.from_amount),
                parser::asset_name_to_bytes(swap.to.clone()),
            );
            let options = Options::with(|o| o.nonce = Some(nonce));
            nonce += U256::one();
            let tx = self.contract.call("exchange", args, self.manager, options);
            txs.push(tx);
        }
        self.submit_batch().await;

        let mut i = 0;
        for tx in txs {
            let tx_hash = tx.await.unwrap();
            let swap = &swaps[i];
            i += 1;
            info!(
                "Exchanged {} {} to {} on hash {}",
                swap.from_amount, swap.from, swap.to, tx_hash
            );
        }
    }

    pub async fn submit_batch(&mut self) {
        let _ = self.web3.transport().submit_batch().await.unwrap();
    }

    pub async fn get_nonce(&mut self) -> U256 {
        let nonce = self.web3.eth().transaction_count(self.manager, None);
        self.submit_batch().await;
        nonce.await.unwrap()
    }
}
