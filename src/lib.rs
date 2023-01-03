/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/

use near_non_transferrable_token::fungible_token::core_impl::{FungibleToken, Account};
use near_non_transferrable_token::fungible_token::core::{FungibleTokenCore};
use near_non_transferrable_token::fungible_token::account::FungibleTokenAccount;
use near_non_transferrable_token::fungible_token::sender::FungibleTokenSender;
use near_non_transferrable_token::fungible_token::resolver::FungibleTokenResolver;
use near_non_transferrable_token::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

use near_non_transferrable_token::{impl_fungible_token_core, impl_fungible_token_storage};
use near_non_transferrable_token::storage_management::{
    StorageManagement, StorageBalance, StorageBalanceBounds
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet, LookupMap};
use near_sdk::json_types::{U128};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::{json, self};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue, Promise, Gas, bs58, base64};
use utils::{get_root_id};
use std::collections::{HashSet, HashMap};
use std::convert::TryFrom;
use std::str::FromStr;


pub mod utils;
pub mod resolver;
pub mod internal;
pub mod view;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
    white_list: HashSet<AccountId>
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
    coe_map: UnorderedMap<String, u128>,
}



const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/png;base64,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";

const THIS_FUNCTION_CALL_GAS: u64  = 50_000_000_000_000;
const COLLECT_DRIP_GAS: u64 = 10_000_000_000_000;
const RESOLVE_COLLECT_DRIP_GAS_BASE: u64 = 3_000_000_000_000;
const RESOLVE_COLLECT_DRIP_GAS_X: u64 = 2_000_000_000_000;

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta() -> Self {
        Self::new(
            get_root_id(env::current_account_id()),
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Popula Drip".to_string(),
                symbol: "DRIP".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner_id,
            white_list: HashSet::new()
        };
        this
    }

    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_contract: OldContract = env::state_read().unwrap();
        assert!(env::predecessor_account_id() == old_contract.owner_id, "not owner");
        env::storage_remove("STATE".as_bytes());
        Self {
            token: old_contract.token,
            metadata: old_contract.metadata,
            owner_id: old_contract.owner_id,
            white_list: HashSet::new()
        }
    }

    #[init(ignore_state)]
    pub fn renew() -> Self {
        let old_contract: Contract = env::state_read().unwrap();
        assert!(env::predecessor_account_id() == old_contract.owner_id, "not owner");
        env::storage_remove(&b"m".to_vec());
        old_contract
    }

    pub fn set_white_list(&mut self, contract_id: AccountId, del: bool) {
        match del {
            true => self.white_list.remove(&contract_id),
            false => self.white_list.insert(contract_id)
        };
    }

    #[payable]
    pub fn ft_collect(&mut self, collects: Vec<AccountId>) {
        let sender_id = env::predecessor_account_id();

        let storage_balance = match self.token.storage_balance_of(sender_id.clone()) {
            Some(v) => v.available.0,
            None => {
                assert!(self.token.account_storage_usage as u128 * env::storage_byte_cost() < env::attached_deposit(), "not registered");
                self.token.internal_register_account(&sender_id);
                0
            }
        };

        let account = self.token.accounts.get(&sender_id).unwrap();
        let mut unregister_count = 0;
        let collects: Vec<AccountId> = collects.into_iter().filter(|contract_id| {
            if get_root_id(contract_id.clone()) == get_root_id(env::current_account_id()) || self.white_list.get(&contract_id).is_some() {
                if account.is_registered(&contract_id) == false {
                    unregister_count += 1;
                }
                true
            } else {
                false
            }
        }).collect();
        assert!(self.token.account_storage_usage as u128 * env::storage_byte_cost() * unregister_count <= env::attached_deposit() + storage_balance, "not enough deposit");

        assert!(collects.len() as u64 * (COLLECT_DRIP_GAS + RESOLVE_COLLECT_DRIP_GAS_X) + RESOLVE_COLLECT_DRIP_GAS_BASE < (env::prepaid_gas() - Gas::from(THIS_FUNCTION_CALL_GAS)).0, "not enough gas");

        let mut promises: Vec<u64> = Vec::new();
        for contract_id in collects.clone() {
            let new_promise = env::promise_create(contract_id.clone(), "collect_drip", json!({
            }).to_string().as_bytes(), 1, COLLECT_DRIP_GAS.into());
            promises.push(new_promise);
        }

        let remain_gas = env::prepaid_gas() - env::used_gas() - Gas::from(collects.len() as u64 * COLLECT_DRIP_GAS + RESOLVE_COLLECT_DRIP_GAS_BASE);
        let batch_promise = env::promise_and(&promises[..]);
        env::promise_then(batch_promise, env::current_account_id(), "resolve_collect", json!({
            "collects": collects,
            "account_id": sender_id
        }).to_string().as_bytes(), 0, remain_gas);

        assert!(promises.len() > 0, "failed");
    }

}

impl_fungible_token_core!(Contract, token);
impl_fungible_token_storage!(Contract, token);


#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
