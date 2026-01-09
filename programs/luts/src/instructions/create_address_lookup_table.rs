use crate::constants::LutProgram;
use crate::error::LutError;
use crate::events::LutCreated;
use crate::state::user_address_lookup_table::UserAddressLookupTable;
use anchor_lang::prelude::*;
use solana_address_lookup_table_interface::instruction::create_lookup_table;

/// Arguments for creating a new Address Lookup Table.
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateAddressLookupTableArgs {
    /// A recent slot used to derive the LUT address (must be within ~150 slots of current).
    pub recent_slot: u64,
    /// User-defined identifier allowing multiple LUTs per signer.
    pub id: u64,
}

/// Creates a new Address Lookup Table with an associated wrapper PDA.
///
/// The wrapper PDA becomes the authority of the native LUT, allowing this program
/// to manage extensions and lifecycle. The LUT address is deterministically derived
/// from the wrapper PDA and the recent_slot.
#[derive(Accounts)]
#[instruction(args: CreateAddressLookupTableArgs)]
pub struct CreateAddressLookupTable<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub address_lookup_table_program: Program<'info, LutProgram>,
    /// CHECK: Validated inside instruction
    #[account(mut)]
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [UserAddressLookupTable::SEED.as_bytes(), signer.key().as_ref(), args.id.to_le_bytes().as_ref()],
        space = UserAddressLookupTable::SIZE,
        bump
    )]
    pub user_address_lookup_table: Box<Account<'info, UserAddressLookupTable>>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_address_lookup_table(
    ctx: Context<CreateAddressLookupTable>,
    args: CreateAddressLookupTableArgs,
) -> Result<()> {
    let signer = &ctx.accounts.signer;
    let user_address_lookup_table = &mut ctx.accounts.user_address_lookup_table;
    let address_lookup_table = &ctx.accounts.address_lookup_table;
    user_address_lookup_table.bump = ctx.bumps.user_address_lookup_table;
    user_address_lookup_table.id = args.id;
    user_address_lookup_table.signer = signer.key();
    user_address_lookup_table.address_lookup_table = address_lookup_table.key();
    user_address_lookup_table.size = 0;
    let clock = Clock::get()?;
    user_address_lookup_table.last_updated_slot = clock.slot;
    let (ix, address) = create_lookup_table(
        user_address_lookup_table.key(),
        signer.key(),
        args.recent_slot,
    );
    require_keys_eq!(
        address,
        address_lookup_table.key(),
        LutError::InvalidLookupTable
    );
    program::invoke(
        &ix,
        &[
            signer.to_account_info(),
            address_lookup_table.to_account_info(),
            user_address_lookup_table.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.address_lookup_table_program.to_account_info(),
        ],
    )?;
    emit!(LutCreated {
        wrapper: user_address_lookup_table.key(),
        lut_address: address_lookup_table.key(),
        authority: signer.key(),
        slot: clock.slot,
    });
    Ok(())
}
