
use crate::*;

pub(crate) fn get_root_id(contract_id: AccountId) -> AccountId {
    let contract_id = contract_id.to_string();
    //let index = contract_id.find('.').unwrap();
    let arr: Vec<String> = contract_id.split('.').map(|v| v.to_string()).collect();
    //let parent_id = contract_id[index + 1..].to_string();
    let root_id = arr.get(arr.len() - 2).unwrap().clone() + "." + arr.get(arr.len() - 1).unwrap();
    AccountId::try_from(root_id).unwrap()
}
