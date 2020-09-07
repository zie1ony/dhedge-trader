# dHedge Trader

Experimental dHedge rebalancer bot. Work in progress...

## Features
- [x] Rebalances portfolio to keep fixed percentages of assets.
- [x] Detects the best possible trades.
- [x] Cap the min trades amount to fight gas prices. 
- [ ] Config via CLI arguments. For now the configuration is done in `main.rs`.

## Usage

#### Ethereum Node
It is required to run your own `openethereum` node with loaded and unlocked private key.

#### Set config.
Update `HTTP_ENDPOINT` to point the Ethereum Node and `POOL_ADDRESS` to be your dHedge pool address.

#### Build
Build the binary.
```
$ cargo build --release
```

#### Cron
Execute periodically via Cron and capture all logs.
```
*/3 * * * *   /home/user/dht 2>&1 | tee -a /home/user/dht.log
```
