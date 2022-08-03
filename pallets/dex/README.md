# decentralized exchange pallet

Uses the substrate-node-template to build a custom pallet with basic functionality of a decentralised token exchange.
Written in Rust/Substrate/Frame


The main file is 

```shell
src/lib.rs
```

to compile, use:

```shell
cargo check -p pallet-dex
```


Storage Items
-------------------

LiqPool contains total supply of tokens of pair token1:token2
TokBal contains balance for each user

Functions
-------------------
dispatchables

add_liquidity() : user send token1 and token2 to the liquidityPool for staking
remove_liquidity() : user gets token back from liquidityPool
swap_tokens() : user swaps token according to the constant product liquidity pool exchange rate

helper

account_id() : get accountId of the treasury
enquire_rate() : ask how many token can be acquired for a given number of other token without actually swapping

To do list / improvements
-----------------------------

- implement staking rewards by taking x% of swapped tokens as fee and distribute according to staking amount per user
- no test cases yet
- implement pallet in runtime to build a full blockchain
