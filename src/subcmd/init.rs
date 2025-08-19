use clap::Args;
use inf1_pp_flatslab_core::{
    instructions::init::{INIT_IX_IS_SIGNER, INIT_IX_IS_WRITER, InitIxData, NewInitIxAccsBuilder},
    keys::SLAB_ID,
};
use solana_instruction::Instruction;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

use crate::{
    sscu::parse_signer,
    tx_utils::{handle_tx, keys_signer_writable_to_metas, to_signed_tx, with_auto_cb},
};

#[derive(Args, Debug)]
#[command(long_about = "Runs the initialize instruction")]
pub struct InitArgs;

impl InitArgs {
    pub async fn run(
        &self,
        crate::Args {
            config,
            send_mode,
            fee_cb,
            ..
        }: &crate::Args,
    ) {
        let rpc = RpcClient::new(config.json_rpc_url.to_owned());
        let signer = parse_signer(&config.keypair_path).unwrap();
        let signer_pk = signer.pubkey();

        let init_ix = Instruction::new_with_bytes(
            crate::PROGRAM_ID.into(),
            InitIxData::new().as_buf(),
            keys_signer_writable_to_metas(
                NewInitIxAccsBuilder::start()
                    .with_payer(signer_pk.to_bytes())
                    .with_slab(SLAB_ID)
                    .with_system_program([0u8; 32])
                    .build()
                    .0
                    .iter(),
                INIT_IX_IS_SIGNER.0.iter(),
                INIT_IX_IS_WRITER.0.iter(),
            ),
        );

        let ixs = with_auto_cb(vec![init_ix], &signer_pk, &rpc, *send_mode, *fee_cb).await;
        let tx = to_signed_tx(ixs, vec![&signer], &rpc).await;
        handle_tx(&rpc, *send_mode, &tx).await;
    }
}
