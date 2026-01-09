use crate::constants::LutProgram;
use crate::error::LutError;
use crate::events::LutExtended;
use crate::state::user_address_lookup_table::UserAddressLookupTable;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use solana_address_lookup_table_interface::instruction::extend_lookup_table;

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
        seeds = [UserAddressLookupTable::SEED.as_bytes(), signer.key().as_ref()],
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

    let new_addresses: Vec<Pubkey> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| *acc.key)
        .filter(|addr| !user_address_lookup_table.accounts.contains(addr))
        .collect();

    if new_addresses.is_empty() {
        return Ok(());
    }

    let total_after = user_address_lookup_table
        .accounts
        .len()
        .saturating_add(new_addresses.len());
    require!(
        total_after <= UserAddressLookupTable::MAX_ADDRESSES,
        LutError::MaxAddressesExceeded
    );

    let clock = Clock::get()?;
    user_address_lookup_table.last_updated_slot = clock.slot;
    user_address_lookup_table.last_updated_timestamp = clock.unix_timestamp;

    user_address_lookup_table
        .accounts
        .extend(new_addresses.iter().cloned());

    let ix = extend_lookup_table(
        address_lookup_table.key(),
        user_address_lookup_table.key(),
        Some(signer.key()),
        new_addresses.clone(),
    );

    let binding = signer.key();
    let seeds = &[
        UserAddressLookupTable::SEED.as_bytes(),
        binding.as_ref(),
        &[user_address_lookup_table.bump],
    ];

    solana_program::program::invoke_signed(
        &ix,
        &[
            signer.to_account_info(),
            system_program.to_account_info(),
            address_lookup_table.to_account_info(),
            user_address_lookup_table.to_account_info(),
        ],
        &[seeds],
    )?;

    let is_ready = user_address_lookup_table.is_ready(clock.slot);
    let slots_until_ready = user_address_lookup_table.slots_until_ready(clock.slot);

    emit!(LutExtended {
        wrapper: user_address_lookup_table.key(),
        addresses_added: new_addresses.len() as u32,
        total_addresses: user_address_lookup_table.accounts.len() as u32,
        is_ready,
        slots_until_ready,
    });

    Ok(())
}
