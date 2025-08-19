use clap::Subcommand;

use crate::subcmd::{init::InitArgs, set_admin::SetAdminArgs};

mod init;
mod set_admin;

#[derive(Debug, Subcommand)]
pub enum Subcmd {
    Init(InitArgs),
    SetAdmin(SetAdminArgs),
}

impl Subcmd {
    pub async fn run(args: crate::Args) {
        match &args.subcmd {
            Self::Init(a) => a.run(&args).await,
            Self::SetAdmin(a) => a.run(&args).await,
        }
    }
}
