# Indodax sandbox bots

Standalone Indodax sandbox bots (one per language) that share one behavior contract. They talk to the Indodax sandbox through a CCXT fork branch that already has TAPI v2 and sandbox hosts.

This repository does not call the sandbox by itself. You must run a language bot with credentials. A TAPI v1 key cannot call TAPI v2.

## Credentials

Load `apiKey` and `secret` from a JSON file, then let the environment override when the variable is non-empty.

- File: `--config` path, otherwise `config.json` in the current working directory. Shape: `{ "apiKey": "", "secret": "" }`. Copy `config.example.json` to start.
- Env: `INDODAX_APIKEY`, `INDODAX_SECRET`.
- If either value is empty, the bot exits 2 before any HTTP call and names the missing variable. Never print the key or secret.

## Hosts

Fixed. No trailing slash. Do not read hosts from config. Do not call `setSandboxMode` / `set_sandbox_mode` / `SetSandboxMode`. Do not use `indodax.com` or `api.indodax.com`. `tapiVersion` is the string `"2"`, not a number.

| Calls | Base |
| --- | --- |
| Public REST | `https://d-idx.com` |
| TAPI v1 private | `https://d-idx.com/tapi` |
| TAPI v2 | `https://api.d-idx.com` (`options.sandboxUrl`) |

`enableRateLimit` is true.

## Flags

| Flag | Meaning |
| --- | --- |
| `--trade` | After the read-only calls, place one limit buy and cancel it |
| `--verbose` | Set the CCXT verbose flag (default off) |
| `--config` | Path to the JSON credentials file |

## Run commands

From each language directory:

```bash
# Go
cd go && go run ./cmd/cli run

# JavaScript
cd javascript && node src/cli.js

# Python
cd python && python -m indodax_sandbox

# PHP
cd php && php bin/run.php

# C#
cd csharp/IndodaxSandbox && dotnet run

# Java
cd java && ./gradlew run

# Rust
cd rust && cargo run
```

Add `--trade`, `--verbose`, or `--config <path>` as needed.
