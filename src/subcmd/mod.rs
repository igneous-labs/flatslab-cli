use clap::Subcommand;

use crate::subcmd::{
    init::InitArgs, set_admin::SetAdminArgs, sync_prices::SyncPricesArgs,
    view_prices::ViewPricesArgs,
};

mod init;
mod set_admin;
mod sync_prices;
mod view_prices;

#[derive(Debug, Subcommand)]
pub enum Subcmd {
    Init(InitArgs),
    SetAdmin(SetAdminArgs),
    SyncPrices(SyncPricesArgs),
    ViewPrices(ViewPricesArgs),
}

impl Subcmd {
    pub async fn run(args: crate::Args) {
        match &args.subcmd {
            Self::Init(a) => a.run(&args).await,
            Self::SetAdmin(a) => a.run(&args).await,
            Self::SyncPrices(a) => a.run(&args).await,
            Self::ViewPrices(a) => a.run(&args).await,
        }
    }
}
