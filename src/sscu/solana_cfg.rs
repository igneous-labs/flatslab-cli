use std::{io, sync::Arc};

use solana_cli_config::{CONFIG_FILE, Config};

pub fn parse_solana_config_from_path(path: &str) -> Result<Arc<Config>, io::Error> {
    let p = if path.is_empty() {
        CONFIG_FILE.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Solana CONFIG_FILE could not identify the user's home directory",
            )
        })?
    } else {
        path
    };
    Ok(Arc::new(Config::load(p)?))
}
