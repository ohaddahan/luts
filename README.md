# LUTs - Address Lookup Table Wrapper

An Anchor program that wraps Solana's native Address Lookup Table (ALT) program with deduplication and readiness tracking.

## Overview

This program provides a managed wrapper around Solana's Address Lookup Tables, adding:

- **Deduplication**: Automatically filters duplicate addresses when extending tables
- **Readiness tracking**: Enforces a cooldown period (15 slots) between extensions to ensure LUT activation
- **Ownership tracking**: Links each LUT to a specific user via a PDA wrapper account
- **Size limits**: Enforces the 256-address maximum per lookup table

## Program ID

```
846qK5Drj9NEn2P4AvXCKxoVnyYQYGzMu2W7gyvoYjHT
```

## State

### UserAddressLookupTable

A PDA wrapper account that tracks ownership and state of an underlying Address Lookup Table.

| Field | Type | Description |
|-------|------|-------------|
| `bump` | `u8` | PDA bump seed |
| `signer` | `Pubkey` | Owner/authority of this LUT |
| `size` | `u64` | Number of addresses added through this wrapper |
| `id` | `u64` | User-defined identifier for multiple LUTs per signer |
| `address_lookup_table` | `Pubkey` | The underlying native ALT address |
| `last_updated_slot` | `u64` | Slot of last modification (for cooldown tracking) |

**PDA Seeds**: `["UserAddressLookupTable", signer, id]`

## Address Derivation

The program uses a two-level address derivation scheme:

### 1. UserAddressLookupTable PDA (Program-Controlled)

The wrapper account address is derived deterministically from:

```
Seeds: ["UserAddressLookupTable", signer_pubkey, id_as_le_bytes]
Program: LUTS Program (846qK5Drj9NEn2P4AvXCKxoVnyYQYGzMu2W7gyvoYjHT)
```

The `id` parameter is **user-controlled**, allowing a single signer to create multiple independent LUTs by incrementing the id.

### 2. Native LUT Address (Solana ALT Program)

The underlying Address Lookup Table address is derived by Solana's native ALT program:

```
Seeds: [authority_pubkey, recent_slot_as_le_bytes]
Program: AddressLookupTab1e1111111111111111111111111
```

