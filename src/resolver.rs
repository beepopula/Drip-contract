
use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn resolve_collect(&mut self, collects: Vec<AccountId>, account_id: AccountId) {
        let result_count = env::promise_results_count();
        for i in 0..result_count {
            match env::promise_result(i) {
                near_sdk::PromiseResult::Successful(result) => {
                    let result: U128 = serde_json::from_slice(&result).unwrap_or(0.into());
                    let contract_id = collects.get(i as usize);
                    if contract_id.is_some() {
                        self.internal_set_drip(result.0, contract_id.unwrap().clone(), TokenSource::Building, account_id.clone(), );
                    }
                },
                _ => continue
            }
        }
    }
}