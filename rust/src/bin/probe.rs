//! Read-only V2/V1 sandbox smoke probe. Prints no secrets, addresses, tags, or balances.

use ccxt::exchanges::indodax::IndodaxCore;
use ccxt::runtime::call_typed;
use ccxt::value::HashMap;
use ccxt::{Config, Indodax, Params, Value};
use serde::Deserialize;
use std::env;
use std::fs;
use std::process::ExitCode;
use std::sync::Arc;

const SYMBOL: &str = "BTC/USDT";
const PUBLIC_URL: &str = "https://d-idx.com";
const PRIVATE_URL: &str = "https://d-idx.com/tapi";
const SANDBOX_URL: &str = "https://api.d-idx.com";

#[derive(Deserialize)]
struct FileConfig {
    #[serde(rename = "apiKey")]
    api_key: String,
    secret: String,
}

fn redact(text: &str, api_key: &str, secret: &str) -> String {
    let mut out = text.to_string();
    if !api_key.is_empty() {
        out = out.replace(api_key, "[REDACTED]");
    }
    if !secret.is_empty() {
        out = out.replace(secret, "[REDACTED]");
    }
    out
}

fn classify_transport(err: &str) -> Option<&'static str> {
    let lower = err.to_lowercase();
    if lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("deadline exceeded")
    {
        return Some("TIMEOUT");
    }
    if lower.contains("dns")
        || lower.contains("name or service not known")
        || lower.contains("nodename nor servname")
        || lower.contains("failed to lookup")
        || lower.contains("resolve")
    {
        return Some("DNS");
    }
    None
}

fn stop_transport(label: &str, err: &str) -> ExitCode {
    if let Some(kind) = classify_transport(err) {
        println!("{label} STOP {kind}: {err}");
        return ExitCode::from(3);
    }
    ExitCode::SUCCESS
}

fn url_no_query(value: &Value) -> String {
    let raw = match value {
        Value::Str(s) => s.to_string(),
        other => format!("{other:?}"),
    };
    raw.split('?').next().unwrap_or(&raw).to_string()
}

fn host_of(url: &str) -> String {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

fn path_of(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    match rest.find('/') {
        Some(idx) => rest[idx..].to_string(),
        None => "/".to_string(),
    }
}

fn dict_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Dict(map) => map.get(key),
        _ => None,
    }
}

fn num_of(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
        Value::Str(s) => s.to_string(),
        Value::Null => "null".to_string(),
        other => format!("{other:?}"),
    }
}

