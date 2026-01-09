use crate::codama_rust_luts::instructions::CreateAddressLookupTableBuilder;
use crate::common::pda::{derive_address_lookup_table, get_user_address_lookup_table_pda};
use crate::LUTS_ID;
use mollusk_helper::{MolluskContextHelper, ADDRESS_LOOKUP_TABLE_PROGRAM_ID};
use solana_pubkey::Pubkey;

pub fn create_context() -> MolluskContextHelper {
    let elf = std::fs::read("../../target/deploy/luts.so")
        .expect("Failed to read luts.so - run `anchor build` first");

    let alt_elf = std::fs::read("tests/fixtures/address_lookup_table.so")
        .expect("Failed to read address_lookup_table.so - download from mainnet");

    let mut ctx = MolluskContextHelper::new(&LUTS_ID, &elf);
    ctx.add_program(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID, &alt_elf);

    ctx
}

pub fn create_lut(
    ctx: &mut MolluskContextHelper,
    signer: &Pubkey,
    id: u64,
    recent_slot: u64,
) -> (Pubkey, Pubkey) {
    ctx.warp_to_slot(recent_slot);
    ctx.warp_to_slot(recent_slot + 1);

    let (user_address_lookup_table, _) = get_user_address_lookup_table_pda(signer, id);
    let (address_lookup_table, _) =
        derive_address_lookup_table(&user_address_lookup_table, recent_slot);

    let instruction = CreateAddressLookupTableBuilder::new()
        .signer(*signer)
        .address_lookup_table(address_lookup_table)
        .user_address_lookup_table(user_address_lookup_table)
        .recent_slot(recent_slot)
        .id(id)
        .instruction();

    let result = ctx.process_instruction(&instruction);
    assert!(
        result.is_ok(),
        "CreateAddressLookupTable should succeed: {:?}",
        result
    );

    (user_address_lookup_table, address_lookup_table)
}
