use std::{process::exit, sync::Arc};

use clap::Args;
use inf1_pp_flatslab_core::{
    accounts::Slab,
    instructions::admin::{
        remove_lst::{
            NewRemoveLstIxAccsBuilder, REMOVE_LST_IX_IS_SIGNER, REMOVE_LST_IX_IS_WRITER,
            RemoveLstIxData,
        },
        set_lst_fee::{
            NewSetLstFeeIxAccsBuilder, SET_LST_FEE_IX_IS_SIGNER, SET_LST_FEE_IX_IS_WRITER,
            SetLstFeeIxArgs, SetLstFeeIxData,
        },
    },
    keys::SLAB_ID,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

use crate::{
    slabcsv::{SlabCsvEntry, read_slab_csv_file},
    sscu::parse_signer,
    utils::{
        fetch_slab_data, handle_tx, keys_signer_writable_to_metas, to_signed_tx, with_auto_cb,
    },
};

const MAX_SET_LST_FEE_IX_PER_TX: usize = 17;
const MAX_REMOVE_LST_IX_PER_TX: usize = 18;

#[derive(Args, Debug)]
#[command(long_about = "Syncs price entries with the slab onchain")]
pub struct SyncPricesArgs {
    #[arg(
        long,
        short,
        help = "If flag set, deletes entries that are missing from the csv from onchain. Otherwise, ignores them.",
        default_value_t = false
    )]
    pub remove: bool,

    #[arg(
        long,
        short,
        help = "If flag set, skip verification step that verifies that no (inp, out) pair results in a negative fee before running the operation.",
        default_value_t = false
    )]
    pub no_verify: bool,

    #[arg(
        long,
        short,
        help = "Path to admin keypair signer. Defaults to config wallet if not set."
    )]
    pub admin: Option<Arc<str>>,

    #[arg(help = "Path to slab prices csv file", default_value_t = {"slab.csv".into()})]
    pub csv: Arc<str>,
}

impl SyncPricesArgs {
    pub async fn run(
        &self,
        crate::Args {
            config,
            send_mode,
            fee_cb,
            ..
        }: &crate::Args,
    ) {
        let Self {
            remove,
            no_verify,
            csv,
            admin,
        } = self;

        let entries = read_slab_csv_file(csv.as_ref());
        let rpc = RpcClient::new(config.json_rpc_url.to_owned());
        let payer = parse_signer(&config.keypair_path).unwrap();
        let payer_pk = payer.pubkey();
        let admin_opt = admin.as_ref().map(|s| parse_signer(s).unwrap());
        let admin = admin_opt.as_ref().unwrap_or(&payer);

        if !*no_verify {
            let inp_key = |e: &&SlabCsvEntry| e.inp;
            let out_key = |e: &&SlabCsvEntry| e.out;

            let [s_inp, s_out] = [inp_key, out_key].map(|key_fn| entries.iter().min_by_key(key_fn));

            match (s_inp, s_out) {
                (Some(s_inp), Some(s_out)) => {
                    if s_inp.inp + s_out.out < 0 {
                        eprintln!("inp={}, out={} results in fee < 0", s_inp.mint, s_out.mint);
                        exit(-1);
                    }
                }
                _empty_list => (),
            }
        }

        let slab_d = fetch_slab_data(&rpc).await;
        let slab = Slab::of_acc_data(&slab_d).unwrap();
        let curr_entries = slab.entries();

        let (msg, ixs): (String, Vec<Instruction>) =
            entries
                .iter()
                .fold((String::new(), vec![]), |(mut msg, mut ixs), e| {
                    if let Ok(curr) = curr_entries.find_by_mint(e.mint.as_array()) {
                        if curr.inp_fee_nanos() == e.inp && curr.out_fee_nanos() == e.out {
                            return (msg, ixs);
                        }
                    }

                    msg.extend(setting_msg(e).drain(..));
                    msg.push('\n');

                    let set_lst_ix = Instruction::new_with_bytes(
                        crate::PROGRAM_ID.into(),
                        SetLstFeeIxData::new(SetLstFeeIxArgs {
                            inp_fee_nanos: e.inp,
                            out_fee_nanos: e.out,
                        })
                        .as_buf(),
                        keys_signer_writable_to_metas(
                            NewSetLstFeeIxAccsBuilder::start()
                                .with_admin(slab.admin())
                                .with_mint(e.mint.as_array())
                                .with_payer(payer_pk.as_array())
                                .with_slab(&SLAB_ID)
                                .with_system_program(&[0u8; 32])
                                .build()
                                .0
                                .iter()
                                .copied(),
                            SET_LST_FEE_IX_IS_SIGNER.0.iter(),
                            SET_LST_FEE_IX_IS_WRITER.0.iter(),
                        ),
                    );
                    ixs.push(set_lst_ix);

                    (msg, ixs)
                });

        if !msg.is_empty() {
            eprintln!("Setting:");
            eprint!("{msg}");
        }

        for batch in ixs.chunks(MAX_SET_LST_FEE_IX_PER_TX) {
            let ixs = with_auto_cb(batch.into(), &payer_pk, &rpc, *send_mode, *fee_cb).await;
            let tx = to_signed_tx(ixs, vec![&payer, admin], &rpc).await;
            handle_tx(&rpc, *send_mode, &tx).await;
        }

        if !*remove {
            return;
        }

        // Remove

        let (msg, ixs): (String, Vec<Instruction>) =
            curr_entries
                .0
                .iter()
                .fold((String::new(), vec![]), |(mut msg, mut ixs), curr| {
                    if entries.iter().any(|e| e.mint.as_array() == curr.mint()) {
                        return (msg, ixs);
                    }

                    msg.extend(Pubkey::new_from_array(*curr.mint()).to_string().drain(..));
                    msg.push('\n');

                    let remove_ix = Instruction::new_with_bytes(
                        crate::PROGRAM_ID.into(),
                        RemoveLstIxData::new().as_buf(),
                        keys_signer_writable_to_metas(
                            NewRemoveLstIxAccsBuilder::start()
                                .with_admin(slab.admin())
                                .with_mint(curr.mint())
                                .with_refund_rent_to(payer_pk.as_array())
                                .with_slab(&SLAB_ID)
                                .build()
                                .0
                                .iter()
                                .copied(),
                            REMOVE_LST_IX_IS_SIGNER.0.iter(),
                            REMOVE_LST_IX_IS_WRITER.0.iter(),
                        ),
                    );
                    ixs.push(remove_ix);

                    (msg, ixs)
                });

        if !msg.is_empty() {
            eprintln!("Removing:");
            eprint!("{msg}");
        }

        for batch in ixs.chunks(MAX_REMOVE_LST_IX_PER_TX) {
            let ixs = with_auto_cb(batch.into(), &payer_pk, &rpc, *send_mode, *fee_cb).await;
            let tx = to_signed_tx(ixs, vec![&payer, admin], &rpc).await;
            handle_tx(&rpc, *send_mode, &tx).await;
        }
    }
}

