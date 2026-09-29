//! Load Indodax API credentials from a JSON file and environment variables.

use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// API credentials for the Indodax exchange.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Indodax API key.
    pub api_key: String,
    /// Indodax API secret.
    pub secret: String,
}

#[derive(Debug, Deserialize, Default)]
struct FileCredentials {
    #[serde(default, rename = "apiKey")]
    api_key: String,
    #[serde(default)]
    secret: String,
}

/// Loads credentials from a JSON file, then applies non-empty environment overrides.
///
/// Returns [`ExitCode`] from the process when a required credential is missing
/// after merging file and environment values. Names the missing variable on stderr.
pub fn load_credentials(config_path: Option<&str>) -> Result<Credentials, ExitCode> {
    let path = Path::new(config_path.unwrap_or("config.json"));
    let mut file_api_key = String::new();
    let mut file_secret = String::new();

    if path.is_file() {
        let raw = fs::read_to_string(path).map_err(|err| {
            eprintln!("{err}");
            ExitCode::from(1)
        })?;
        let parsed: FileCredentials = serde_json::from_str(&raw).map_err(|err| {
            eprintln!("{err}");
            ExitCode::from(1)
        })?;
        file_api_key = parsed.api_key;
        file_secret = parsed.secret;
    }

    let env_api_key = env::var("INDODAX_APIKEY").unwrap_or_default();
    let env_secret = env::var("INDODAX_SECRET").unwrap_or_default();

    let api_key = if !env_api_key.is_empty() {
        env_api_key
    } else {
        file_api_key
    };
    let secret = if !env_secret.is_empty() {
        env_secret
    } else {
        file_secret
    };

    if api_key.is_empty() {
        eprintln!("missing INDODAX_APIKEY");
        return Err(ExitCode::from(2));
    }
    if secret.is_empty() {
        eprintln!("missing INDODAX_SECRET");
        return Err(ExitCode::from(2));
    }

    Ok(Credentials { api_key, secret })
}
