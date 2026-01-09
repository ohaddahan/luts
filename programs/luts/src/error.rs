use anchor_lang::prelude::*;

#[error_code]
pub enum LutError {
    #[msg("Invalid Lookup Table address")]
    InvalidLookupTable,
    #[msg("LUT not yet ready for use (cooldown period)")]
    LutNotReady,
    #[msg("Maximum addresses exceeded (256 limit)")]
    MaxAddressesExceeded,
}
