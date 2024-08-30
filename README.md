# Solana Token 2022 Staking program
This repo contains a staking contract that allows users to create staking pools and stake tokens, all incorporating the new Token 20222 program! 

## Instructions

### `init_pool`
Initializes a new staking pool. The pool is a pda with the address of the token mint that the pool is intended for and "state" as seeds.

### `init_stake_entry`
Initializes an account to hold state about a user's stake position. PDA with the User's pubkey, mint of token, and "stake_entry" as seeds.

### `stake`
Transfers tokens from a User token account to the program token vault, where they are kept while staked.

### `unstake`
Transfers tokens from the staking pool back to a user.

User can call this at any time.

Users can only unstake tokens that they have staked themselves.
