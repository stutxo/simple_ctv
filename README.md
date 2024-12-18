# CTV + Pay To Anchor Example 🥪 + ⚓

Here is a simple example of spending a transaction from a CTV (CHECKTEMPLATEVERIFY) contract via Pay To Anchor (P2A), with an extra input to bump the fees.

## Index


1. [Requirements](#requirements)
2. [Setup Instructions](#setup)
    - [Install Rust](#install-rust)
    - [Signet Configuration](#signet)
    - [Regtest Configuration](#regtest)
3. [Resources](#resources)
4. [Transaction Examples](#transaction-examples)
    - [Minimum Possible Fees with No Extra Input](#minimum-possible-fees-with-no-extra-input)
    - [Adding Extra Input to Cover the Fees](#adding-extra-input-to-cover-the-fees)
    - [Bumping Fee by Deducting Fee from CTV Output](#bumping-fee-by-deducting-fee-from-ctv-output)
    - [Spending Using Just a 1p1c Package](#spending-using-just-a-1p1c-package)


## Requirements

- Rust https://www.rust-lang.org/tools/install
- Bitcoin Inquisition 28.0 node https://github.com/bitcoin-inquisition/bitcoin/releases/tag/v28.0-inq

## Setup

follow this guide to compile bitcoin (works for the inquisition fork) I will add a docker file or something to do this eventually

https://jonatack.github.io/articles/how-to-compile-bitcoin-core-and-run-the-tests

### install rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### signet
```bash
./bitcoind -signet -addnode=inquisition.bitcoin-signet.net -minrelaytxfee=0 -fallbackfee=0.0001
```
```bash
export BITCOIN_RPC_USER="rpc_username"
export BITCOIN_RPC_PASS="rpc_password"
export SIGNET_WALLET="signet wallet name"
cargo run --features "signet"
```

### regtest
```bash
./bitcoind -regtest -minrelaytxfee=0 -fallbackfee=0.0001
```
```bash
export BITCOIN_RPC_USER="rpc_username"
export BITCOIN_RPC_PASS="rpc_password"
cargo run --features "regtest"
```
### Resources

Shoutout to **supertestnet**, **bennyhodl**, **glozow** and **Jeremy Rubin** for their code/examples.

- [Bennyhodl's dlcat for the ctv hash function](https://github.com/bennyhodl/dlcat)
- [Supertestnet Zero Fee Playground](https://x.com/super_testnet/status/1866982166833578286)
- [Ephemeral Anchors on Bitcoin Optech](https://bitcoinops.org/en/topics/ephemeral-anchors/)
- [Bitcoin GitHub PR #30239](https://github.com/bitcoin/bitcoin/pull/30239)
- [Bitcoin Core 0.28 Wallet Integration Guide on Bitcoin Optech](https://bitcoinops.org/en/bitcoin-core-28-wallet-integration-guide/)

### Transaction Examples:

### Minimum Possible Fees with No Extra Input

- [orignal fee spend: Parent Transaction on Mempool Space](https://mempool.space/signet/tx/8b0c09b92387ddbbea7ae2a8bca24e48a28551be4025c50ab74844ac8001077c)
- [orignal fee spend: Child Transaction on Mempool Space](https://mempool.space/signet/tx/bf1e5af2e886a8f2072dcecad6cff1f736084983713bd32df606259e15bab67f)

### Adding Extra Input to Cover the Fees

- [bump fee with extra input: Parent Transaction on Mempool Space](https://mempool.space/signet/tx/32f4f4e6165e7f8df9b9a762e11a6ca7f16087713e0e3e42352021e6bf3800e3)
- [bump fee with extra input: Child Transaction on Mempool Space](https://mempool.space/signet/tx/9a3582f03b0ac39cff8ed024cf8f38e4fc4a1ee2ff216badf041bf4572c0d03b)

  <img src="/screenshots/zero-fee-ctv-spend-1.png" alt="alt text">
  <img src="/screenshots/zero-fee-ctv-spend-in&outs.png" alt="alt text">
  <img src="/screenshots/anchor-cpfp-spend.png" alt="alt text">

### Bumping Fee by Deducting Fee from CTV Output

- [bump fee with extra input or child transaction: Parent Transaction on Mempool Space](https://mempool.space/signet/tx/86896275fb71d4e3b84d1edeeacb90f7c4ccf77ee3a29e66d7effff4bb0682fb)


### Spending Using Just a 1p1c Package

https://x.com/1440000bytes/status/1868375944832156108

- [spending wih 1p1c package: Parent Transaction on Mempool Space](https://mempool.space/signet/tx/75bf34f89d82c4a6783a6f8d51dd7a1d8cdc0799f31a367b01bbec655fd79dab)
