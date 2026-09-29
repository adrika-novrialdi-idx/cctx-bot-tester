//! CLI entrypoint for the Indodax sandbox bot.

mod bot;
mod config;
mod exchange;

use std::env;
use std::process::ExitCode;

struct CliArgs {
    trade: bool,
    verbose: bool,
    config: Option<String>,
}

fn parse_args(argv: &[String]) -> Result<CliArgs, String> {
    let mut trade = false;
    let mut verbose = false;
    let mut config = None;
    let mut index = 0;

    while index < argv.len() {
        match argv[index].as_str() {
            "--trade" => {
                trade = true;
                index += 1;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            "--config" => {
                let path = argv
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --config".to_string())?;
                config = Some(path.clone());
                index += 2;
            }
            flag if flag.starts_with("--config=") => {
                config = Some(flag.trim_start_matches("--config=").to_string());
                index += 1;
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    Ok(CliArgs {
        trade,
        verbose,
        config,
    })
}

#[tokio::main]
async fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(1);
        }
    };

    let credentials = match config::load_credentials(args.config.as_deref()) {
        Ok(credentials) => credentials,
        Err(code) => return code,
    };

    let mut exchange = exchange::create_exchange(&credentials, args.verbose);
    match bot::run(&mut exchange, &credentials, args.trade).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}
