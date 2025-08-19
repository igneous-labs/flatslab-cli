use std::sync::Arc;

use clap::{Parser, builder::ValueParser};
use solana_cli_config::Config;
use tokio::runtime::Runtime;

use crate::{
    sscu::{TxSendMode, parse_solana_config_from_path},
    subcmd::Subcmd,
};

pub use inf1_pp_flatslab_core::ID as PROGRAM_ID;

mod sscu;
mod subcmd;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Args {
    #[arg(
        long,
        short,
        default_value = "",
        value_parser = ValueParser::new(parse_solana_config_from_path)
    )]
    pub config: Arc<Config>, // use Arc<> to workaround clap requiring field to be clone

    #[arg(
        long,
        short,
        default_value_t = TxSendMode::default(),
        value_enum,
    )]
    pub send_mode: TxSendMode,

    #[arg(
        long,
        short,
        help = "ComputeBudget fees to pay, in lamports",
        default_value_t = 1
    )]
    pub fee_cb: u64,

    #[command(subcommand)]
    pub subcmd: Subcmd,
}

fn main() {
    let args = Args::parse();
    let rt = Runtime::new().unwrap();
    rt.block_on(Subcmd::run(args));
}
