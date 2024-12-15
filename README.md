# simple_ctv

## Requirements

- Rust https://www.rust-lang.org/tools/install
- Bitcoin Inquisition node https://github.com/bitcoin-inquisition/bitcoin

## Setup

follow this guide to compile bitcoin (works for the inquisition fork) I will add a docker file or something to do this eventually

https://jonatack.github.io/articles/how-to-compile-bitcoin-core-and-run-the-tests

### Install rustup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## signet
```
./bitcoind -signet -addnode=inquisition.bitcoin-signet.net -minrelaytxfee=0 -fallbackfee=0.0001
```
```
cargo run --features "signet"
```

## regtest
```
./bitcoind -regtest -minrelaytxfee=0 -fallbackfee=0.0001
```
```
cargo run --features "regtest"
```
