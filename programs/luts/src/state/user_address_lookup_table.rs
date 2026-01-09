use anchor_lang::prelude::*;
use std::mem::size_of;

#[account]
pub struct UserAddressLookupTable {
    pub bump: u8,
    pub signer: Pubkey,
    pub size: u64,
    pub id: u64,
    pub address_lookup_table: Pubkey,
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
        + size_of::<u64>() // id
        + size_of::<u64>() // size
        + size_of::<Pubkey>() // address_lookup_table
        + size_of::<u64>() // last_updated_slot
        + size_of::<i64>(); // last_updated_timestamp

    pub fn is_ready(&self, current_slot: u64) -> bool {
        current_slot >= self.last_updated_slot.saturating_add(Self::COOLDOWN_SLOTS)
    }

    pub fn slots_until_ready(&self, current_slot: u64) -> u64 {
        let ready_slot = self.last_updated_slot.saturating_add(Self::COOLDOWN_SLOTS);
        ready_slot.saturating_sub(current_slot)
    }

    pub fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            Self::SEED.as_bytes().to_vec(),
            self.signer.to_bytes().to_vec(),
            self.id.to_le_bytes().to_vec(),
            vec![self.bump],
        ]
    }
}
