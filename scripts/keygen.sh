#!/usr/bin/env bash

# Tworzy nowy portfel Solana tak jak w przewodniku – w katalogu examples.

set -euo pipefail

KEYPAIR_DIR="${KEYPAIR_DIR:-$(pwd)/wallet}"
mkdir -p "$KEYPAIR_DIR"

# Generujemy parę kluczy i zapisujemy ją w bezpiecznym katalogu z ograniczonymi uprawnieniami.
solana-keygen new --outfile "$KEYPAIR_DIR/id.json" --no-passphrase

# Wyświetlamy adres publiczny, aby móc wykorzystać go w kolejnych krokach.
solana address --keypair "$KEYPAIR_DIR/id.json"
