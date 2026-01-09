use anchor_lang::prelude::*;
use std::mem::size_of;

/// A PDA wrapper account that tracks ownership and state of an underlying Address Lookup Table.
///
/// This account is derived using seeds `["UserAddressLookupTable", signer, id]` and serves as
/// the authority for the native LUT, enabling program-controlled management with additional
/// features like deduplication and cooldown enforcement.
#[account]
pub struct UserAddressLookupTable {
    /// PDA bump seed for address derivation.
    pub bump: u8,
    /// The owner/authority who can modify this LUT.
    pub signer: Pubkey,
    /// Number of addresses added through this wrapper (may differ from LUT if extended externally).
    pub size: u64,
    /// User-defined identifier allowing multiple LUTs per signer.
    pub id: u64,
    /// The underlying native Address Lookup Table address.
    pub address_lookup_table: Pubkey,
    /// Slot of last modification, used to enforce the cooldown period before the LUT is usable.
    pub last_updated_slot: u64,
}

impl UserAddressLookupTable {
    pub const SEED: &'static str = "UserAddressLookupTable";
    /// Minimum slots to wait after extending before the LUT is usable in transactions.
    pub const COOLDOWN_SLOTS: u64 = 15;
    /// Maximum addresses a single LUT can hold.
    pub const MAX_ADDRESSES: usize = 256;

    pub const SIZE: usize = 8 // discriminator
        + size_of::<u8>() // bump
        + size_of::<Pubkey>() // signer
        + size_of::<u64>() // id
        + size_of::<u64>() // size
        + size_of::<Pubkey>() // address_lookup_table
        + size_of::<u64>(); // last_updated_slot

    /// Returns true if the cooldown period has passed and the LUT is ready for use.
    pub fn is_ready(&self, current_slot: u64) -> bool {
        current_slot >= self.last_updated_slot.saturating_add(Self::COOLDOWN_SLOTS)
    }

    /// Returns the PDA seeds for signing CPIs.
    pub fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            Self::SEED.as_bytes().to_vec(),
            self.signer.to_bytes().to_vec(),
            self.id.to_le_bytes().to_vec(),
            vec![self.bump],
        ]
    }
}
