use bitcoincore_rpc::RpcApi;
use core::time;
use std::thread;

pub fn send_funding_transaction(
    rpc: &bitcoincore_rpc::Client,
    address: &bitcoin::Address,
    amount: bitcoin::Amount,
) -> bitcoin::Txid {
    let txid_result = rpc.send_to_address(address, amount, None, None, None, None, None, None);

    match txid_result {
        Ok(txid) => {
            println!("Funding transaction sent: {}", txid);
            txid
        }
        Err(e) => {
            eprintln!("Error sending funding transaction: {:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn get_vout_after_confirmation(
    rpc: &bitcoincore_rpc::Client,
    funding_txid: bitcoin::Txid,
    amount: bitcoin::Amount,
) -> u32 {
    loop {
        let transaction_info = rpc.get_transaction(&funding_txid, None).unwrap();
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

    get_vout(rpc, &funding_txid, amount).unwrap()
}

pub fn get_vout(
    rpc: &bitcoincore_rpc::Client,
    txid: &bitcoin::Txid,
    amount: bitcoin::Amount,
) -> Option<u32> {
    let transaction_details: bitcoin::Transaction = rpc
        .get_raw_transaction(txid, None)
        .expect("Failed to retrieve raw transaction");

    // Find the output index that matches the given amount
    for (index, output) in transaction_details.output.iter().enumerate() {
        if output.value == amount {
            return Some(index as u32);
        }
    }

    None
}
