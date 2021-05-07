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

/// the voting module
///
///

use frame_support::{
    ensure,dispatch::{DispatchResult},
};
use crate::{OrgCounter, OrgInfos,Proposals,ProposalOf,ProposalIdOf, Error,Module, RawEvent, Trait,
            OrgCount,OrgInfoOf,BalanceOf};
use crate::utils::*;
use crate::rules::{BaseRule,OrgRuleParam};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug};
use sp_std::{cmp::PartialOrd,prelude::Vec, collections::btree_map::BTreeMap, marker};
use sp_runtime::traits::Hash;
use idavoll_asset::{token::BaseToken,finance::BaseFinance};


impl<T: Trait> Module<T> {
    /// lock the voter's balance for voting the proposal,it can auto unlocked
    /// when the proposal was closed
    pub fn vote_on_proposal(oid: T::AccountId,
                            pid: ProposalIdOf<T>,
                            voter: T::AccountId,
                            value: BalanceOf<T>,
                            yesorno: bool,
                            height: T::BlockNumber,
    ) -> DispatchResult {
        if !Self::is_member(oid.clone(),&voter) {
            return Err(Error::<T>::NotMember.into());
        }
        let oinfo = Self::get_orginfo_by_id(oid.clone())?;
        let proposal = Self::get_proposal_by_id(pid)?;
        if proposal.detail.is_expire(height) {
            return Err(Error::<T>::ProposalExpired.into());
        }
        // lock the voter's balance
        let aid = oinfo.get_asset_id();
        T::AssetHandle::lock(aid.clone(),&voter.clone(),value)?;
        Self::base_vote_on_proposal(pid,voter,value,yesorno)?;
        // check the proposal can closed
        Self::try_close_proposal(aid.clone(),pid.clone(),height)
    }
    /// close the proposal when the proposal was expire or passed.
    /// it will auto unlocked the voter's asset
    pub fn try_close_proposal(aid: T::AssetId,pid: ProposalIdOf<T>,height: T::BlockNumber) -> DispatchResult {
        let proposal = Self::get_proposal_by_id(pid)?;
        let is_expire = proposal.detail.is_expire(height);
        let is_pass = Self::is_pass(proposal.clone());
        if is_pass {
            Self::base_call_dispatch(pid.clone(),proposal.clone())?;
        }
        if is_expire || is_pass {
            let clone_proposal = proposal.clone();
            Self::remove_proposal_by_id(pid.clone());
            clone_proposal.detail.votes.iter().for_each(|val|{
                T::AssetHandle::unlock(aid,&val.0.clone(),val.1.0.clone());
            });
            Self::deposit_event(RawEvent::ProposalPassed(pid));
        }
        Ok(())
    }
    /// create new token with new organization
    pub fn create_new_token(owner: T::AccountId,total: T::Balance) -> T::AssetId {
        T::AssetHandle::create(owner,total)
    }

    pub fn handle_transfer_by_decision(oid: T::AccountId,to: T::AccountId,value: T::Balance) -> DispatchResult {
        T::Finance::transfer_by_vault(oid.clone(),to.clone(),value)
    }
}