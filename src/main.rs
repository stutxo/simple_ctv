use anyhow::Result;
use core::time;
use std::{env, str::FromStr, thread};

use bitcoin::{
    absolute,
    consensus::{encode::serialize_hex, Encodable},
    hashes::{sha256, Hash},
    key::{Keypair, Secp256k1},
    opcodes::all::{OP_DROP, OP_NOP4, OP_RETURN},
    script::Builder,
    taproot::{LeafVersion, TaprootBuilder, TaprootSpendInfo},
    transaction::{self},
    Address, Amount, Network, Opcode, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    XOnlyPublicKey,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};

const OP_CTV: Opcode = OP_NOP4;

// https://bitcoinops.org/en/bitcoin-core-28-wallet-integration-guide/
// mainnet: bc1pfeessrawgf
// regtest: bcrt1pfeesnyr2tx
// testnet: tb1pfees9rn5nz

#[cfg(feature = "signet")]
const FEE_ANCHOR_ADDR: &str = "tb1pfees9rn5nz";
#[cfg(feature = "signet")]
const NETWORK: Network = Network::Signet;
#[cfg(feature = "signet")]
const PORT: &str = "38332";
#[cfg(feature = "signet")]
//change this to your own signet wallet name
const WALLET_NAME: &str = "siggy";

const FEE_ANCHOR_ADDR: &str = "bcrt1pfeesnyr2tx";
const NETWORK: Network = Network::Regtest;
const PORT: &str = "18443";
const WALLET_NAME: &str = "simple_ctv";

//this is the min dut amount required to anchor the transaction
const ANCHOR_AMOUNT: u64 = 240;

fn main() {
    let bitcoin_rpc_user = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER not set");
    let bitcoin_rpc_pass = env::var("BITCOIN_RPC_PASS").expect("BITCOIN_RPC_PASS not set");

    let bitcoin_rpc_url = format!("http://localhost:{}/wallet/{}", PORT, WALLET_NAME);

    let bitcoin_rpc = Client::new(
        &bitcoin_rpc_url,
        Auth::UserPass(bitcoin_rpc_user, bitcoin_rpc_pass),
    )
    .unwrap();

    let create_wallet = bitcoin_rpc.create_wallet(WALLET_NAME, None, None, None, None);

    if create_wallet.is_ok() {
        println!("Wallet created successfully.");
    }

    let load_wallet = bitcoin_rpc.load_wallet(WALLET_NAME);

    match load_wallet {
        Ok(_) => println!("Wallet loaded successfully."),
        Err(e) => println!("Error loading wallet: {:?}", e),
    }

    let ctv_spend_address = bitcoin_rpc.get_new_address(None, None).unwrap();
    let ctv_spend_address = ctv_spend_address.require_network(NETWORK).unwrap();

    println!("\nCTV spend address: {}", ctv_spend_address);

    let amount = Amount::from_sat(1337) + Amount::from_sat(ANCHOR_AMOUNT);

    let anchor_addr = Address::from_str(FEE_ANCHOR_ADDR)
        .unwrap()
        .require_network(NETWORK)
        .unwrap();

    let amount_out_1 = amount - Amount::from_sat(ANCHOR_AMOUNT);
    let amount_out_2 = Amount::from_sat(ANCHOR_AMOUNT);

    let ctv_tx_out = [
        TxOut {
            value: amount_out_1,
            script_pubkey: ctv_spend_address.script_pubkey(),
        },
        TxOut {
            value: amount_out_2,
            script_pubkey: anchor_addr.script_pubkey(),
        },
    ];

    //calculate ctv hash
    let ctv_hash = calc_ctv_hash(&ctv_tx_out, None);

    // create ctv contract address
    let ctv_tr_spend_info = create_ctv_address(ctv_hash).unwrap();
    let ctv_contract_address = Address::p2tr_tweaked(ctv_tr_spend_info.output_key(), NETWORK);
    println!("\nCTV address: {}", ctv_contract_address);

    //enable this if you need to fund your regtest wallet
    // #[cfg(not(feature = "signet"))]
    // let _ = bitcoin_rpc.generate_to_address(101, &ctv_spend_address);

    let txid_result = bitcoin_rpc.send_to_address(
        &ctv_contract_address,
        amount,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let funding_txid = match txid_result {
        Ok(txid) => {
            println!("\nFunding transaction sent: {}", txid);
            txid
        }
        Err(e) => {
            eprintln!("Error sending funding transaction: {:?}", e);
            return;
        }
    };

    println!("\nSpending ctv transaction...");

    #[cfg(not(feature = "signet"))]
    let _ = bitcoin_rpc.generate_to_address(1, &ctv_spend_address);

    //we have to wait for the funding transaction to be confirmed to pay to anchor

    loop {
        let transaction_info = bitcoin_rpc.get_transaction(&funding_txid, None).unwrap();
        let confirmations = transaction_info.info.confirmations;

        println!(
            "Current confirmations: {} for funding transaction {}",
            confirmations, funding_txid
        );

        if confirmations >= 1 {
            println!("Funding Transaction is confirmed! We can now spend the CTV transaction.");
            break;
        }

        thread::sleep(time::Duration::from_secs(10));
    }

    let inputs = vec![TxIn {
        previous_output: OutPoint {
            txid: funding_txid,
            vout: 0,
        },
        sequence: Sequence(0xfffffffd),
        ..Default::default()
    }];

    let unsigned_tx = Transaction {
        version: transaction::Version(3),
        lock_time: absolute::LockTime::ZERO,
        input: inputs,
        output: ctv_tx_out.to_vec(),
    };

    let parent_tx = spend_ctv(unsigned_tx, ctv_tr_spend_info, ctv_hash);

    let parent_serialized_tx = serialize_hex(&parent_tx);

    println!("\nCTV parent tx: {}", parent_serialized_tx);

    let parent_txid = bitcoin_rpc
        .send_raw_transaction(parent_serialized_tx)
        .unwrap();

    println!("\nCTV parent txid: {}", parent_txid);

    println!("\nSpending child transaction...");

    let data = b"99 problems but 0 fees aint 1 - stutxo";

    let child_script = Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(data)
        .into_script();

    let child_spend = Transaction {
        version: transaction::Version(3),
        lock_time: absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: parent_txid,
                vout: 1,
            },
            ..Default::default()
        }],
        output: vec![TxOut {
            value: Amount::from_sat(0),
            script_pubkey: child_script,
        }],
    };

    let child_serialized_tx = serialize_hex(&child_spend);

    println!("child tx: {}", child_serialized_tx);

    let child_txid = bitcoin_rpc
        .send_raw_transaction(child_serialized_tx)
        .unwrap();

    println!("child txid: {}", child_txid);
}

