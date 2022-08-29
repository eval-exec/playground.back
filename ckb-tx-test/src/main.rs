use ckb_hash::{blake2b_256, new_blake2b};
use ckb_jsonrpc_types::OutputsValidator::Passthrough;
use ckb_jsonrpc_types::{BlockNumber, CellWithStatus};
use ckb_sdk::constants::SECP_SIGNATURE_SIZE;
use ckb_sdk::traits::DefaultCellDepResolver;
use ckb_sdk::unlock::MultisigConfig;
use ckb_sdk::util::serialize_signature;
use ckb_sdk::{tx_builder, types, Address, CkbRpcClient, HumanCapacity, SECP256K1};
use ckb_types::bytes::{Bytes, BytesMut};
use ckb_types::core::{BlockView, Capacity, TransactionBuilder, TransactionView};
use ckb_types::packed::{
    CellDep, CellInput, CellOutput, OutPoint, Script, Transaction, WitnessArgs,
};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{constants, h256, packed, H160, H256};
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};

pub type SignerFn = Box<
    dyn FnMut(
        &HashSet<H160>,
        &H256,
        &ckb_jsonrpc_types::Transaction,
    ) -> Result<Option<[u8; 65]>, String>,
>;
fn get_privkey(path: &str) -> Result<secp256k1::SecretKey, String> {
    let mut content = String::new();
    let mut file = fs::File::open(&path).map_err(|err| err.to_string())?;
    file.read_to_string(&mut content)
        .map_err(|err| err.to_string())?;
    let privkey_string: String = content
        .split_whitespace()
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "File is empty".to_string())?;
    let data: H256 = H256::from_str(privkey_string.as_str()).map_err(|err| err.to_string())?;
    secp256k1::SecretKey::from_slice(data.as_bytes())
        .map_err(|err| format!("Invalid secp256k1 secret key format, error: {}", err))
}

pub fn get_privkey_signer(privkey: secp256k1::SecretKey) -> SignerFn {
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let lock_arg = H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20])
        .expect("Generate hash(H160) from pubkey failed");
    Box::new(
        move |lock_args: &HashSet<H160>, message: &H256, _tx: &ckb_jsonrpc_types::Transaction| {
            if lock_args.contains(&lock_arg) {
                if message == &h256!("0x0") {
                    Ok(Some([0u8; 65]))
                } else {
                    let message = secp256k1::Message::from_slice(message.as_bytes())
                        .expect("Convert to secp256k1 message failed");
                    let signature = SECP256K1.sign_recoverable(&message, &privkey);
                    Ok(Some(serialize_signature(&signature)))
                }
            } else {
                Ok(None)
            }
        },
    )
}

#[allow(unused_imports)]
fn main() {
    let mut client = CkbRpcClient::new("https://testnet.ckbapp.dev/rpc");

    let view = TransactionBuilder::default().build();
    let input_tx_hash: H256 =
        h256!("0xc1e0d2842d8349f253014e46d109a959072c01eca6d1bbd6366ed0bb47a1fcfb");
    let input_index: u32 = 0;
    let out_capacity = "9999";
    let to_sign_hash_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvxejkz084ruhl7acy76spycdasjgxxd8sewxu0e";
    let privkey_path = "/tmp/2.privkey-path";

    let previous_out_point = OutPoint::new_builder()
        .tx_hash(input_tx_hash.pack())
        .index(input_index.pack())
        .build();
    let genesis_block = client
        .get_block_by_number(ckb_jsonrpc_types::BlockNumber::from(0))
        .map_err(|err| err.to_string())
        .unwrap()
        .unwrap();
    let genesis_resolver = DefaultCellDepResolver::from_genesis(&genesis_block.into()).unwrap();

    let mut cell_deps: HashSet<CellDep> = HashSet::default();
    let cell_dep = genesis_resolver.sighash_dep().unwrap().0.clone();
    cell_deps.insert(cell_dep);

    let input = CellInput::new_builder()
        .previous_output(previous_out_point)
        .since(0_u64.pack())
        .build();

    let address = Address::from_str(to_sign_hash_address).unwrap();
    let payload = address.payload();
    let lock_script = Script::from(payload);

    let output = CellOutput::new_builder()
        .capacity(HumanCapacity::from_str(out_capacity).unwrap().pack())
        .lock(lock_script)
        .build();

    // build transaction
    let tx_view = TransactionBuilder::default()
        .build()
        .as_advanced_builder()
        .version(constants::TX_VERSION.pack())
        .cell_deps(cell_deps)
        .input(input.clone())
        .output(output)
        .output_data(Bytes::new().pack())
        .build();

    let privkey = get_privkey(privkey_path).unwrap();

    let mut signer_fn: SignerFn = get_privkey_signer(privkey);

    let lock = get_live_cell(&mut client, input.previous_output(), true)
        .unwrap()
        .0
        .lock();
    let lock_arg = lock.args().raw_data();
    let mut lock_args = HashSet::default();
    lock_args.insert(H160::from_slice(lock_arg.as_ref()).unwrap());

    let mut signature: Bytes = Bytes::default();
    let idxs = [0];

    // if signer_fn(&lock_args, &h256!("0x0"), &Transaction::default().into())
    //     .unwrap()
    //     .is_some()

    let mut witnesses: Vec<packed::Bytes> = vec![];
    witnesses.push(Bytes::new().pack());
    signature = build_signature(
        &tx_view,
        1,
        &idxs,
        &witnesses,
        None,
        |message: &H256, tx: &ckb_jsonrpc_types::Transaction| {
            signer_fn(&lock_args, message, tx).map(|sig| sig.unwrap())
        },
    )
    .unwrap();
    println!("signature:   {:?}", signature.clone());

    let init_witness = WitnessArgs::default();

    let mut witnesses: Vec<packed::Bytes> = vec![];
    witnesses.push(Bytes::new().pack());
    witnesses[0] = init_witness
        .as_builder()
        .lock(Some(signature).pack())
        .build()
        .as_bytes()
        .pack();

    let tx = tx_view
        .as_advanced_builder()
        .set_witnesses(witnesses)
        .build();
    // go signature: signature

    // send transaction

    let rpc_tx = ckb_jsonrpc_types::Transaction::from(tx.data());
    eprintln!(
        "[send transaction]:\n{}",
        serde_json::to_string_pretty(&rpc_tx).unwrap()
    );

    let send_tx_result = client
        .send_transaction(tx.data().into(), Some(Passthrough))
        .unwrap();
    println!("{:#?}", send_tx_result.to_string());
}

