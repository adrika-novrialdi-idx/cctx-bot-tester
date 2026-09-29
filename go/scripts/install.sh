#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_NAME="indodax-sandbox"
CT_BIN_DIR="${CT_BIN_DIR:-/Users/adrika.novrialdi/Work/support-tools-config/bin}"
mkdir -p "$CT_BIN_DIR"
cd "$ROOT"
go build -o "$CT_BIN_DIR/$BIN_NAME" ./cmd/cli
echo "Installed $CT_BIN_DIR/$BIN_NAME"
