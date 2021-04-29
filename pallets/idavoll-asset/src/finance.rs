// This file is part of Idavoll Network.

// Copyright (C) 2021 Idavoll Network.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// the finance module

use frame_support::{
    ensure,dispatch::{DispatchResult,DispatchError},
    traits::{Get,ExistenceRequirement::AllowDeath},
};
use sp_runtime::{RuntimeDebug,
                 traits::{AccountIdConversion,Hash},
};
use sp_std::{cmp::PartialOrd,prelude::Vec, collections::btree_map::BTreeMap, marker};
use crate::{Error,Module, RawEvent,Finances, Trait,ModuleId,LocalBalance};
// #[cfg(feature = "std")]
// use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};


/// An index of a bounty. Just a `u32`.
pub type BountyIndex = u32;

/// the trait implement the Function of hosting user organization local assets(IDV).
/// the member donate the local assets to the organization, it managed by the idv-asset
/// pallet's module named 'Finance',the Finance will only be access the local asset(IDV),
/// Although the local asset belongs to the organization, the organization cannot transfer it,
/// the Vault was transfer only by the Finance.
pub trait BaseFinance<AccountId,Balance> {
    /// return the balance of the organization's Vault,it reserved by the members of the organization
    fn balance_of(oid: AccountId) -> Result<Balance,DispatchError>;
    ///
    fn reserve_to_org(oid: AccountId,who: AccountId,value: Balance) -> DispatchResult;
    fn transfer(oid: AccountId,to: AccountId,value: Balance) -> DispatchResult;
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
    /// get the balance(for local idv asset) by the id(organization id), the return was
    /// the balance(record in to the idv-asset pallet storage), the real asset is storage
    /// to the pallet_balance pallet
    fn balance_of(oid: T::AccountId) -> Result<LocalBalance<T>,DispatchError> {
        Self::Vault_balance_of(oid)
    }
    /// the asset(idv) donated by the member of the organization,it will be transfer to the the account by
    /// ModuleID of the the pallet, and record to the storage of the pallet with the organization id
    fn reserve_to_org(oid: T::AccountId,who: T::AccountId,value: LocalBalance<T>) -> DispatchResult {
        Self::transfer_to_Vault(oid,who,value)
    }
    /// transfer the asset(idv) to the user account, and reduce the organization's amount
    fn transfer(oid: T::AccountId,to: T::AccountId,value: LocalBalance<T>) -> DispatchResult {
        Self::spend_organization_Vault(oid,to,value)
    }
}