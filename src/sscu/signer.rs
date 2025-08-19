use std::{error::Error, str::FromStr};

use solana_clap_utils::keypair::signer_from_path;
use solana_pubkey::Pubkey;
use solana_signer::{Signer, null_signer::NullSigner};

pub fn parse_signer(arg: &str) -> Result<Box<dyn Signer>, Box<dyn Error + 'static>> {
    match Pubkey::from_str(arg) {
        Ok(p) => Ok(Box::new(NullSigner::new(&p))),
        Err(_) => signer_from_path(&Default::default(), arg, "signer", &mut None),
    }
}
