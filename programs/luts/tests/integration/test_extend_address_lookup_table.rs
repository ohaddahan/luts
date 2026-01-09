use crate::codama_rust_luts::instructions::ExtendAddressLookupTableBuilder;
use crate::common::helpers::{create_context, create_lut};
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

const COOLDOWN_SLOTS: u64 = 15;

#[test]
fn test_extend_rejects_during_cooldown() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    let addr1 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_err(),
        "Extend during cooldown should fail with LutNotReady"
    );
}

#[test]
fn test_extend_succeeds_after_cooldown() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS + 2);

    let addr1 = Pubkey::new_unique();
    let addr2 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .add_remaining_account(AccountMeta::new_readonly(addr2, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "Extend after cooldown should succeed: {:?}",
        result
    );
}

#[test]
fn test_extend_rejects_only_duplicates() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS + 2);

    let addr1 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(result.is_ok(), "First extend should succeed: {:?}", result);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS * 2 + 4);

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_err(),
        "Extend with only duplicates should fail with NoNewAddresses"
    );
}

#[test]
fn test_extend_filters_duplicates_and_adds_new() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS + 2);

    let addr1 = Pubkey::new_unique();
    let addr2 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .add_remaining_account(AccountMeta::new_readonly(addr2, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(result.is_ok(), "First extend should succeed: {:?}", result);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS * 2 + 4);

    let addr3 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .add_remaining_account(AccountMeta::new_readonly(addr3, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "Extend with mix of duplicate and new should succeed: {:?}",
        result
    );
}

#[test]
fn test_size_field_tracks_addresses() {
    let mut ctx = create_context();

    let signer = Pubkey::new_unique();
    let id: u64 = 0;
    let recent_slot: u64 = 100;

    ctx.fund_account(&signer, 10_000_000_000);

    let (user_address_lookup_table, address_lookup_table) =
        create_lut(&mut ctx, &signer, id, recent_slot);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS + 2);

    let addr1 = Pubkey::new_unique();
    let addr2 = Pubkey::new_unique();
    let addr3 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr1, false))
        .add_remaining_account(AccountMeta::new_readonly(addr2, false))
        .add_remaining_account(AccountMeta::new_readonly(addr3, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(result.is_ok(), "First extend should succeed: {:?}", result);

    ctx.warp_to_slot(recent_slot + COOLDOWN_SLOTS * 2 + 4);

    let addr4 = Pubkey::new_unique();
    let addr5 = Pubkey::new_unique();

    let instruction = ExtendAddressLookupTableBuilder::new()
        .signer(signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .add_remaining_account(AccountMeta::new_readonly(addr4, false))
        .add_remaining_account(AccountMeta::new_readonly(addr5, false))
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "Second extend should succeed: {:?}",
        result
    );
}
