use std::sync::Arc;

use solana_cli_config::{CONFIG_FILE, Config};

pub fn parse_solana_config_from_path(path: &str) -> Result<Arc<Config>, String> {
    let p = if path.is_empty() {
        CONFIG_FILE.as_ref().ok_or_else(|| {
            "Solana CONFIG_FILE could not identify the user's home directory".to_owned()
        })?
    } else {
        path
    };
    Ok(Arc::new(Config::load(p).map_err(|e| {
        format!("Failed to load solana config: {e}")
    })?))
}
