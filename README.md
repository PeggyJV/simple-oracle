# simple-oracle

Feeder for the levana simple oracle

## Crates

- `erc4626`: Bindings for an ERC4626 contract
- `simple-oracle-bin`: Simple CLI binary crate
- `simple-oracle-lib`: Business logic

## Example config

```toml
signing_key_path = "key.pem"
check_variance_period = 30
submission_period = 300 
min_time_between_quotes = 6

# add additional [[assets]] tables for each new asset
[[assets]]
ethereum_contract = "0x0000000000000000000000000000000000000000"
decimals = 18
base = "ETH"
quote = "RYETH"

# tells the oracle which cosmwasm contract to submit prices to for each etheruem contract
[contract_map]
0x0000000000000000000000000000000000000000 = "osmo100000000000000000000000000000001"
```
