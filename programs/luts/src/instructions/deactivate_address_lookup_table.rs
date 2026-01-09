use crate::constants::LutProgram;
use crate::events::LutDeactivated;
use crate::state::user_address_lookup_table::UserAddressLookupTable;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use solana_address_lookup_table_interface::instruction::deactivate_lookup_table;

#[derive(Accounts)]
pub struct DeactivateAddressLookupTable<'info> {
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

pub fn deactivate_address_lookup_table(ctx: Context<DeactivateAddressLookupTable>) -> Result<()> {
    let signer = &ctx.accounts.signer;
    let user_address_lookup_table = &ctx.accounts.user_address_lookup_table;
    let address_lookup_table = &ctx.accounts.address_lookup_table;

    let binding = signer.key();
    let seeds = &[
        UserAddressLookupTable::SEED.as_bytes(),
        binding.as_ref(),
        &[user_address_lookup_table.bump],
    ];

    let ix = deactivate_lookup_table(address_lookup_table.key(), user_address_lookup_table.key());

    solana_program::program::invoke_signed(
        &ix,
        &[
            signer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            address_lookup_table.to_account_info(),
            user_address_lookup_table.to_account_info(),
        ],
        &[seeds],
    )?;

    emit!(LutDeactivated {
        wrapper: user_address_lookup_table.key(),
        lut_address: address_lookup_table.key(),
    });

    Ok(())
}