Here, the `authority` is the UserAddressLookupTable PDA (not the user's signer), giving this program control over the LUT. The `recent_slot` must be within ~150 slots of the current slot.

### Derivation Flow

```
User provides: signer + id + recent_slot
         │
         ▼
┌─────────────────────────────────────────────────┐
│ UserAddressLookupTable PDA                      │
│ = PDA(["UserAddressLookupTable", signer, id])   │
└─────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────┐
│ Native LUT Address                              │
│ = PDA([UserAddressLookupTable, recent_slot])    │
└─────────────────────────────────────────────────┘
```

## Instructions

### create_address_lookup_table

Creates a new Address Lookup Table with an associated wrapper account.

**Arguments**:
- `recent_slot`: A recent slot used to derive the LUT address
- `id`: User-defined identifier (allows multiple LUTs per signer)

**Accounts**:
- `signer`: Transaction signer and LUT owner (mut, signer)
- `system_program`: System program
- `address_lookup_table_program`: Native ALT program
- `address_lookup_table`: The LUT to be created (mut)
- `user_address_lookup_table`: Wrapper PDA to be initialized (mut)
- `rent`: Rent sysvar

### extend_address_lookup_table

Adds new addresses to an existing lookup table. Automatically deduplicates against existing entries.

**Accounts**:
- `signer`: LUT owner (mut, signer)
- `system_program`: System program
- `address_lookup_table_program`: Native ALT program
- `address_lookup_table`: The LUT to extend (mut)
- `user_address_lookup_table`: Wrapper PDA (mut)
- `rent`: Rent sysvar
- `remaining_accounts`: Addresses to add to the LUT

**Constraints**:
- Must wait 15 slots after last update (cooldown period)
- Total addresses cannot exceed 256
- At least one new (non-duplicate) address must be provided

### deactivate_address_lookup_table

Begins the deactivation process for a lookup table. After deactivation, the table can be closed once it's no longer in use by any recent transactions.

**Accounts**:
- `signer`: LUT owner (mut, signer)
- `system_program`: System program
- `address_lookup_table_program`: Native ALT program
- `address_lookup_table`: The LUT to deactivate (mut)
- `user_address_lookup_table`: Wrapper PDA (mut)
- `rent`: Rent sysvar

### close_address_lookup_table

Closes a deactivated lookup table and its wrapper account, reclaiming rent to the signer.

**Accounts**:
- `signer`: LUT owner (mut, signer)
- `system_program`: System program
- `address_lookup_table_program`: Native ALT program
- `address_lookup_table`: The LUT to close (mut)
- `user_address_lookup_table`: Wrapper PDA to close (mut)
- `rent`: Rent sysvar

## Events

| Event | Fields | Description |
|-------|--------|-------------|
| `LutCreated` | wrapper, lut_address, authority, slot | Emitted when a new LUT is created |
| `LutExtended` | wrapper, addresses_added, total_addresses | Emitted when addresses are added |
| `LutDeactivated` | wrapper, lut_address | Emitted when a LUT is deactivated |
| `LutClosed` | wrapper, lut_address | Emitted when a LUT is closed |

## Errors

| Error | Description |
|-------|-------------|
| `InvalidLookupTable` | The provided LUT address doesn't match the expected derived address |
| `LutNotReady` | Cooldown period (15 slots) hasn't passed since last update |
| `MaxAddressesExceeded` | Adding addresses would exceed the 256 address limit |
| `NoNewAddresses` | All provided addresses already exist in the LUT |

## Development

### Prerequisites

- Rust and Cargo
- Solana CLI tools
- Anchor CLI
- Node.js and npm

### Build

```bash
# Build the program
anchor build

# Or via npm
npm run build
```

### Generate Clients

Generate TypeScript and Rust client code using Codama:

```bash
npm run gen-clients
```

This runs:
1. `anchor build` - Builds the program and generates IDL
2. `npm run codama` - Generates Rust and TypeScript clients from IDL
3. `scripts/fix-codama.sh` - Applies post-generation fixes

## Generated Client Libraries

### codama-rust-luts/

Auto-generated Rust client library containing:
- `instructions/` - Instruction builders for all program instructions
- `accounts/` - Account structs with serialization
- `types/` - Event types (LutCreated, LutExtended, etc.)
- `errors/` - Program error definitions
- `shared.rs` - Shared utilities and account fetching (with `fetch` feature)

Used by the Mollusk-based tests for instruction building.

### codama-ts-luts/

Auto-generated TypeScript client library containing:
- `instructions/` - Instruction builders
- `accounts/` - Account decoders (e.g., `getUserAddressLookupTableDecoder`)
- `types/` - Event type definitions
- `errors/` - Error definitions
- `programs/` - Program address constant (`LUTS_PROGRAM_ADDRESS`)

### codama-ts-luts-custom/

Hand-written TypeScript utilities that extend the generated client:
- `constants/` - Seed constants and program IDs
- `pda/` - PDA derivation helpers:
  - `getUserAddressLookupTableAddress(signer, id)` - Derives wrapper PDA
  - `deriveAddressLookupTableAddress(authority, recentSlot)` - Derives native LUT address
- `wrappers/` - High-level instruction builders:
  - `buildCreateAddressLookupTableInstruction()`
  - `buildExtendAddressLookupTableInstruction()`
  - `buildDeactivateAddressLookupTableInstruction()`
  - `buildCloseAddressLookupTableInstruction()`
- `utils/` - Transaction helpers (`processAndValidateTransaction`)

## Testing

### TypeScript Integration Tests (requires local validator)

Located in `tests/`:

**tests/luts-anchor.ts** - Uses the Anchor client library directly:
```bash
anchor test
```

**tests/luts-codama.ts** - Uses the Codama-generated clients and custom wrappers:
```bash
anchor test
```

Both test suites cover:
- Creating address lookup tables
- Cooldown period enforcement (LutNotReady error)
- Extending tables with new addresses
- Duplicate address filtering (NoNewAddresses error)
- Size tracking across multiple extensions

### Mollusk Unit Tests (fast, no validator required)

Located in `programs/luts/tests/`:

```
programs/luts/tests/
├── main.rs                    # Test entry point
├── common/
│   ├── helpers.rs             # Test context and utilities
│   └── pda.rs                 # PDA derivation helpers
└── integration/
    ├── test_create_address_lookup_table.rs
    ├── test_extend_address_lookup_table.rs
    ├── test_deactivate_address_lookup_table.rs
    └── test_close_address_lookup_table.rs
```

Uses [mollusk-helper](https://crates.io/crates/mollusk-helper) for fast, deterministic testing without a validator. Tests use the Codama-generated Rust client (`codama-rust-luts`) for instruction building.

```bash
# Run mollusk tests
npm run gen-clients && cargo nextest run --features test-sbf

# Or via anchor script
anchor run test-mollusk
```

### Type Checking

```bash
anchor run check
# Or directly:
cargo check --tests --features test-sbf
```

### Linting

```bash
npm run lint        # Check for issues
npm run lint:fix    # Auto-fix issues
```
