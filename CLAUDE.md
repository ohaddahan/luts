# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LUTs is an Anchor program that wraps Solana's native Address Lookup Table (ALT) program, adding deduplication, cooldown enforcement, and ownership tracking via a PDA wrapper account.

## Build and Test Commands

```bash
# Build
anchor build
npm run build

# Generate Rust and TypeScript clients from IDL
npm run gen-clients

# Run all TypeScript tests (requires local validator)
anchor test

# Run Mollusk unit tests (fast, no validator)
anchor run test-mollusk
cargo nextest run --features test-sbf

# Run a single Mollusk test
cargo nextest run test_create_address_lookup_table --features test-sbf

# Type check
anchor run check
cargo check --tests --features test-sbf

# Lint
npm run lint
npm run lint:fix
```

## Architecture

### Address Derivation (Two-Level PDA Scheme)

1. **UserAddressLookupTable PDA**: `["UserAddressLookupTable", signer, id_le_bytes]` → LUTS program
2. **Native LUT**: `[UserAddressLookupTable_pubkey, recent_slot_le_bytes]` → ALT program

The wrapper PDA becomes the authority of the native LUT, giving this program control. The `id` parameter is user-controlled, allowing multiple LUTs per signer.

### Program Structure

```
programs/luts/src/
├── lib.rs                    # Program entry, declares 4 instructions
├── state/                    # UserAddressLookupTable account definition
├── instructions/             # Instruction handlers (create, extend, deactivate, close)
├── constants.rs              # LutProgram wrapper for ALT program ID
├── error.rs                  # Custom errors
└── events.rs                 # Event definitions
```

### Generated Clients

- `codama-rust-luts/` - Auto-generated Rust client (used by Mollusk tests)
- `codama-ts-luts/` - Auto-generated TypeScript client
- `codama-ts-luts-custom/` - Hand-written TS utilities (PDA derivation, instruction wrappers)

### Test Structure

- `tests/luts-anchor.ts` - TypeScript tests using Anchor client
- `tests/luts-codama.ts` - TypeScript tests using Codama clients
- `programs/luts/tests/` - Mollusk-based Rust unit tests (no validator needed)

### Key Constraints

- 15-slot cooldown between LUT extensions (ensures activation)
- 256 address maximum per LUT
- Automatic deduplication when extending
