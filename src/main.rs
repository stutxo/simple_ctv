use bitcoincore_rpc::RpcApi;
use config::NetworkConfig;

use ctv_scripts::{calc_ctv_hash, create_ctv_address, spend_ctv};
use rpc_helper::{get_vout, get_vout_after_confirmation, send_funding_transaction};
use std::str::FromStr;

use bitcoin::{
    absolute,
    consensus::encode::serialize_hex,
    opcodes::all::OP_RETURN,
    script::Builder,
    transaction::{self},
    Address, Amount, OutPoint, Sequence, Transaction, TxIn, TxOut,
};

mod config;
mod ctv_scripts;
mod rpc_helper;

//the amount you want to spend in the ctv transaction
const CTV_SPEND_AMOUNT: Amount = Amount::from_sat(1577);
//the fee you want the cpfp to pay
const CPFP_FEE_AMOUNT: Amount = Amount::from_sat(69180);
//this is the min dust amount required to anchor the transaction
const ANCHOR_AMOUNT: Amount = Amount::from_sat(240);

fn main() {
    let config = NetworkConfig::new();
    let rpc = config.bitcoin_rpc();

    let ctv_spend_to_address = rpc
        .get_new_address(None, None)
        .unwrap()
        .require_network(config.network)
        .unwrap();

    println!("CTV target address: {}", ctv_spend_to_address);

    let anchor_addr = Address::from_str(config.fee_anchor_addr)
        .unwrap()
        .require_network(config.network)
        .unwrap();

    let op_return_script = Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(b"\xF0\x9F\xA5\xAA \xe2\x9a\x93 \xF0\x9F\xA5\xAA")
        .into_script();

    let ctv_tx_out = [
        TxOut {
            value: CTV_SPEND_AMOUNT - ANCHOR_AMOUNT,
            script_pubkey: ctv_spend_to_address.script_pubkey(),
        },
        TxOut {
            value: ANCHOR_AMOUNT,
            script_pubkey: anchor_addr.script_pubkey(),
        },
        TxOut {
            value: Amount::from_sat(0),
            script_pubkey: op_return_script,
        },
    ];

    //calculate ctv hash
    let ctv_hash = calc_ctv_hash(&ctv_tx_out, None);

    // create ctv contract address
    let ctv_tr_spend_info = create_ctv_address(ctv_hash).unwrap();
    let ctv_contract_address =
        Address::p2tr_tweaked(ctv_tr_spend_info.output_key(), config.network);
    println!("CTV contract address: {}", ctv_contract_address);

    #[cfg(feature = "regtest")]
    if rpc.get_balance(None, None).unwrap() < CTV_SPEND_AMOUNT + CPFP_FEE_AMOUNT {
        let _ = rpc.generate_to_address(101, &ctv_spend_to_address);
    }

    println!("Funding ctv contract address...");
    let ctv_funding_txid = send_funding_transaction(&rpc, &ctv_contract_address, CTV_SPEND_AMOUNT);

    //this is to create a txid that we can spend later as the cpfp transaction.
    //could generate a new address for this but im just going to use the same one as the ctv spend for now
    println!("Funding cpfp address...");
    let cpfp_funding_txid = send_funding_transaction(&rpc, &ctv_spend_to_address, CPFP_FEE_AMOUNT);

    #[cfg(feature = "regtest")]
    let _ = rpc.generate_to_address(1, &ctv_spend_to_address);

    //we have to wait for the funding transaction to be confirmed to pay to anchor
    let ctv_vout = get_vout_after_confirmation(&rpc, ctv_funding_txid, CTV_SPEND_AMOUNT);

    let inputs = vec![TxIn {
        previous_output: OutPoint {
            txid: ctv_funding_txid,
            vout: ctv_vout,
        },
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
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
    println!("\nSpending ctv transaction...");
    println!("\nParent tx: {}", parent_serialized_tx);

    let parent_txid = rpc.send_raw_transaction(parent_serialized_tx).unwrap();

    println!("\nParent txid: {}", parent_txid);

    #[cfg(feature = "regtest")]
    let _ = rpc.generate_to_address(1, &ctv_spend_to_address);

    println!("\nSpending child transaction...");

    let anchor_vout = get_vout(&rpc, &parent_txid, ANCHOR_AMOUNT).unwrap();
    let cpfp_vout = get_vout(&rpc, &cpfp_funding_txid, CPFP_FEE_AMOUNT).unwrap();

    let op_return_script = Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(b"\xe2\x9a\x93 \xF0\x9F\xA5\xAA \xe2\x9a\x93")
        .into_script();

    let child_spend = Transaction {
        version: transaction::Version(3),
        lock_time: absolute::LockTime::ZERO,
        input: vec![
            TxIn {
                previous_output: OutPoint {
                    txid: parent_txid,
                    vout: anchor_vout,
                },
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                ..Default::default()
            },
            TxIn {
                previous_output: OutPoint {
                    txid: cpfp_funding_txid,
                    vout: cpfp_vout,
                },
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                ..Default::default()
            },
        ],
        output: vec![TxOut {
            value: Amount::from_sat(0),
            script_pubkey: op_return_script,
        }],
    };

    let child_serialized_tx = serialize_hex(&child_spend);

    println!("\nchild tx: {}", child_serialized_tx);

    let signed_child_tx = rpc
        .sign_raw_transaction_with_wallet(child_serialized_tx, None, None)
        .unwrap();

    let child_txid = rpc.send_raw_transaction(&signed_child_tx.hex).unwrap();

    println!("\nchild txid: {}", child_txid);

    #[cfg(feature = "regtest")]
    let _ = rpc.generate_to_address(1, &ctv_spend_to_address);
}
