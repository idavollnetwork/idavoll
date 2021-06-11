/*
 * Copyright 2021 Idavoll Network
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use frame_support::{
    dispatch::{DispatchResult,DispatchError},
    traits::{Get},
};
use sp_runtime::{
    traits::{AccountIdConversion},
};
use crate::{Module,Trait,LocalBalance};



/// An index of a bounty. Just a `u32`.
pub type BountyIndex = u32;

/// the trait implement the Function of hosting user organization local assets(IDV).
/// the member donate the local assets to the organization, it managed by the idv-asset
/// pallet's module named 'Finance',the Finance will only be access the local asset(IDV),
/// Although the local asset belongs to the organization, the organization cannot transfer it,
/// the Vault was transfer only by the Finance.
///
pub trait BaseFinance<AccountId,Balance> {
    /// get the balance(for local idv asset) by the id(organization id), the return was
    /// the balance(record in to the idv-asset pallet storage), the real asset is storage
    /// to the pallet_balance pallet
    fn balance_of(oid: AccountId) -> Result<Balance,DispatchError>;
    /// the asset(idv) donated by the member of the organization,it will be transfer to the the account by
    /// ModuleID of the the pallet, and record to the storage of the pallet with the organization id
    fn reserve_to_org(oid: AccountId,who: AccountId,value: Balance) -> DispatchResult;
    /// transfer the asset(idv) to the user account, and reduce the organization's amount
    fn transfer_by_vault(oid: AccountId,to: AccountId,value: Balance) -> DispatchResult;
    /// get the locked balance(for local idv asset) by the user who create the proposal,
    /// the real asset is storage to the pallet_balance pallet
    fn locked_balance_of(oid: AccountId,who: AccountId) -> Result<Balance,DispatchError>;
    /// lock the balance by the user
    fn lock_balance(oid: AccountId,who: AccountId,value: Balance) -> DispatchResult;
    /// unlock the balance by the user
    fn unlock_balance(oid: AccountId,who: AccountId,value: Balance)-> DispatchResult;
}

impl<T: Trait> Module<T> {
    /// The account ID of the idv-asset pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }
    /// The account ID of a bounty account
    pub fn bounty_account_id(id: BountyIndex) -> T::AccountId {
        // only use two byte prefix to support 16 byte account id (used by test)
        // "modl" ++ "py/idv" ++ "bt" is 12 bytes, and four bytes remaining for bounty index
        T::ModuleId::get().into_sub_account(("bt", id))
    }
}

impl<T: Trait> BaseFinance<T::AccountId,LocalBalance<T>> for Module<T> {

    fn balance_of(oid: T::AccountId) -> Result<LocalBalance<T>,DispatchError> {
        Self::vault_balance_of(oid)
    }
    fn reserve_to_org(oid: T::AccountId,who: T::AccountId,value: LocalBalance<T>) -> DispatchResult {
        Self::transfer_to_vault(oid,who,value)
    }
    fn transfer_by_vault(oid: T::AccountId,to: T::AccountId,value: LocalBalance<T>) -> DispatchResult {
        Self::spend_organization_vault(oid,to,value)
    }
    fn locked_balance_of(oid: T::AccountId,who: T::AccountId) -> Result<LocalBalance<T>,DispatchError> {
        Self::vault_locked_balance_of(oid, who)
    }
    fn lock_balance(oid: T::AccountId,who: T::AccountId,value: LocalBalance<T>) -> DispatchResult {
        Self::vault_lock_asset(oid, who, value)
    }
    fn unlock_balance(oid: T::AccountId,who: T::AccountId,value: LocalBalance<T>)-> DispatchResult {
        Self::vault_unlock_asset(oid, who, value)
    }
}