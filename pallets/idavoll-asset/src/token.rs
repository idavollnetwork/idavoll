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

use codec::FullCodec;
use frame_support::traits::{BalanceStatus, LockIdentifier};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize,Saturating},
    DispatchError, DispatchResult,
};
use sp_std::{
    cmp::{Eq, PartialEq},
    fmt::Debug,
    result,
};
use crate::{Trait,Module};


/// Abstraction trait over a multiple currencies system, each currency type
/// is identified by a `AssetId`, if it is set to `None` when calling a
/// function the implementation should default to the native currency of the
/// runtime.
pub trait BaseToken<AccountId> {
    /// The type used to identify currencies
    type AssetId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug + Default;

    /// The balance of an account.
    type Balance: AtLeast32BitUnsigned
    + FullCodec
    + Copy
    + MaybeSerializeDeserialize
    + Debug
    + Default;

    // PUBLIC IMMUTABLES

    /// create the new token
    fn create(owner: AccountId,total: Self::Balance) -> Self::AssetId;
    /// The total amount of the asset.
    fn total(aid: Self::AssetId) -> Self::Balance;

    /// Reduce the total tokens by `amount` and remove the `amount` tokens by `who`'s
    /// account, return error if not enough tokens.
    fn burn(aid: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

    /// Increase the balance of `who` by `amount`,if there have the permission。
    fn mint(aid: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

    /// The 'free' balance of a given account.
    fn free_balance_of(aid: Self::AssetId, who: &AccountId) -> Self::Balance;
    /// The 'lock' balance of a given account.
    fn lock_balance_of(aid: Self::AssetId, who: &AccountId) -> Self::Balance;
    /// The total balance of a given account.
    fn total_balance_of(aid: Self::AssetId, who: &AccountId) -> Self::Balance;

    /// Transfer some free balance to another account.
    fn transfer(aid: Self::AssetId, from: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;

    /// Lock `value` from the free balance,return `Err` if the free balance is lower than `value`.
    /// otherwise return `ok`.
    fn lock(aid: Self::AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;

    /// Unlock `value` from locked balance to free balance. This function cannot fail.
    /// If the locked balance of `who` is less than `value`, then the remaining amount will be returned.
    fn unlock(aid: Self::AssetId, who: &AccountId, value: Self::Balance) -> Self::Balance;
}


impl<T: Trait> BaseToken<T::AccountId> for Module<T> {
    type AssetId = T::AssetId;
    type Balance = T::Balance;


    fn create(owner: T::AccountId,total: Self::Balance) -> Self::AssetId {
        Self::create_token(owner,total)
    }
    fn total(aid: Self::AssetId) -> Self::Balance {
        Self::total_issuances(aid)
    }
    /// The 'free' balance of a given account.
    fn free_balance_of(aid: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        Self::free_balance(aid,&who)
    }
    /// The 'lock' balance of a given account.
    fn lock_balance_of(aid: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        let all = Self::total_balance(aid,who);
        let free = Self::free_balance(aid,who);
        return all.saturating_sub(free)
    }
    /// The total balance of a given account.
    fn total_balance_of(aid: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        Self::total_balance(aid,who)
    }

    /// Reduce the total tokens by `amount` and remove the `amount` tokens by `who`'s
    /// account, return error if not enough tokens.
    fn burn(aid: Self::AssetId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        Self::base_burn(aid,who,amount)
    }
    /// Increase the balance of `who` by `amount`,if there have the permission。
    fn mint(aid: Self::AssetId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        Self::base_mint(aid,who,amount)
    }

    /// Transfer some free balance to another account.
    fn transfer(aid: Self::AssetId, from: &T::AccountId, to: &T::AccountId, value: Self::Balance) -> DispatchResult {
        Self::base_transfer(aid,from,to,value)
    }

    /// Lock `value` from the free balance,return `Err` if the free balance is lower than `value`.
    /// otherwise return `ok`.
    fn lock(aid: Self::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
        Self::base_lock(aid,who,value)
    }

    /// Unlock `value` from locked balance to free balance. This function cannot fail.
    /// If the locked balance of `who` is less than `value`, then the remaining amount will be returned.
    fn unlock(aid: Self::AssetId, who: &T::AccountId, value: Self::Balance) -> Self::Balance{
        Self::base_unlock(aid,who,value)
    }
}