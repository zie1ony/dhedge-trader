use log::*;
use std::collections::HashMap;

pub type Symbol = String;

#[derive(Debug, Clone)]
pub struct Asset {
    pub balance: f64,
    pub rate: f64,
}

impl Asset {
    pub fn new(balance: f64, rate: f64) -> Asset {
        Asset { balance, rate }
    }
}

#[derive(Debug)]
pub struct Swap {
    pub from: Symbol,
    pub from_amount: f64,
    pub to: Symbol,
}

#[derive(Debug)]
pub struct Pool {
    expected_shares: HashMap<Symbol, f64>,
    assets: HashMap<Symbol, Asset>,
    min_trade_value: f64,
}

impl Pool {
    pub fn new(
        expected_shares: HashMap<Symbol, f64>,
        assets: HashMap<Symbol, Asset>,
        min_trade_value: f64,
    ) -> Pool {
        Pool {
            expected_shares,
            assets,
            min_trade_value,
        }
    }

    pub fn print_status(&self) {
        info!("Pool Status");

        let total_value = self.total_value();
        info!("Total Value: ${}", &total_value);
        info!("");

        info!("Pool Assets");
        info!(
            "{:<15} {:<15} {:<16} {:<16} {:<15} {:<15} {:<16} {:<15} {:<16}",
            "Asset",
            "Amount",
            "Rate",
            "Value",
            "Share",
            "Exp Share",
            "Exp Value",
            "Exp Bal Change",
            "Exp Val Change"
        );
        for symbol in &self.symbols() {
            info!(
                "{:<15} {:<15.5} ${:<15.5} ${:<15.5} {:<15.5} {:<15.5} ${:<15.5} {:<15.5} ${:<15.5}", 
                symbol,
                self.balance(symbol),
                self.rate(symbol),
                self.value(symbol),
                self.share(symbol),
                self.expected_share(symbol),
                self.expected_value(symbol),
                self.expected_balance_change(symbol),
                self.expected_value_change(symbol),
            );
        }
        let swaps = &self.rebalance_plan();
        info!("");
        info!("Rebalance Plan");
        info!("Actions required: {}", swaps.len());
        if swaps.len() > 0 {
            info!(
                "{:<15} {:<15} {:<15} {:<16}",
                "From", "To", "From Amount", "Value"
            );
            for swap in swaps {
                info!(
                    "{:<15} {:<15} {:<15.5} ${:<15.5}",
                    swap.from,
                    swap.to,
                    swap.from_amount,
                    swap.from_amount * self.rate(&swap.from)
                );
            }
        }
    }

    fn symbols(&self) -> Vec<Symbol> {
        self.expected_shares
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<Symbol>>()
            .clone()
    }

    fn asset(&self, symbol: &Symbol) -> &Asset {
        self.assets.get(symbol).unwrap()
    }

    fn balance(&self, symbol: &Symbol) -> f64 {
        self.asset(symbol).balance
    }

    fn rate(&self, symbol: &Symbol) -> f64 {
        self.asset(symbol).rate
    }

    fn value(&self, symbol: &Symbol) -> f64 {
        self.balance(symbol) * self.rate(symbol)
    }

    fn share(&self, symbol: &Symbol) -> f64 {
        self.value(symbol) / self.total_value()
    }

    fn expected_share(&self, symbol: &Symbol) -> f64 {
        self.expected_shares.get(symbol).unwrap().clone()
    }

    fn expected_value(&self, symbol: &Symbol) -> f64 {
        self.total_value() * self.expected_share(symbol)
    }

    fn expected_balance_change(&self, symbol: &Symbol) -> f64 {
        (self.expected_value(symbol) - self.value(symbol)) / self.rate(symbol)
    }

    fn expected_value_change(&self, symbol: &Symbol) -> f64 {
        self.expected_balance_change(symbol) * self.rate(symbol)
    }

    fn total_value(&self) -> f64 {
        let mut total = 0.0;
        for symbol in &self.symbols() {
            total += self.balance(symbol) * self.rate(symbol);
        }
        total
    }

    pub fn rebalance_plan(&self) -> Vec<Swap> {
        let mut swaps: Vec<Swap> = Vec::new();

        let mut symbols = self.symbols();
        symbols.sort_by(|a, b| {
            let a_val = self.expected_value_change(a);
            let b_val = self.expected_value_change(b);
            a_val.partial_cmp(&b_val).unwrap()
        });

        while symbols.len() > 1 {
            let len = symbols.len();
            let best = &symbols[0];
            let worst = &symbols[len - 1];
            let worst_value_change = self.expected_value_change(&worst);
            let best_value_change = self.expected_value_change(&best);

            if worst_value_change <= 0.0 || best_value_change >= 0.0 {
                break;
            }

            let value_change = if worst_value_change.abs() > best_value_change {
                worst_value_change.abs()
            } else {
                best_value_change
            };

            if value_change < self.min_trade_value {
                break;
            }

            swaps.push(Swap {
                from: best.to_string(),
                to: worst.to_string(),
                from_amount: value_change / self.rate(best),
            });

            symbols = symbols[1..(len - 1)].to_vec();
        }
        swaps
    }

    pub fn balanced(&self) -> bool {
        self.rebalance_plan().len() == 0
    }

    pub fn rebalance(&mut self) {
        if self.balanced() {
            info!("Pool is already balanced.");
            return;
        };

        info!("Rebalancing:");
        for swap in &self.rebalance_plan() {
            let current_from_balance = self.asset(&swap.from.to_string()).balance;
            let current_to_balance = self.asset(&swap.to.to_string()).balance;
            let to_amount = swap.from_amount * self.rate(&swap.from) / self.rate(&swap.to);
            self.assets.insert(
                swap.from.clone(),
                Asset {
                    rate: self.rate(&swap.from),
                    balance: current_from_balance - swap.from_amount,
                },
            );
            self.assets.insert(
                swap.to.clone(),
                Asset {
                    rate: self.rate(&swap.to),
                    balance: current_to_balance + to_amount,
                },
            );
            info!("[x] Swaped {} to {}", swap.from, swap.to);
        }
        info!("Rebalancing done!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut expected_shares = HashMap::new();
        expected_shares.insert("sBTC".to_string(), 0.20);
        expected_shares.insert("sETH".to_string(), 0.20);
        expected_shares.insert("sUSD".to_string(), 0.20);
        expected_shares.insert("sBNB".to_string(), 0.20);
        expected_shares.insert("sLTC".to_string(), 0.20);

        let mut assets = HashMap::new();
        assets.insert("sBTC".to_string(), Asset::new(0.9, 10000.0));
        assets.insert("sETH".to_string(), Asset::new(20.2, 200.0));
        assets.insert("sUSD".to_string(), Asset::new(100.0, 1.0));
        assets.insert("sBNB".to_string(), Asset::new(50.0, 18.0));
        assets.insert("sLTC".to_string(), Asset::new(10.0, 45.0));

        let min_trade_value = 1.0;

        let mut pool = Pool::new(expected_shares, assets, min_trade_value);
        pool.print_status();
        for _ in 0..3 {
            pool.rebalance();
            pool.print_status();
        }
        assert!(pool.balanced());
    }
}
