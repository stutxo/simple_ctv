use std::env;

use bitcoin::Network;

use bitcoincore_rpc::{Auth, Client, RpcApi};

// https://bitcoinops.org/en/bitcoin-core-28-wallet-integration-guide/
// mainnet: bc1pfeessrawgf
// regtest: bcrt1pfeesnyr2tx
// testnet: tb1pfees9rn5nz

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub network: Network,
    pub port: &'static str,
    pub fee_anchor_addr: &'static str,
    pub wallet_name: String,
}

impl NetworkConfig {
    #[allow(clippy::needless_return)]
    pub fn new() -> Self {
        #[cfg(feature = "regtest")]
        {
            return Self {
                network: Network::Regtest,
                port: "18443",
                fee_anchor_addr: "bcrt1pfeesnyr2tx",
                wallet_name: "simple_ctv".to_string(),
            };
        }
        #[cfg(feature = "signet")]
        {
            let wallet_name = env::var("SIGNET_WALLET").expect("SIGNET_WALLET env var not set");
            println!("wallet name: {}", wallet_name);
            return Self {
                network: Network::Signet,
                port: "38332",
                fee_anchor_addr: "tb1pfees9rn5nz",
                wallet_name,
            };
        }
        //wen mainnet
    }

    pub fn bitcoin_rpc(&self) -> Client {
        let bitcoin_rpc_user =
            env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER env var not set");
        let bitcoin_rpc_pass =
            env::var("BITCOIN_RPC_PASS").expect("BITCOIN_RPC_PASS env var not set");

        let bitcoin_rpc_url =
            format!("http://localhost:{}/wallet/{}", self.port, self.wallet_name,);

        println!("wallet name in use: {}", self.wallet_name);

        let bitcoin_rpc = Client::new(
            &bitcoin_rpc_url,
            Auth::UserPass(bitcoin_rpc_user, bitcoin_rpc_pass),
        )
        .unwrap();

        #[cfg(feature = "regtest")]
        let regtest_wallet = bitcoin_rpc.create_wallet(&self.wallet_name, None, None, None, None);
        #[cfg(feature = "regtest")]
        if regtest_wallet.is_ok() {
            println!("regtest wallet created")
        }

        let _ = bitcoin_rpc.load_wallet(&self.wallet_name);

        bitcoin_rpc
    }
}
