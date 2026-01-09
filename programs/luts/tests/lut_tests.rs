#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use luts::state::user_address_lookup_table::UserAddressLookupTable;

    #[test]
    fn test_user_address_lookup_table_is_ready() {
        let wrapper = UserAddressLookupTable {
            bump: 255,
            id: 1,
            signer: Pubkey::new_unique(),
            address_lookup_table: Pubkey::new_unique(),
            accounts: Vec::new(),
            last_updated_slot: 100,
            last_updated_timestamp: 1000,
        };

        assert!(!wrapper.is_ready(100));
        assert!(!wrapper.is_ready(200));
        assert!(wrapper.is_ready(250));
        assert!(wrapper.is_ready(300));
    }

    #[test]
    fn test_user_address_lookup_table_slots_until_ready() {
        let wrapper = UserAddressLookupTable {
            bump: 255,
            id: 1,
            signer: Pubkey::new_unique(),
            address_lookup_table: Pubkey::new_unique(),
            accounts: Vec::new(),
            last_updated_slot: 100,
            last_updated_timestamp: 1000,
        };

        assert_eq!(wrapper.slots_until_ready(100), 150);
        assert_eq!(wrapper.slots_until_ready(200), 50);
        assert_eq!(wrapper.slots_until_ready(250), 0);
        assert_eq!(wrapper.slots_until_ready(300), 0);
    }

    #[test]
    fn test_user_address_lookup_table_contains() {
        let addr1 = Pubkey::new_unique();
        let addr2 = Pubkey::new_unique();
        let addr3 = Pubkey::new_unique();

        let wrapper = UserAddressLookupTable {
            bump: 255,
            id: 1,
            signer: Pubkey::new_unique(),
            address_lookup_table: Pubkey::new_unique(),
            accounts: vec![addr1, addr2],
            last_updated_slot: 100,
            last_updated_timestamp: 1000,
        };

        assert!(wrapper.contains(&addr1));
        assert!(wrapper.contains(&addr2));
        assert!(!wrapper.contains(&addr3));
    }

    #[test]
    fn test_user_address_lookup_table_address_count() {
        let wrapper = UserAddressLookupTable {
            bump: 255,
            id: 1,
            signer: Pubkey::new_unique(),
            address_lookup_table: Pubkey::new_unique(),
            accounts: vec![
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            ],
            last_updated_slot: 100,
            last_updated_timestamp: 1000,
        };

        assert_eq!(wrapper.address_count(), 3);
    }

    #[test]
    fn test_user_address_lookup_table_remaining_capacity() {
        let wrapper = UserAddressLookupTable {
            bump: 255,
            id: 1,
            signer: Pubkey::new_unique(),
            address_lookup_table: Pubkey::new_unique(),
            accounts: vec![Pubkey::new_unique(); 10],
            last_updated_slot: 100,
            last_updated_timestamp: 1000,
        };

        assert_eq!(wrapper.remaining_capacity(), 246);
    }

    #[test]
    fn test_user_address_lookup_table_size_constant() {
        assert!(UserAddressLookupTable::SIZE > 0);
        assert_eq!(UserAddressLookupTable::COOLDOWN_SLOTS, 150);
        assert_eq!(UserAddressLookupTable::MAX_ADDRESSES, 256);
    }
}
