use crate::codama_rust_luts::instructions::{
    CloseAddressLookupTableBuilder, DeactivateAddressLookupTableBuilder,
};
use crate::common::helpers::{create_context, create_lut};
use mollusk_helper::MolluskContextHelper;
use solana_pubkey::Pubkey;

fn deactivate_lut(
    ctx: &MolluskContextHelper,
    signer: &Pubkey,
    user_address_lookup_table: &Pubkey,
    address_lookup_table: &Pubkey,
) {
    let instruction = DeactivateAddressLookupTableBuilder::new()
        .signer(*signer)
        .address_lookup_table(*address_lookup_table)
        .user_address_lookup_table(*user_address_lookup_table)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "DeactivateAddressLookupTable should succeed: {:?}",
        result
    );
}

#[test]
fn test_close_fails_without_deactivation() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    let instruction = CloseAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_err(),
        "CloseAddressLookupTable without deactivation should fail"
    );
}

#[test]
fn test_close_fails_immediately_after_deactivation() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    deactivate_lut(
        &ctx,
        &signer,
        &user_address_lookup_table,
        &address_lookup_table,
    );

    let instruction = CloseAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_err(),
        "CloseAddressLookupTable immediately after deactivation should fail"
    );
}

#[test]
fn test_close_succeeds_after_deactivation_period() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    ctx.warp_to_slot(recent_slot + 10);

    deactivate_lut(
        &ctx,
        &signer,
        &user_address_lookup_table,
        &address_lookup_table,
    );

    ctx.warp_to_slot(recent_slot + 513 + 10);

    let instruction = CloseAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "CloseAddressLookupTable after deactivation period should succeed: {:?}",
        result
    );

    let lut_account = ctx.get_account(&address_lookup_table);
    if let Some(account) = lut_account {
        assert_eq!(
            account.lamports, 0,
            "AddressLookupTable account should have zero lamports"
        );
    }
}
