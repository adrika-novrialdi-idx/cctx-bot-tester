"""Load Indodax API credentials from a JSON file and environment variables."""

from __future__ import annotations

import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Credentials:
    """API credentials for the Indodax exchange."""

    api_key: str
    secret: str


def load_credentials(config_path: str | None = None) -> Credentials:
    """Load credentials from a JSON file, then apply non-empty environment overrides.

    Args:
        config_path: Path to the JSON config file. Defaults to ``config.json``
            in the current working directory.

    Returns:
        The resolved API key and secret.

    Raises:
        SystemExit: Exit code 2 when either credential is empty after merging
            file and environment values. Names the missing variable on stderr.
    """
    path = Path(config_path) if config_path else Path("config.json")
    file_api_key = ""
    file_secret = ""

    if path.is_file():
        with path.open(encoding="utf-8") as handle:
            data = json.load(handle)
        if isinstance(data, dict):
            file_api_key = str(data.get("apiKey") or "")
            file_secret = str(data.get("secret") or "")

    env_api_key = os.environ.get("INDODAX_APIKEY", "")
    env_secret = os.environ.get("INDODAX_SECRET", "")

    api_key = env_api_key if env_api_key else file_api_key
    secret = env_secret if env_secret else file_secret

    if not api_key:
        print("missing INDODAX_APIKEY", file=sys.stderr)
        raise SystemExit(2)
    if not secret:
        print("missing INDODAX_SECRET", file=sys.stderr)
        raise SystemExit(2)

    return Credentials(api_key=api_key, secret=secret)
