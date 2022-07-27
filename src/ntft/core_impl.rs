
use near_contract_standards::fungible_token::events::{FtMint, FtBurn, FtTransfer};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::json;
use near_sdk::{
    assert_one_yocto, env, log, require, AccountId, Balance, Gas, IntoStorageKey, PromiseOrValue,
    PromiseResult, StorageUsage,
};

use crate::ntft::receiver::ext_ft_receiver;
use crate::ntft::resolver::ext_ft_resolver;

use super::core::FungibleTokenCore;
use super::resolver::FungibleTokenResolver;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

/// Implementation of a FungibleToken standard.
/// Allows to include NEP-141 compatible token to any contract.
/// There are next traits that any contract may implement:
///     - FungibleTokenCore -- interface with ft_transfer methods. FungibleToken provides methods for it.
///     - FungibleTokenMetaData -- return metadata for the token in NEP-148, up to contract to implement.
///     - StorageManager -- interface for NEP-145 for allocating storage per account. FungibleToken provides methods for it.
///     - AccountRegistrar -- interface for an account to register and unregister
///
/// For example usage, see examples/fungible-token/src/lib.rs.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    /// AccountID -> Account balance.
    pub accounts: LookupMap<AccountId, UnorderedMap<Option<AccountId>, Balance>>,

    /// Total supply of the all token.
    pub total_supply: Balance,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,
}

impl FungibleToken {
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let mut this =
            Self { accounts: LookupMap::new(prefix), total_supply: 0, account_storage_usage: 0 };
        this.measure_account_storage_usage();
        this
    }

    fn measure_account_storage_usage(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = AccountId::new_unchecked("a".repeat(64));
        self.accounts.insert(&tmp_account_id, &UnorderedMap::new("contracts".as_bytes()));
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        env::storage_remove("contracts".as_bytes());
        self.accounts.remove(&tmp_account_id);
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance, contract_id: Option<AccountId>) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let balance = account.get(&contract_id).expect(format!("The contract {} is not registered", &account_id.to_string()).as_str());
        if let Some(new_balance) = balance.checked_add(amount) {
            account.insert(&contract_id, &new_balance);
            self.accounts.insert(account_id, &account);
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
            FtMint {
                owner_id: account_id,
                amount: &amount.into(),
                memo: Some(&json!(contract_id).to_string()),
            }
            .emit();
        } else {
            env::panic_str("Balance overflow");
        }
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance, contract_id: Option<AccountId>) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let balance = account.get(&contract_id).expect(format!("The contract {} is not registered", &account_id.to_string()).as_str());
        if let Some(new_balance) = balance.checked_sub(amount) {
            account.insert(&contract_id, &new_balance);
            self.accounts.insert(account_id, &account);
            self.total_supply = self
                .total_supply
                .checked_sub(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
            FtBurn {
                owner_id: account_id,
                amount: &amount.into(),
                memo: Some(&json!(contract_id).to_string()),
            }
            .emit();
        } else {
            env::panic_str("The account doesn't have enough balance");
        }
    }


    pub fn internal_register_account(&mut self, account_id: &AccountId, contract_id: Option<AccountId>) {
        let contract_id = match contract_id {
            Some(v) => v.to_string(),
            None => "".to_string()
        };
        if self.accounts.insert(account_id, &UnorderedMap::new((account_id.to_string() + &contract_id).as_bytes())).is_some() {
            env::panic_str("The account is already registered");
        }
    }
    
}

impl FungibleTokenCore for FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        unreachable!()
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        unreachable!()
    }

    fn ft_burn_call(
        &mut self,
        contract_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        require!(env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL, "More gas is required");
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_withdraw(&sender_id, amount, Some(contract_id.clone()));
        // Initiating receiver's call and the callback
        ext_ft_receiver::ext(contract_id.clone())
        .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL)
        .ft_on_burn(sender_id.clone(), amount.into(), msg)
        .then(
            ext_ft_resolver::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                .ft_resolve_burn(sender_id, amount.into(), contract_id),
        )
        .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        match self.accounts.get(&account_id) {
            Some(account) => account.get(&None).unwrap_or(0).into(),
            None => 0.into()
        }
    }
}

impl FungibleToken {
    /// Internal method that returns the amount of burned tokens in a corner case when the sender
    /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount)
    pub fn internal_ft_resolve_burn(
        &mut self,
        owner_id: &AccountId,
        amount: U128,
        contract_id: AccountId
    ) -> (u128, u128) {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let refund_amount = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if refund_amount > 0 {
            self.internal_deposit(owner_id, refund_amount, Some(contract_id));
            return (amount - refund_amount, 0);
        }
        (amount, 0)
    }
}

impl FungibleTokenResolver for FungibleToken {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        unreachable!()
    }

    fn ft_resolve_burn(
        &mut self,
        owner_id: AccountId,
        amount: U128,
        contract_id: AccountId
    ) -> U128 {
        self.internal_ft_resolve_burn(&owner_id, amount, contract_id).0.into()
    }

}

impl FungibleToken {
    pub fn ft_balance_by_contract(&self, account_id: &AccountId, contract_id: Option<AccountId>) -> U128 {
        match self.accounts.get(&account_id) {
            Some(account) => account.get(&contract_id).unwrap_or(0).into(),
            None => 0.into()
        }
    }  
}
