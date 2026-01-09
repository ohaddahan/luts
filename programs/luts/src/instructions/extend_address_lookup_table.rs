use crate::constants::LutProgram;
use crate::error::LutError;
use crate::events::LutExtended;
use crate::state::user_address_lookup_table::UserAddressLookupTable;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program;
use solana_address_lookup_table_interface::instruction::extend_lookup_table;
use solana_address_lookup_table_interface::state::AddressLookupTable;

/// Extends an Address Lookup Table with new addresses.
///
/// Addresses to add are passed via remaining_accounts. The instruction automatically
/// deduplicates against existing entries in the LUT. A cooldown period must have passed
/// since the last update, and the total cannot exceed 256 addresses.
#[derive(Accounts)]
pub struct ExtendAddressLookupTable<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub address_lookup_table_program: Program<'info, LutProgram>,
    /// CHECK: Validated via has_one constraint on wrapper
    #[account(mut)]
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(
        mut,
        has_one = address_lookup_table,
        has_one = signer,
        seeds = [UserAddressLookupTable::SEED.as_bytes(), signer.key().as_ref(), user_address_lookup_table.id.to_le_bytes().as_ref()],
        bump = user_address_lookup_table.bump
    )]
    pub user_address_lookup_table: Box<Account<'info, UserAddressLookupTable>>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn extend_address_lookup_table(ctx: Context<ExtendAddressLookupTable>) -> Result<()> {
    let signer = &ctx.accounts.signer;
    let user_address_lookup_table = &mut ctx.accounts.user_address_lookup_table;
    let address_lookup_table = &ctx.accounts.address_lookup_table;
    let system_program = &ctx.accounts.system_program;
    let clock = Clock::get()?;
    require!(
        user_address_lookup_table.is_ready(clock.slot),
        LutError::LutNotReady
    );
    let lut_data = address_lookup_table.try_borrow_data()?;
    let lut =
        AddressLookupTable::deserialize(&lut_data).map_err(|_| LutError::InvalidLookupTable)?;
    let existing_addresses = lut.addresses;
    let new_addresses: Vec<Pubkey> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| *acc.key)
        .filter(|addr| !existing_addresses.contains(addr))
        .collect();
    require!(!new_addresses.is_empty(), LutError::NoNewAddresses);
    user_address_lookup_table.size += new_addresses.len() as u64;
    let total_after = existing_addresses.len().saturating_add(new_addresses.len());
    require!(
        total_after <= UserAddressLookupTable::MAX_ADDRESSES,
        LutError::MaxAddressesExceeded
    );
    drop(lut_data);
    user_address_lookup_table.last_updated_slot = clock.slot;
    let ix = extend_lookup_table(
        address_lookup_table.key(),
        user_address_lookup_table.key(),
        Some(signer.key()),
        new_addresses.clone(),
    );
    let seeds = user_address_lookup_table.seeds();
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|v| v.as_slice()).collect();
    let signer_seeds: &[&[&[u8]]] = &[seed_slices.as_slice()];
    program::invoke_signed(
        &ix,
        &[
            signer.to_account_info(),
            system_program.to_account_info(),
            address_lookup_table.to_account_info(),
            user_address_lookup_table.to_account_info(),
        ],
        signer_seeds,
    )?;

    emit!(LutExtended {
        wrapper: user_address_lookup_table.key(),
        addresses_added: new_addresses.len() as u32,
        total_addresses: total_after as u32,
    });
    Ok(())
}
