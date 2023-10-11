# simple-oracle

Feeder for the levana simple oracle

## Crates

- `erc4626`: Bindings for an ERC4626 contract
- `simple-oracle-bin`: Simple CLI binary crate
- `simple-oracle-lib`: Business logic

## Example config

For the signing key, pass in a mnemonic using the `COSMOS_WALLET` environment variable.

```toml
# decimal representing % change since last quote that should trigger an 
# asynchronous price update
price_variance_threshold = 0.025
# how frequently the price variance should be checked
check_variance_period = 30
# how often a price should be submitted regardless of variance
submission_period = 300 
# buffer to prevent the two periods from submitting a price at the same time
min_time_between_quotes = 6
osmosis_rpc_url = "https://..."
osomosis_grpc_url = "grpc://..."
ethereum_rpc_url = "https://..."

# add additional [[assets]] tables for each new asset
[[assets]]
ethereum_contract = "0x0000000000000000000000000000000000000000"
decimals = 18
base = "ETH"
quote = "RYETH"

# tells the oracle which cosmwasm contract to submit prices to for each etheruem contract
[contract_map]
0x0000000000000000000000000000000000000000 = "<CONTRACT ADDRESS>"
```
