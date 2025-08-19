use data_encoding::BASE64;
use inf1_pp_flatslab_core::keys::SLAB_ID;
use solana_commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::{VersionedMessage, v0::Message};
use solana_pubkey::Pubkey;
use solana_rpc_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction};
use solana_rpc_client_types::config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use solana_transaction_status_client_types::UiTransactionEncoding;

use crate::sscu::TxSendMode;

pub fn keys_signer_writable_to_metas<'a>(
    keys: impl Iterator<Item = &'a [u8; 32]>,
    signer: impl Iterator<Item = &'a bool>,
    writable: impl Iterator<Item = &'a bool>,
) -> Vec<AccountMeta> {
    keys.zip(signer)
        .zip(writable)
        .map(|((key, signer), writable)| AccountMeta {
            pubkey: Pubkey::new_from_array(*key),
            is_signer: *signer,
            is_writable: *writable,
        })
        .collect()
}

/// Does nothing if `TxSendMode::DumpMsg`
pub async fn with_auto_cb(
    mut ixs: Vec<Instruction>,
    payer_pk: &Pubkey,
    rpc: &RpcClient,
    tsm: TxSendMode,
    fee_cb: u64,
) -> Vec<Instruction> {
    match tsm {
        TxSendMode::Dump64 | TxSendMode::Dump58 => ixs,
        TxSendMode::SendActual | TxSendMode::SimOnly => {
            let result = rpc
                .simulate_transaction_with_config(&to_est_cu_sim_tx(payer_pk, &ixs), SIM_TX_CFG)
                .await
                .unwrap();
            let cus = result.value.units_consumed.unwrap();
            let cus = (cus as f64 * 1.01) as u64 + 300;
            let cu_price = (fee_cb * 1_000_000).div_ceil(cus);
            ixs.insert(
                0,
                ComputeBudgetInstruction::set_compute_unit_limit(cus.try_into().unwrap()),
            );
            ixs.insert(
                0,
                ComputeBudgetInstruction::set_compute_unit_price(cu_price),
            );
            ixs
        }
    }
}

fn to_est_cu_sim_tx(payer_pk: &Pubkey, ixs: &[Instruction]) -> VersionedTransaction {
    // must set CU limit else default 200k will be used and expense txs will fail sim
    let ixs: Vec<_> = core::iter::once(ComputeBudgetInstruction::set_compute_unit_limit(
        1_400_000, // per tx cu limit
    ))
    .chain(ixs.iter().cloned())
    .collect();
    let message =
        VersionedMessage::V0(Message::try_compile(payer_pk, &ixs, &[], Hash::default()).unwrap());
    VersionedTransaction {
        signatures: vec![Signature::default(); message.header().num_required_signatures.into()],
        message,
    }
}

/// First signer in signers is transaction payer
pub async fn to_signed_tx(
    ixs: Vec<Instruction>,
    mut signers: Vec<&dyn Signer>,
    rpc: &RpcClient,
) -> VersionedTransaction {
    signers.sort_by_key(|s| s.pubkey());
    signers.dedup_by_key(|s| s.pubkey());

    let payer_pk = signers.first().unwrap().pubkey();

    let rbh = rpc.get_latest_blockhash().await.unwrap();

    VersionedTransaction::try_new(
        VersionedMessage::V0(Message::try_compile(&payer_pk, &ixs, &[], rbh).unwrap()),
        &signers,
    )
    .unwrap()
}

const SEND_TX_CFG: RpcSendTransactionConfig = RpcSendTransactionConfig {
    skip_preflight: false,
    preflight_commitment: Some(CommitmentLevel::Processed),
    encoding: Some(UiTransactionEncoding::Base64),
    max_retries: Some(0),
    min_context_slot: None,
};

pub const SIM_TX_CFG: RpcSimulateTransactionConfig = RpcSimulateTransactionConfig {
    sig_verify: false,
    replace_recent_blockhash: true,
    commitment: Some(CommitmentConfig::processed()),
    encoding: Some(UiTransactionEncoding::Base64),
    accounts: None,
    min_context_slot: None,
    inner_instructions: true,
};

pub async fn handle_tx(rpc: &RpcClient, send_mode: TxSendMode, tx: &impl SerializableTransaction) {
    match send_mode {
        TxSendMode::SendActual => {
            let sig = rpc
                .send_and_confirm_transaction_with_spinner_and_config(
                    tx,
                    CommitmentConfig::processed(),
                    SEND_TX_CFG,
                )
                .await
                .unwrap();
            eprintln!("Signature:");
            eprintln!("{sig}");
        }
        TxSendMode::SimOnly => {
            let result = rpc
                .simulate_transaction_with_config(tx, SIM_TX_CFG)
                .await
                .unwrap();
            eprintln!("Simulate result:");
            eprintln!("{result:#?}");
        }
        TxSendMode::Dump64 => {
            println!("{}", BASE64.encode(&bincode::serialize(tx).unwrap()))
        }
        TxSendMode::Dump58 => {
            println!(
                "{}",
                bs58::encode(&bincode::serialize(tx).unwrap()).into_string()
            )
        }
    }
}

pub async fn fetch_slab_data(rpc: &RpcClient) -> Vec<u8> {
    rpc.get_account_data(&SLAB_ID.into()).await.unwrap()
}