fn load_config(path: &str) -> Result<FileConfig, String> {
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

fn build_urls_value() -> Value {
    let mut api = HashMap::new();
    api.insert("public".into(), Value::from(PUBLIC_URL));
    api.insert("private".into(), Value::from(PRIVATE_URL));
    let mut urls = HashMap::new();
    urls.insert("api".into(), Value::Map(api));
    Value::Map(urls)
}

fn inject_urls(mut config_value: Value) -> Value {
    let urls = build_urls_value();
    match &mut config_value {
        Value::Dict(entries) => {
            let map = Arc::make_mut(entries);
            map.insert("urls".into(), urls);
        }
        _ => {
            let mut top = HashMap::new();
            top.insert("urls".into(), urls);
            config_value = Value::Map(top);
        }
    }
    config_value
}

fn build_v2(api_key: &str, secret: &str) -> Value {
    let config_value = Config::new()
        .api_key(api_key)
        .secret(secret)
        .enable_rate_limit(true)
        .verbose(false)
        .timeout_ms(30_000)
        .option_str("tapiVersion", "2")
        .option_str("sandboxUrl", SANDBOX_URL)
        .into_value();
    inject_urls(config_value)
}

fn build_v1(api_key: &str, secret: &str) -> Value {
    let config_value = Config::new()
        .api_key(api_key)
        .secret(secret)
        .enable_rate_limit(true)
        .verbose(false)
        .timeout_ms(30_000)
        .option_str("tapiVersion", "1")
        .into_value();
    inject_urls(config_value)
}

fn err_text(err: &impl ToString, api_key: &str, secret: &str) -> String {
    redact(&err.to_string(), api_key, secret)
}

fn print_trading_limits_entry(entry: &Value) {
    let precision = dict_get(entry, "precision").cloned().unwrap_or(Value::Null);
    let limits = dict_get(entry, "limits").cloned().unwrap_or(Value::Null);
    let amount = dict_get(&limits, "amount").cloned().unwrap_or(Value::Null);
    let cost = dict_get(&limits, "cost").cloned().unwrap_or(Value::Null);
    println!(
        "fetch_trading_limits BTC/USDT precision.amount={} precision.price={} amount.min={} cost.min={}",
        num_of(dict_get(&precision, "amount").unwrap_or(&Value::Null)),
        num_of(dict_get(&precision, "price").unwrap_or(&Value::Null)),
        num_of(dict_get(&amount, "min").unwrap_or(&Value::Null)),
        num_of(dict_get(&cost, "min").unwrap_or(&Value::Null)),
    );
}

fn addr_flags(item: &ccxt::types::DepositAddress) {
    let address_empty = item.address.as_deref().unwrap_or("").is_empty();
    let tag_empty = item.tag.as_deref().unwrap_or("").is_empty();
    println!(
        "deposit currency={} address_empty={} tag_empty={} network={}",
        item.currency.as_deref().unwrap_or(""),
        address_empty,
        tag_empty,
        item.network.as_deref().unwrap_or("")
    );
}

async fn run_v2(api_key: &str, secret: &str) -> ExitCode {
    println!("=== V2 ===");
    let config = build_v2(api_key, secret);
    let mut exchange = Indodax::new(Some(config.clone()));

    match exchange.fetch_time(Params::none()).await {
        Ok(Some(ms)) => {
            let url = url_no_query(&exchange.exchange.last_request_url);
            println!("fetch_time ok ms={ms}");
            println!("fetch_time last_request_url={url}");
            println!("fetch_time host={}", host_of(&url));
            if host_of(&url) != "d-idx.com" {
                println!("fetch_time FAIL expected host d-idx.com");
            }
        }
        Ok(None) => println!("fetch_time FAIL empty"),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_time", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            println!("fetch_time FAIL {text}");
        }
    }

    match exchange.fetch_balance(Params::none()).await {
        Ok(_) => {
            let url = url_no_query(&exchange.exchange.last_request_url);
            println!("fetch_balance ok");
            println!("fetch_balance last_request_url={url}");
            println!("fetch_balance host={}", host_of(&url));
            println!("fetch_balance path={}", path_of(&url));
            if host_of(&url) != "api.d-idx.com" {
                println!("fetch_balance FAIL expected host api.d-idx.com");
            }
            if path_of(&url) != "/api/v2/account" {
                println!("fetch_balance FAIL expected path /api/v2/account");
            }
        }
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_balance", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            println!("fetch_balance FAIL {text}");
        }
    }

    match exchange.fetch_deposit_addresses(None, Params::none()).await {
        Ok(_) => println!("fetch_deposit_addresses None UNEXPECTED OK"),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_deposit_addresses None", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            let ok = text.contains("ArgumentsRequired") && text.contains("requires symbols like BTC");
            println!("fetch_deposit_addresses None err={text} ok={ok}");
        }
    }

    match exchange
        .fetch_deposit_addresses(Some(vec![]), Params::none())
        .await
    {
        Ok(_) => println!("fetch_deposit_addresses empty UNEXPECTED OK"),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_deposit_addresses empty", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            let ok = text.contains("ArgumentsRequired") && text.contains("requires symbols like BTC");
            println!("fetch_deposit_addresses empty err={text} ok={ok}");
        }
    }

    match exchange
        .fetch_deposit_addresses(Some(vec!["BTC".to_string()]), Params::none())
        .await
    {
        Ok(list) => {
            println!("fetch_deposit_addresses BTC count={}", list.len());
            if let Some(item) = list.first() {
                addr_flags(item);
            }
        }
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_deposit_addresses BTC", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            println!("fetch_deposit_addresses BTC FAIL {text}");
        }
    }

    match exchange.fetch_markets(Params::none()).await {
        Ok(markets) => {
            if let Some(market) = markets.iter().find(|m| m.symbol == SYMBOL) {
                println!(
                    "market BTC/USDT precision.amount={:?} precision.price={:?} limits.amount.min={:?} limits.cost.min={:?} maker={:?} taker={:?}",
                    market.precision.amount,
                    market.precision.price,
                    market.limits.amount.min,
                    market.limits.cost.min,
                    market.maker,
                    market.taker
                );
            } else {
                println!("fetch_markets FAIL symbol missing count={}", markets.len());
            }
        }
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_markets", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            println!("fetch_markets FAIL {text}");
        }
    }

    {
        let mut core = IndodaxCore::new(Some(config));
        let symbols = Value::Arr(Arc::new(vec![Value::from(SYMBOL)]));
        let limits =
            call_typed(core.fetch_trading_limits(&[symbols, Value::Dict(Arc::new(HashMap::new()))]))
                .await;
        match limits {
            Ok(Value::Dict(map)) => match map.get(SYMBOL) {
                Some(entry) => print_trading_limits_entry(entry),
                None => println!("fetch_trading_limits FAIL symbol missing"),
            },
            Ok(other) => {
                let text = redact(&format!("{other:?}"), api_key, secret);
                println!("fetch_trading_limits FAIL unexpected {text}");
            }
            Err(err) => {
                let text = err_text(&err, api_key, secret);
                let code = stop_transport("fetch_trading_limits", &text);
                if code != ExitCode::SUCCESS {
                    return code;
                }
                let caught = text.contains("ArgumentsRequired") || text.contains("NotSupported");
                println!("fetch_trading_limits err={text} caught_ok={caught}");
            }
        }
    }

    ExitCode::SUCCESS
}

