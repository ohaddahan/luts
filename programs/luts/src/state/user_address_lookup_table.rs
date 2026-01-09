use anchor_lang::prelude::*;
use std::mem::size_of;

#[account]
pub struct UserAddressLookupTable {
    pub bump: u8,
    pub signer: Pubkey,
    pub id: u64,
    pub address_lookup_table: Pubkey,
    pub accounts: Vec<Pubkey>,
    pub last_updated_slot: u64,
    pub last_updated_timestamp: i64,
}

impl UserAddressLookupTable {
    pub const SEED: &'static str = "UserAddressLookupTable";
    pub const COOLDOWN_SLOTS: u64 = 150;
    pub const MAX_ADDRESSES: usize = 256;

    pub const SIZE: usize = 8 // discriminator
        + size_of::<u8>() // bump
        + size_of::<Pubkey>() // signer
        + size_of::<u64>() // signer
        + size_of::<Pubkey>() // address_lookup_table
        + 4 + (Self::MAX_ADDRESSES * size_of::<Pubkey>()) // accounts vec (length prefix + data)
        + size_of::<u64>() // last_updated_slot
        + size_of::<i64>() // last_updated_timestamp
        + 100; // padding

    pub fn is_ready(&self, current_slot: u64) -> bool {
        current_slot >= self.last_updated_slot.saturating_add(Self::COOLDOWN_SLOTS)
    }

    pub fn slots_until_ready(&self, current_slot: u64) -> u64 {
        let ready_slot = self.last_updated_slot.saturating_add(Self::COOLDOWN_SLOTS);
        ready_slot.saturating_sub(current_slot)
    }

    pub fn contains(&self, address: &Pubkey) -> bool {
        self.accounts.contains(address)
    }

    pub fn address_count(&self) -> usize {
        self.accounts.len()
    }

    pub fn remaining_capacity(&self) -> usize {
        Self::MAX_ADDRESSES.saturating_sub(self.accounts.len())
    }
}
