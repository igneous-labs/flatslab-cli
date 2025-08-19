use std::io;

use clap::Args;
use inf1_pp_flatslab_core::accounts::Slab;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

use crate::{
    slabcsv::{SlabCsvEntry, write_slab_csv},
    utils::fetch_slab_data,
};

#[derive(Args, Debug)]
#[command(long_about = "Outputs current price nanos as a csv to stdout")]
pub struct ViewPricesArgs;

impl ViewPricesArgs {
    pub async fn run(&self, crate::Args { config, .. }: &crate::Args) {
        let rpc = RpcClient::new(config.json_rpc_url.to_owned());

        let slab_d = fetch_slab_data(&rpc).await;
        let slab = Slab::of_acc_data(&slab_d).unwrap();

        write_slab_csv(
            io::stdout(),
            slab.entries().0.iter().map(|e| SlabCsvEntry::from(*e)),
        );
    }
}
