"""Build a sandbox-configured Indodax CCXT exchange client."""

from __future__ import annotations

import ccxt

from indodax_sandbox.config import Credentials

PUBLIC_URL = "https://d-idx.com"
PRIVATE_URL = "https://d-idx.com/tapi"
SANDBOX_URL = "https://api.d-idx.com"


def create_exchange(credentials: Credentials, verbose: bool = False) -> ccxt.indodax:
    """Create a sync Indodax client pointed at the sandbox hosts.

    Args:
        credentials: API key and secret.
        verbose: When True, enable CCXT verbose request logging.

    Returns:
        A configured ``ccxt.indodax`` instance. Does not call ``set_sandbox_mode``.
    """
    return ccxt.indodax(
        {
            "apiKey": credentials.api_key,
            "secret": credentials.secret,
            "enableRateLimit": True,
            "verbose": verbose,
            "urls": {
                "api": {
                    "public": PUBLIC_URL,
                    "private": PRIVATE_URL,
                },
            },
            "options": {
                "tapiVersion": "2",
                "sandboxUrl": SANDBOX_URL,
            },
        }
    )
