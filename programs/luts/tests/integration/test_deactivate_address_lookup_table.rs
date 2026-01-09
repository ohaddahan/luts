use crate::codama_rust_luts::instructions::DeactivateAddressLookupTableBuilder;
use crate::common::helpers::{create_context, create_lut};
use solana_pubkey::Pubkey;

#[test]
fn test_deactivate_address_lookup_table() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    let instruction = DeactivateAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "DeactivateAddressLookupTable should succeed: {:?}",
        result
    );
}

#[test]
fn test_deactivate_wrong_signer_fails() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let wrong_signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);
    ctx.fund_account(&wrong_signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    let instruction = DeactivateAddressLookupTableBuilder::new()
        .signer(wrong_signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_err(),
        "DeactivateAddressLookupTable with wrong signer should fail"
    );
}