fn setting_msg(entry: &SlabCsvEntry) -> String {
    format!("{} inp={} out={}", entry.mint, entry.inp, entry.out)
}

#[cfg(test)]
mod tests {
    use solana_compute_budget_interface::ComputeBudgetInstruction;
    use solana_message::{VersionedMessage, v0::Message};
    use solana_signature::Signature;
    use solana_transaction::versioned::VersionedTransaction;

    use super::*;

    pub fn to_mocked_tx(ixs: Vec<Instruction>, payer_pk: &Pubkey) -> VersionedTransaction {
        let message = VersionedMessage::V0(
            Message::try_compile(payer_pk, &ixs, &[], Default::default()).unwrap(),
        );
        VersionedTransaction {
            signatures: vec![Signature::default(); message.header().num_required_signatures.into()],
            message,
        }
    }

    #[test]
    fn verify_max_set_lst_fee_ix_per_tx() {
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let ixs: Vec<_> = [
            ComputeBudgetInstruction::set_compute_unit_limit(0),
            ComputeBudgetInstruction::set_compute_unit_price(0),
        ]
        .into_iter()
        .chain((0..MAX_SET_LST_FEE_IX_PER_TX).map(|_| {
            Instruction::new_with_bytes(
                crate::PROGRAM_ID.into(),
                SetLstFeeIxData::new(SetLstFeeIxArgs {
                    inp_fee_nanos: 0,
                    out_fee_nanos: 0,
                })
                .as_buf(),
                keys_signer_writable_to_metas(
                    NewSetLstFeeIxAccsBuilder::start()
                        .with_admin(admin.as_array())
                        .with_mint(Pubkey::new_unique().as_array())
                        .with_payer(payer.as_array())
                        .with_slab(&SLAB_ID)
                        .with_system_program(&[0u8; 32])
                        .build()
                        .0
                        .iter()
                        .copied(),
                    SET_LST_FEE_IX_IS_SIGNER.0.iter(),
                    SET_LST_FEE_IX_IS_WRITER.0.iter(),
                ),
            )
        }))
        .collect();

        let tx = to_mocked_tx(ixs, &payer);

        let tx_byte_len = bincode::serialize(&tx).unwrap().len();
        //eprintln!("{tx_byte_len}");
        assert!(tx_byte_len < 1232, "{tx_byte_len}");
    }

    #[test]
    fn verify_max_remove_lst_ix_per_tx() {
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let ixs: Vec<_> = [
            ComputeBudgetInstruction::set_compute_unit_limit(0),
            ComputeBudgetInstruction::set_compute_unit_price(0),
        ]
        .into_iter()
        .chain((0..MAX_REMOVE_LST_IX_PER_TX).map(|_| {
            Instruction::new_with_bytes(
                crate::PROGRAM_ID.into(),
                SetLstFeeIxData::new(SetLstFeeIxArgs {
                    inp_fee_nanos: 0,
                    out_fee_nanos: 0,
                })
                .as_buf(),
                keys_signer_writable_to_metas(
                    NewRemoveLstIxAccsBuilder::start()
                        .with_admin(admin.as_array())
                        .with_mint(Pubkey::new_unique().as_array())
                        .with_refund_rent_to(payer.as_array())
                        .with_slab(&SLAB_ID)
                        .build()
                        .0
                        .iter()
                        .copied(),
                    REMOVE_LST_IX_IS_SIGNER.0.iter(),
                    REMOVE_LST_IX_IS_WRITER.0.iter(),
                ),
            )
        }))
        .collect();

        let tx = to_mocked_tx(ixs, &payer);

        let tx_byte_len = bincode::serialize(&tx).unwrap().len();
        //eprintln!("{tx_byte_len}");
        assert!(tx_byte_len < 1232, "{tx_byte_len}");
    }
}
