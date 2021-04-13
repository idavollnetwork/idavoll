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

use crate::{Error,Module, RawEvent, Trait,ModuleId,Finances};
use crate::utils::*;
use crate::rules::{BaseRule,OrgRuleParam};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};


/// An index of a bounty. Just a `u32`.
pub type BountyIndex = u32;

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
    pub fn Vault_balance_of(oid: T::AccountId) -> Result<T::Balance, DispatchError> {
        match Finances::<T>::try_get(oid) {
            Err(e) => Err(Error::<T>::OrganizationNotFound.into()),
            Ok(b) => Ok(b),
        }
    }
    /// transfer the balance to the organization's Vault from the members in the organization
    pub fn transfer_to_Vault(oid: T::AccountId,who: T::AccountId,value: T::Balance) -> DispatchResult {
        let balance = T::Currency::free_balance(&who);
        ensure!(balance >= value,Error::<T>::BalanceLow);
        let Vault_account = Self::account_id();
        T::Currency::transfer(&who,&Vault_account,value,AllowDeath)?;

        Finances::<T>::try_mutate(oid.clone(), |a| -> DispatchResult {
            *a = a.saturating_add(value.clone());
            Ok(())
        })
            .or_else(|_|-> DispatchResult {
                <Balances<T>>::insert(oid.clone(), value.clone());
                Ok(())
            })
    }
    /// transfer the balance to `to` from finance's Vault by Call<> function
    pub fn spend_organization_Vault(oid: T::AccountId,to: T::AccountId,value: T::Balance) -> DispatchResult {
        let Vault_balance = Self::Vault_balance_of(oid.clone())?;
        ensure!(Vault_balance >= value,Error::<T>::BalanceLow);
        let Vault_account = Self::account_id();
        T::Currency::transfer(&Vault_account,&to,value,AllowDeath)?;
        Finances::<T>::try_mutate_exists(oid,|x|{
            *x = x.saturating_sub(value.clone());
            Ok(())
        })
    }
}

impl<T: Trait> BaseFinance<T::AccountId,T::Balance> for Module<T> {
    fn balance_of(oid: AccountId) -> Result<Balance,DispatchError> {
        Self::Vault_balance_of(oid)
    }

    fn reserve_to_org(oid: AccountId,who: AccountId,value: Balance) -> DispatchResult {
        Self::transfer_to_Vault(oid,who,value)
    }
    fn transfer(oid: AccountId,to: AccountId,value: Balance) -> DispatchResult {
        Self::spend_organization_Vault(oid,to,value)
    }
}