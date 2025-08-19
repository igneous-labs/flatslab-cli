use std::str::FromStr;

use solana_clap_utils::keypair::signer_from_path;
use solana_pubkey::Pubkey;
use solana_signer::{Signer, null_signer::NullSigner};

pub fn parse_signer(arg: &str) -> Result<Box<dyn Signer>, String> {
    match Pubkey::from_str(arg) {
        Ok(p) => Ok(Box::new(NullSigner::new(&p))),
        Err(_) => signer_from_path(&Default::default(), arg, "signer", &mut None)
            .map_err(|e| format!("Failed to load signer from solana config: {e}")),
    }
}

// Can't just use closure into `ValueParser::new()` because of lifetime issues with clap
pub fn parse_pubkey_from_src(arg: &str) -> Result<Pubkey, String> {
    parse_signer(arg).map(|s| s.pubkey())
}
