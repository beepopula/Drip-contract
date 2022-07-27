use std::collections::HashMap;

use crate::*;

impl Contract {
    pub(crate) fn internal_set_drip(&mut self, drip_map: HashMap<String, U128>, contract_id: AccountId, account_id: AccountId) {
        let mut drip = 0 as u128;
        for (key, value) in drip_map.iter() {
            let coe = self.coe_map.get(&key).unwrap_or(1 as u128);
            drip += (value.0) * coe;
        }
        self.token.internal_deposit(&account_id, drip, Some(contract_id));
    }
}