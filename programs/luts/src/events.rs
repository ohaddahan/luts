use anchor_lang::prelude::*;

#[event]
pub struct LutCreated {
    pub wrapper: Pubkey,
    pub lut_address: Pubkey,
    pub authority: Pubkey,
    pub slot: u64,
}

#[event]
pub struct LutExtended {
    pub wrapper: Pubkey,
    pub addresses_added: u32,
    pub total_addresses: u32,
    pub is_ready: bool,
    pub slots_until_ready: u64,
}

#[event]
pub struct LutDeactivated {
    pub wrapper: Pubkey,
    pub lut_address: Pubkey,
}

#[event]
pub struct LutClosed {
    pub wrapper: Pubkey,
    pub lut_address: Pubkey,
}
