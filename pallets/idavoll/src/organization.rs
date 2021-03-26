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

/// Organization represents an organization
///
///

use frame_support::{
	ensure,dispatch,
};
use crate::{Counter, OrgInfos,Proposals, Module, RawEvent, Trait,
            OrgCount,OrgInfoOf};
use crate::utils::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug};
use sp_std::prelude::Vec;

pub trait DefaultAction {

    fn change_organization_name() -> Error;
    fn transfer() -> Error;
}

/// This structure is used to encode metadata about an organization.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrgInfo<AccountId, VotingSystem> {
    /// A set of accounts of an organization.
    pub members: Vec<AccountId>,

    /// Which voting system is in place. `executors` do not need to go
    /// through it due to their higher privilege permission.
    // pub voting: VotingSystem,
}

impl<AccountId: Ord, VotingSystem> OrgInfo<AccountId, VotingSystem> {
    /// Sort all the vectors inside the strutcture.
    pub fn sort(&mut self) {
        self.members.sort();
    }
}

/// Represent a proposal as stored by the pallet.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Proposal<Call, Metadata, OrganizationId, VotingSystem> {
    pub org: OrganizationId,
    pub call: Call,
    pub metadata: Metadata,
    pub voting: VotingSystem,
}

pub type OrganizationId = u64;


impl<T: Trait> Module<T> {
    // be accountid of organization id for orginfos in the storage
    pub fn counter2Orgid(c: OrgCount) -> T::AccountId {
        Self::ModuleId.into_sub_account(c)
    }
    fn create_org(oinfo: OrgInfoOf<T>) -> dispatch::DispatchResult {
        let counter = Counter::<T>::get();
        let new_counter = counter.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
        let oid = Self::counter2Orgid(counter);

        OrgInfos::<T>::insert(&oid, oinfo.clone());

        Self::deposit_event(RawEvent::OrganizationCreated(org_id, details));
        Counter::<T>::put(new_counter);
        Ok(())
    }
    fn add_member_on_orgid(oid: T::AccountId) -> dispatch::DispatchResult {
        Ok(())
    }
}