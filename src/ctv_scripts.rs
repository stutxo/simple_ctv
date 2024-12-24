use bitcoin::{
    consensus::Encodable,
    hashes::{sha256, Hash},
    io::Error,
    key::{Keypair, Secp256k1},
    opcodes::all::{OP_DROP, OP_NOP4},
    script::Builder,
    taproot::{LeafVersion, TaprootBuilder, TaprootSpendInfo},
    Opcode, ScriptBuf, Sequence, Transaction, TxOut, XOnlyPublicKey,
};

const OP_CTV: Opcode = OP_NOP4;

pub fn ctv_script(ctv_hash: [u8; 32]) -> ScriptBuf {
    Builder::new()
        .push_slice(ctv_hash)
        .push_opcode(OP_CTV)
        .into_script()
}

pub fn calc_ctv_hash(outputs: &[TxOut], timeout: Option<u32>) -> [u8; 32] {
    let mut buffer = Vec::new();
    buffer.extend(3_i32.to_le_bytes()); // version
    buffer.extend(0_i32.to_le_bytes()); // locktime
    buffer.extend(1_u32.to_le_bytes()); // inputs len

    let seq = if let Some(timeout_value) = timeout {
        sha256::Hash::hash(&Sequence(timeout_value).0.to_le_bytes())
    } else {
        sha256::Hash::hash(&Sequence::ENABLE_RBF_NO_LOCKTIME.0.to_le_bytes())
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

pub fn create_ctv_address(ctv_hash: [u8; 32]) -> Result<TaprootSpendInfo, Error> {
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

pub fn spend_ctv(
    mut unsigned_tx: Transaction,
    taproot_spend_info: TaprootSpendInfo,
    ctv_hash: [u8; 32],
) -> Transaction {
    let ctv_script = ctv_script(ctv_hash);

    for input in unsigned_tx.input.iter_mut() {
        let script_ver = (ctv_script.clone(), LeafVersion::TapScript);
        let ctrl_block = taproot_spend_info.control_block(&script_ver).unwrap();

        input.witness.push(script_ver.0.into_bytes());
        input.witness.push(ctrl_block.serialize());
    }
    unsigned_tx
}
