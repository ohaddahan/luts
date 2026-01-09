#![allow(ambiguous_glob_reexports)]
use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

pub use instructions::*;

declare_id!("846qK5Drj9NEn2P4AvXCKxoVnyYQYGzMu2W7gyvoYjHT");

#[program]
pub mod luts {
    use super::*;

    pub fn create_address_lookup_table(
        ctx: Context<CreateAddressLookupTable>,
        args: CreateAddressLookupTableArgs,
    ) -> Result<()> {
        instructions::create_address_lookup_table(ctx, args)
    }

    pub fn extend_address_lookup_table(ctx: Context<ExtendAddressLookupTable>) -> Result<()> {
        instructions::extend_address_lookup_table(ctx)
    }

    pub fn deactivate_address_lookup_table(
        ctx: Context<DeactivateAddressLookupTable>,
    ) -> Result<()> {
        instructions::deactivate_address_lookup_table(ctx)
    }

    pub fn close_address_lookup_table(ctx: Context<CloseAddressLookupTable>) -> Result<()> {
        instructions::close_address_lookup_table(ctx)
    }
}
