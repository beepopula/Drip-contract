use std::collections::HashMap;

use crate::*;

impl Contract {
    pub(crate) fn internal_set_drip(&mut self, balance: u128, contract_id: AccountId, token_source: TokenSource, account_id: AccountId) {
        self.token.internal_deposit(&account_id, balance, &contract_id, &token_source);
    }
}