async fn run_v1(api_key: &str, secret: &str) -> ExitCode {
    println!("=== V1 ===");
    let config = build_v1(api_key, secret);
    let mut exchange = Indodax::new(Some(config));

    match exchange.fetch_time(Params::none()).await {
        Ok(Some(ms)) => {
            let url = url_no_query(&exchange.exchange.last_request_url);
            println!("fetch_time ok ms={ms}");
            println!("fetch_time last_request_url={url}");
        }
        Ok(None) => println!("fetch_time FAIL empty"),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_time", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            println!("fetch_time FAIL {text}");
        }
    }

    match exchange.fetch_balance(Params::none()).await {
        Ok(_) => {
            let url = url_no_query(&exchange.exchange.last_request_url);
            println!("fetch_balance ok");
            println!("fetch_balance last_request_url={url}");
            if !url.starts_with("https://d-idx.com/tapi") {
                println!("fetch_balance FAIL expected https://d-idx.com/tapi");
            }
        }
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_balance", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            println!("fetch_balance FAIL {text}");
        }
    }

    match exchange
        .fetch_open_orders(None, None, None, Params::none())
        .await
    {
        Ok(orders) => println!("fetch_open_orders no_symbol ok count={}", orders.len()),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_open_orders no_symbol", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            let caught = text.contains("ArgumentsRequired") || text.contains("NotSupported");
            println!("fetch_open_orders no_symbol err={text} caught_ok={caught}");
        }
    }

    match exchange
        .fetch_open_orders(Some(SYMBOL), None, None, Params::none())
        .await
    {
        Ok(orders) => println!("fetch_open_orders BTC/USDT ok count={}", orders.len()),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_open_orders BTC/USDT", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            let caught = text.contains("ArgumentsRequired") || text.contains("NotSupported");
            println!("fetch_open_orders BTC/USDT err={text} caught_ok={caught}");
        }
    }

    match exchange.fetch_deposit_addresses(None, Params::none()).await {
        Ok(list) => println!("fetch_deposit_addresses no_codes ok count={}", list.len()),
        Err(err) => {
            let text = err_text(&err, api_key, secret);
            let code = stop_transport("fetch_deposit_addresses no_codes", &text);
            if code != ExitCode::SUCCESS {
                return code;
            }
            let caught = text.contains("ArgumentsRequired") || text.contains("NotSupported");
            println!("fetch_deposit_addresses no_codes err={text} caught_ok={caught}");
        }
    }

    ExitCode::SUCCESS
}

#[tokio::main]
async fn main() -> ExitCode {
    let v2_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "../config.json".to_string());
    let v1_path = env::args()
        .nth(2)
        .unwrap_or_else(|| "../v1-key.json".to_string());

    let v2 = match load_config(&v2_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("v2 config load FAIL {err}");
            return ExitCode::from(1);
        }
    };
    let v1 = match load_config(&v1_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("v1 config load FAIL {err}");
            return ExitCode::from(1);
        }
    };

    let code = run_v2(&v2.api_key, &v2.secret).await;
    if code != ExitCode::SUCCESS {
        return code;
    }
    run_v1(&v1.api_key, &v1.secret).await
}
