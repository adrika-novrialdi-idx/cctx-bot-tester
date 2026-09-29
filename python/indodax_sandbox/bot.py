"""Orchestrate the Indodax sandbox read-only and optional trade paths."""

from __future__ import annotations

import sys
from typing import Any

from indodax_sandbox.exchange import PUBLIC_URL, SANDBOX_URL

SYMBOL = "BTC/USDT"
MAX_NOTIONAL_USD = 25.0
TARGET_NOTIONAL_USD = 10.0


def _redact_error(exc: BaseException, exchange: Any) -> str:
    text = str(exc)
    api_key = getattr(exchange, "apiKey", None) or ""
    secret = getattr(exchange, "secret", None) or ""
    if api_key:
        text = text.replace(str(api_key), "[REDACTED]")
    if secret:
        text = text.replace(str(secret), "[REDACTED]")
    return text


def run(exchange: Any, trade: bool = False) -> None:
    """Run the read-only sandbox path, and optionally the trade path.

    Args:
        exchange: A configured Indodax CCXT client.
        trade: When True, place and cancel one limit buy after the read-only calls.

    Raises:
        SystemExit: Exit code 2 for missing ticker price or refused notional.
            Non-zero exit when an exchange call fails.
    """
    try:
        server_time = exchange.fetch_time()
        print(f"server time: {server_time}")
        exchange.fetch_balance()
        print(f"ok public={PUBLIC_URL} private={SANDBOX_URL}")
    except SystemExit:
        raise
    except Exception as exc:
        print(_redact_error(exc, exchange), file=sys.stderr)
        raise SystemExit(1) from exc

    if not trade:
        return

    try:
        ticker = exchange.fetch_ticker(SYMBOL)
        last = ticker.get("last") if isinstance(ticker, dict) else None
        if last is None or float(last) <= 0:
            print("missing or invalid ticker last price", file=sys.stderr)
            raise SystemExit(2)

        exchange.load_markets()

        raw_price = float(last) * 0.5
        raw_amount = TARGET_NOTIONAL_USD / raw_price
        precise_price = exchange.price_to_precision(SYMBOL, raw_price)
        precise_amount = exchange.amount_to_precision(SYMBOL, raw_amount)
        amount_value = float(precise_amount)
        price_value = float(precise_price)
        notional = amount_value * price_value

        if notional >= MAX_NOTIONAL_USD or amount_value == 0:
            print(
                f"refused: notional {notional} USD is not below 25",
                file=sys.stderr,
            )
            raise SystemExit(2)

        order_id = None
        try:
            order = exchange.create_order(
                SYMBOL,
                "limit",
                "buy",
                amount_value,
                price_value,
            )
            if isinstance(order, dict):
                order_id = order.get("id")
            if order_id is not None:
                print(f"order id: {order_id}")
        finally:
            if order_id is not None:
                try:
                    exchange.cancel_order(order_id, SYMBOL)
                    print(f"canceled order id: {order_id}")
                except Exception as cancel_exc:
                    print(_redact_error(cancel_exc, exchange), file=sys.stderr)
                    raise SystemExit(1) from cancel_exc
    except SystemExit:
        raise
    except Exception as exc:
        print(_redact_error(exc, exchange), file=sys.stderr)
        raise SystemExit(1) from exc
