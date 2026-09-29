//! Build a sandbox-configured Indodax CCXT exchange client.

use crate::config::Credentials;
use ccxt::{Config, Indodax, Value};

/// Public REST host for the Indodax sandbox.
pub const PUBLIC_URL: &str = "https://d-idx.com";
/// TAPI v1 private host for the Indodax sandbox.
pub const PRIVATE_URL: &str = "https://d-idx.com/tapi";
/// TAPI v2 sandbox host.
pub const SANDBOX_URL: &str = "https://api.d-idx.com";

/// Creates an Indodax client pointed at the sandbox hosts.
///
/// Does not call [`Config::sandbox`] or `set_sandbox_mode`.
pub fn create_exchange(credentials: &Credentials, verbose: bool) -> Indodax {
    let mut config_value = Config::new()
        .api_key(&credentials.api_key)
        .secret(&credentials.secret)
        .enable_rate_limit(true)
        .verbose(verbose)
        .option_str("tapiVersion", "2")
        .option_str("sandboxUrl", SANDBOX_URL)
        .into_value();

    let mut api = ccxt::value::HashMap::new();
    api.insert("public".into(), Value::from(PUBLIC_URL));
    api.insert("private".into(), Value::from(PRIVATE_URL));
    let mut urls = ccxt::value::HashMap::new();
    urls.insert("api".into(), Value::Map(api));

    match &mut config_value {
        Value::Dict(entries) => {
            let map = std::sync::Arc::make_mut(entries);
            map.insert("urls".into(), Value::Map(urls));
        }
        _ => {
            let mut top = ccxt::value::HashMap::new();
            top.insert("urls".into(), Value::Map(urls));
            config_value = Value::Map(top);
        }
    }

    Indodax::new(Some(config_value))
}
