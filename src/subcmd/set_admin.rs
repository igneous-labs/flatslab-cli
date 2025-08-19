use std::sync::Arc;

use clap::{Args, builder::ValueParser};
use inf1_pp_flatslab_core::{
    accounts::Slab,
    instructions::admin::set_admin::{
        NewSetAdminIxAccsBuilder, SET_ADMIN_IX_IS_SIGNER, SET_ADMIN_IX_IS_WRITER, SetAdminIxData,
    },
    keys::SLAB_ID,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_signer::Signer;

use crate::{
    sscu::{parse_pubkey_from_src, parse_signer},
    utils::{
        fetch_slab_data, handle_tx, keys_signer_writable_to_metas, to_signed_tx, with_auto_cb,
    },
};

#[derive(Args, Debug)]
#[command(long_about = "Runs the set-admin instruction")]
pub struct SetAdminArgs {
    #[arg(
        long,
        short,
        help = "Path to admin keypair signer. Defaults to config wallet if not set."
    )]
    pub admin: Option<Arc<str>>,

    #[arg(
        value_parser = ValueParser::new(parse_pubkey_from_src)
    )]
    pub new_admin: Pubkey,
}

impl SetAdminArgs {
    pub async fn run(
        &self,
        crate::Args {
            config,
            send_mode,
            fee_cb,
            ..
        }: &crate::Args,
    ) {
        let Self { admin, new_admin } = self;
        let rpc = RpcClient::new(config.json_rpc_url.to_owned());
        let payer = parse_signer(&config.keypair_path).unwrap();
        let payer_pk = payer.pubkey();
        let admin_opt = admin.as_ref().map(|s| parse_signer(s).unwrap());
        let admin = admin_opt.as_ref().unwrap_or(&payer);

        let slab_d = fetch_slab_data(&rpc).await;
        let slab = Slab::of_acc_data(&slab_d).unwrap();

        let set_admin_ix = Instruction::new_with_bytes(
            crate::PROGRAM_ID.into(),
            SetAdminIxData::new().as_buf(),
            keys_signer_writable_to_metas(
                NewSetAdminIxAccsBuilder::start()
                    .with_current_admin(*slab.admin())
                    .with_new_admin(new_admin.to_bytes())
                    .with_slab(SLAB_ID)
                    .build()
                    .0
                    .iter(),
                SET_ADMIN_IX_IS_SIGNER.0.iter(),
                SET_ADMIN_IX_IS_WRITER.0.iter(),
            ),
        );

        let ixs = with_auto_cb(vec![set_admin_ix], &payer_pk, &rpc, *send_mode, *fee_cb).await;
        let tx = to_signed_tx(ixs, vec![&payer, &admin], &rpc).await;
        handle_tx(&rpc, *send_mode, &tx).await;
    }
}
