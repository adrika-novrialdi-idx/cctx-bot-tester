"""CLI entrypoint for the Indodax sandbox bot."""

from __future__ import annotations

import argparse
import sys

from indodax_sandbox.bot import run
from indodax_sandbox.config import load_credentials
from indodax_sandbox.exchange import create_exchange


def main(argv: list[str] | None = None) -> None:
    """Parse CLI flags, load credentials, and run the sandbox bot.

    Args:
        argv: Argument list. Defaults to ``sys.argv[1:]``.
    """
    parser = argparse.ArgumentParser(description="Indodax sandbox bot")
    parser.add_argument(
        "--trade",
        action="store_true",
        help="after read-only calls, place one limit buy and cancel it",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="enable CCXT verbose logging",
    )
    parser.add_argument(
        "--config",
        default=None,
        help="path to JSON credentials file (default: config.json)",
    )
    args = parser.parse_args(argv)

    credentials = load_credentials(args.config)
    exchange = create_exchange(credentials, verbose=args.verbose)
    run(exchange, trade=args.trade)


if __name__ == "__main__":
    main()
