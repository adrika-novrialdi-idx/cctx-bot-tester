//! Orchestrate the Indodax sandbox read-only and optional trade paths.

use crate::config::Credentials;
use crate::exchange::{PUBLIC_URL, SANDBOX_URL};
use ccxt::exchange_generated::ExchangeBase;
use ccxt::runtime::catch_typed;
use ccxt::{Indodax, Params, Value};
use std::process::ExitCode;

const SYMBOL: &str = "BTC/USDT";
const MAX_NOTIONAL_USD: f64 = 25.0;
const TARGET_NOTIONAL_USD: f64 = 10.0;

/// Redacts credential values from an error message.
fn redact_error(message: &str, credentials: &Credentials) -> String {
    let mut text = message.to_string();
    if !credentials.api_key.is_empty() {
        text = text.replace(&credentials.api_key, "[REDACTED]");
    }
    if !credentials.secret.is_empty() {
        text = text.replace(&credentials.secret, "[REDACTED]");
    }
    text
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Float(n) => Some(*n),
        Value::Int(n) => Some(*n as f64),
        Value::Str(s) => s.parse::<f64>().ok(),
        _ => value.as_str().and_then(|s| s.parse::<f64>().ok()),
    }
}

/// Runs the read-only sandbox path, and optionally the trade path.
pub async fn run(
    exchange: &mut Indodax,
    credentials: &Credentials,
    trade: bool,
) -> Result<(), ExitCode> {
    match run_readonly(exchange).await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", redact_error(&err, credentials));
            return Err(ExitCode::from(1));
        }
    }

    if !trade {
        return Ok(());
    }

    run_trade(exchange, credentials).await
}

async fn run_readonly(exchange: &mut Indodax) -> Result<(), String> {
    let server_time = exchange
        .fetch_time(Params::none())
        .await
        .map_err(|err| err.to_string())?;
    match server_time {
        Some(ms) => println!("server time: {ms}"),
        None => return Err("fetch_time returned no server time".to_string()),
    }

    exchange
        .fetch_balance(Params::none())
        .await
        .map_err(|err| err.to_string())?;
    println!("ok public={PUBLIC_URL} private={SANDBOX_URL}");
    Ok(())
}

async fn run_trade(
    exchange: &mut Indodax,
    credentials: &Credentials,
) -> Result<(), ExitCode> {
    let ticker = match exchange.fetch_ticker(SYMBOL, Params::none()).await {
        Ok(ticker) => ticker,
        Err(err) => {
            eprintln!("{}", redact_error(&err.to_string(), credentials));
            return Err(ExitCode::from(1));
        }
    };

    let last = match ticker.last {
        Some(value) if value > 0.0 => value,
        _ => {
            eprintln!("missing or invalid ticker last price");
            return Err(ExitCode::from(2));
        }
    };

    if let Err(err) = exchange.try_load_markets(false).await {
        eprintln!("{}", redact_error(&err.to_string(), credentials));
        return Err(ExitCode::from(1));
    }

    let raw_price = last * 0.5;
    let raw_amount = TARGET_NOTIONAL_USD / raw_price;

    let precise_price_value = match catch_typed(|| {
        ExchangeBase::price_to_precision(
            &*exchange,
            Value::from(SYMBOL),
            Value::from(raw_price),
        )
    }) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}", redact_error(&err.to_string(), credentials));
            return Err(ExitCode::from(1));
        }
    };
    let precise_amount_value = match catch_typed(|| {
        ExchangeBase::amount_to_precision(
            &*exchange,
            Value::from(SYMBOL),
            Value::from(raw_amount),
        )
    }) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}", redact_error(&err.to_string(), credentials));
            return Err(ExitCode::from(1));
        }
    };

    let price_value = match value_to_f64(&precise_price_value) {
        Some(value) => value,
        None => {
            eprintln!("refused: notional NaN USD is not below 25");
            return Err(ExitCode::from(2));
        }
    };
    let amount_value = match value_to_f64(&precise_amount_value) {
        Some(value) => value,
        None => {
            eprintln!("refused: notional NaN USD is not below 25");
            return Err(ExitCode::from(2));
        }
    };
    let notional = amount_value * price_value;

    if !(amount_value > 0.0) || !notional.is_finite() || notional >= MAX_NOTIONAL_USD {
        let value = if notional.is_finite() {
            notional.to_string()
        } else {
            "NaN".to_string()
        };
        eprintln!("refused: notional {value} USD is not below 25");
        return Err(ExitCode::from(2));
    }

    let mut order_id: Option<String> = None;
    let create_err = match exchange
        .create_order(
            SYMBOL,
            "limit",
            "buy",
            amount_value,
            Some(price_value),
            Params::none(),
        )
        .await
    {
        Ok(order) => {
            if let Some(id) = order.id.filter(|id| !id.is_empty()) {
                println!("order id: {id}");
                order_id = Some(id);
            }
            None
        }
        Err(err) => Some(err),
    };

    let cancel_err = if let Some(id) = order_id.as_deref() {
        match exchange
            .cancel_order(id, Some(SYMBOL), Params::none())
            .await
        {
            Ok(_) => {
                println!("canceled order id: {id}");
                None
            }
            Err(err) => Some(err),
        }
    } else {
        None
    };

    if let Some(err) = create_err {
        eprintln!("{}", redact_error(&err.to_string(), credentials));
        return Err(ExitCode::from(1));
    }
    if let Some(err) = cancel_err {
        eprintln!("{}", redact_error(&err.to_string(), credentials));
        return Err(ExitCode::from(1));
    }

    Ok(())
}
