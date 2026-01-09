use anchor_lang::prelude::*;
use anchor_lang::Id;
pub use solana_sdk_ids::address_lookup_table::ID as LOOKUP_TABLE_PROGRAM;

#[derive(Clone)]
pub struct LutProgram;

impl Id for LutProgram {
    fn id() -> Pubkey {
        LOOKUP_TABLE_PROGRAM
    }
}
