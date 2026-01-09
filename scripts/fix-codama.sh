#!/bin/bash

set -e

RUST_DIR="${1:-codama-rust-luts}"

if [ ! -d "$RUST_DIR" ]; then
  echo "Rust output directory $RUST_DIR not found"
  exit 1
fi

echo "Fixing Rust client in $RUST_DIR..."

# Add allow directives to mod.rs
if [ -f "$RUST_DIR/mod.rs" ]; then
  if ! grep -q "#!\[allow(unused_imports,dead_code)\]" "$RUST_DIR/mod.rs"; then
    echo '#![allow(unused_imports,dead_code)]' | cat - "$RUST_DIR/mod.rs" > temp && mv temp "$RUST_DIR/mod.rs"
  fi
fi

# Fix pub(crate) to pub in all Rust files
find "$RUST_DIR" -name "*.rs" | while read -r file; do
  sed -i 's/pub(crate)/pub/g' "$file"
  sed -i 's/crate::types::/crate::codama_rust_luts::types::/g' "$file"
  sed -i 's/crate::generated::types::/crate::codama_rust_luts::types::/g' "$file"
done

# Format Rust files
echo "Formatting Rust files..."
find "$RUST_DIR" -name "*.rs" -exec rustfmt {} \; 2>/dev/null || true

# Format TypeScript files
if command -v npm &> /dev/null; then
  echo "Formatting TypeScript files..."
  npm run lint:fix 2>/dev/null || true
fi

echo "Done fixing Codama output!"
