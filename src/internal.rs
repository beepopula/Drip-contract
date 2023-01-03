use std::collections::HashMap;

use crate::*;

impl Contract {
    pub(crate) fn internal_set_drip(&mut self, balance: u128, contract_id: AccountId, account_id: AccountId) {
        if get_root_id(contract_id.clone()) == get_root_id(env::current_account_id()) || self.white_list.get(&contract_id).is_some() {
            self.token.internal_deposit(&account_id, balance, &contract_id);
        } 
    }
}