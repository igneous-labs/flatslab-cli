use clap::Args;
use inf1_pp_flatslab_core::accounts::Slab;
use solana_pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

use crate::utils::fetch_slab_data;

#[derive(Args, Debug)]
#[command(long_about = "Views the slab's current admin pubkey")]
pub struct ViewAdminArgs;

impl ViewAdminArgs {
    pub async fn run(&self, crate::Args { config, .. }: &crate::Args) {
        let rpc = RpcClient::new(config.json_rpc_url.to_owned());

        let slab_d = fetch_slab_data(&rpc).await;
        let slab = Slab::of_acc_data(&slab_d).unwrap();

        println!("{}", Pubkey::new_from_array(*slab.admin()));
    }
}