fn create_ctv_address(ctv_hash: [u8; 32]) -> Result<TaprootSpendInfo> {
    let secp = Secp256k1::new();

    let key_pair = Keypair::new(&secp, &mut rand::thread_rng());
    // Random unspendable XOnlyPublicKey provided for internal key
    let (unspendable_pubkey, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let ctv_script = ctv_script(ctv_hash);

    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, ctv_script)
        .unwrap()
        .finalize(&secp, unspendable_pubkey)
        .unwrap();

    Ok(taproot_spend_info)
}

fn calc_ctv_hash(outputs: &[TxOut], timeout: Option<u32>) -> [u8; 32] {
    let mut buffer = Vec::new();
    buffer.extend(3_i32.to_le_bytes()); // version
    buffer.extend(0_i32.to_le_bytes()); // locktime
    buffer.extend(1_u32.to_le_bytes()); // inputs len

    let seq = if let Some(timeout_value) = timeout {
        sha256::Hash::hash(&Sequence(timeout_value).0.to_le_bytes())
    } else {
        sha256::Hash::hash(&Sequence(0xfffffffd).0.to_le_bytes())
    };
    buffer.extend(seq.to_byte_array()); // sequences

    let outputs_len = outputs.len() as u32;
    buffer.extend(outputs_len.to_le_bytes()); // outputs len

    let mut output_bytes: Vec<u8> = Vec::new();
    for o in outputs {
        o.consensus_encode(&mut output_bytes).unwrap();
    }
    buffer.extend(sha256::Hash::hash(&output_bytes).to_byte_array()); // outputs hash

    buffer.extend(0_u32.to_le_bytes()); // inputs index

    let hash = sha256::Hash::hash(&buffer);
    hash.to_byte_array()
}

fn ctv_script(ctv_hash: [u8; 32]) -> ScriptBuf {
    Builder::new()
        .push_slice(ctv_hash)
        .push_opcode(OP_CTV)
        .push_opcode(OP_DROP)
        .into_script()
}

fn spend_ctv(
    mut unsigned_tx: Transaction,
    taproot_spend_info: TaprootSpendInfo,
    ctv_hash: [u8; 32],
) -> Transaction {
    let ctv_script = ctv_script(ctv_hash);

    for input in unsigned_tx.input.iter_mut() {
        let script_ver = (ctv_script.clone(), LeafVersion::TapScript);
        let ctrl_block = taproot_spend_info.control_block(&script_ver).unwrap();

        input.witness.push(ctv_hash);
        input.witness.push(script_ver.0.into_bytes());
        input.witness.push(ctrl_block.serialize());
    }
    unsigned_tx
}
