initSidebarItems({"fn":[["clean_account_after_recovery","During search_addresses_with_funds we created new addresses that don’t have funds, so we remove them again addresses_len was before we generated new addresses in search_addresses_with_funds"],["search_addresses_with_funds","Search addresses with funds `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address has balance, the counter is reset Addresses that got crated during this operation and have a higher key_index than the latest one with balance, will be removed again, to keep the account size smaller"]]});