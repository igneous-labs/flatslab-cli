use clap::Subcommand;

use crate::subcmd::init::InitArgs;

mod init;

#[derive(Debug, Subcommand)]
pub enum Subcmd {
    Init(InitArgs),
}

impl Subcmd {
    pub async fn run(args: crate::Args) {
        match &args.subcmd {
            Subcmd::Init(a) => a.run(&args).await,
        }
    }
}
