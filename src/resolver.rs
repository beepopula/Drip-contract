use std::collections::HashMap;

use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn resolve_collect(&mut self, collects: Vec<AccountId>) {
        let result_count = env::promise_results_count();
        for i in 0..result_count {
            match env::promise_result(i) {
                near_sdk::PromiseResult::Successful(result) => {
                    let result: HashMap<String, U128> = serde_json::from_slice(&result).unwrap_or(HashMap::new());
                    let contract_id = collects.get(i as usize);
                    if contract_id.is_some() {
                        self.internal_set_drip(result, contract_id.unwrap().clone());
                    }
                },
                _ => continue
            }
        }
    }
}