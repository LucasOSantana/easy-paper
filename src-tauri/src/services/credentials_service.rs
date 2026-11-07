//! API keys of online sources, kept in Windows Credential Manager instead of
//! the plain-text config file. Keys never leave the Rust side: the frontend
//! only learns whether a key is set.

use keyring::Entry;

use crate::models::StockProvider;

const SERVICE: &str = "easy-paper";

fn entry(provider: StockProvider) -> Result<Entry, String> {
    Entry::new(SERVICE, provider.id())
        .map_err(|e| format!("Cannot access Windows Credential Manager: {}", e))
}

pub fn get_api_key(provider: StockProvider) -> Option<String> {
    entry(provider)
        .ok()?
        .get_password()
        .ok()
        .filter(|key| !key.trim().is_empty())
}

pub fn set_api_key(provider: StockProvider, key: &str) -> Result<(), String> {
    entry(provider)?
        .set_password(key)
        .map_err(|e| format!("Cannot save the key in Windows Credential Manager: {}", e))
}

pub fn delete_api_key(provider: StockProvider) -> Result<(), String> {
    match entry(provider)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Cannot remove the key from Windows Credential Manager: {}", e)),
    }
}