pub fn get_live_cell(
    client: &mut CkbRpcClient,
    out_point: OutPoint,
    with_data: bool,
) -> Result<(CellOutput, Bytes), String> {
    let cell = client
        .get_live_cell(out_point.clone().into(), with_data)
        .unwrap();
    if cell.status != "live" {
        return Err(format!(
            "Invalid cell status: {}, out_point: {}",
            cell.status, out_point
        ));
    }
    let cell_status = cell.status.clone();
    cell.cell
        .map(|cell| {
            (
                cell.output.into(),
                cell.data
                    .map(|data| data.content.into_bytes())
                    .unwrap_or_default(),
            )
        })
        .ok_or_else(|| {
            format!(
                "Invalid input cell, status: {}, out_point: {}",
                cell_status, out_point
            )
        })
}

pub fn build_signature<
    S: FnMut(&H256, &ckb_jsonrpc_types::Transaction) -> Result<[u8; SECP_SIGNATURE_SIZE], String>,
>(
    tx: &TransactionView,
    input_size: usize,
    input_group_idxs: &[usize],
    witnesses: &[packed::Bytes],
    multisig_config_opt: Option<&MultisigConfig>,
    mut signer: S,
) -> Result<Bytes, String> {
    let init_witness_idx = input_group_idxs[0];
    let init_witness = if witnesses[init_witness_idx].raw_data().is_empty() {
        WitnessArgs::default()
    } else {
        WitnessArgs::from_slice(witnesses[init_witness_idx].raw_data().as_ref())
            .map_err(|err| err.to_string())?
    };

    let init_witness = if let Some(multisig_config) = multisig_config_opt {
        let lock_without_sig = {
            let sig_len = (multisig_config.threshold() as usize) * SECP_SIGNATURE_SIZE;
            let mut data = BytesMut::from(&multisig_config.to_witness_data()[..]);
            data.extend_from_slice(vec![0u8; sig_len].as_slice());
            data.freeze()
        };
        init_witness
            .as_builder()
            .lock(Some(lock_without_sig).pack())
            .build()
    } else {
        init_witness
            .as_builder()
            .lock(Some(Bytes::from(vec![0u8; SECP_SIGNATURE_SIZE])).pack())
            .build()
    };

    let mut blake2b = new_blake2b();
    blake2b.update(tx.hash().as_slice());
    blake2b.update(&(init_witness.as_bytes().len() as u64).to_le_bytes());
    blake2b.update(&init_witness.as_bytes());
    for idx in input_group_idxs.iter().skip(1).cloned() {
        let other_witness: &packed::Bytes = &witnesses[idx];
        blake2b.update(&(other_witness.len() as u64).to_le_bytes());
        blake2b.update(&other_witness.raw_data());
    }
    for outter_witness in &witnesses[input_size..witnesses.len()] {
        blake2b.update(&(outter_witness.len() as u64).to_le_bytes());
        blake2b.update(&outter_witness.raw_data());
    }
    let mut message = [0u8; 32];
    blake2b.finalize(&mut message);
    let message = H256::from(message);
    let mut new_witnesses = witnesses.to_vec();
    new_witnesses[init_witness_idx] = init_witness.as_bytes().pack();
    let new_tx = tx
        .as_advanced_builder()
        .set_witnesses(new_witnesses)
        .build();
    signer(&message, &new_tx.data().into()).map(|data| Bytes::from(data.to_vec()))
}
