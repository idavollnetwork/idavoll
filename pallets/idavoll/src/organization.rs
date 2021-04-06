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
use crate::{Counter, OrgInfos,Proposals,ProposalOf,ProposalIdOf, Module, RawEvent, Trait,
            OrgCount,OrgInfoOf};
use crate::utils::*;
use crate::rules::{BaseRule,OrgRuleParam};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug};
use sp_std::{cmp::PartialOrd,prelude::Vec, collections::btree_map::BTreeMap, marker};
use sp_runtime::traits::Hash;

pub type OrganizationId = u64;
pub trait DefaultAction {
    fn change_organization_name() -> Error;
    fn transfer() -> Error;
}

/// this is the free proposal,every one in the organization can create
/// the proposal for pay a little fee, it not staking any asset to do this.
#[derive(Eq, PartialEq, RuntimeDebug, Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ProposalDetail<AccountId, Balance, BlockNumber>
    where
        AccountId: Ord,
{
    /// A map of voter => (coins, in agree or against)
    pub votes: BTreeMap<AccountId, (Balance, bool)>,
    /// the creator of the proposal
    pub creator: AccountId,
    /// the datetime of the proposal created.
    pub dt: u128,
}

pub type ProposalDetailOf<T> = ProposalDetail<<T as frame_system::Trait>::AccountId,
    <<T as frame_system::Trait>::AccountId>::Balance,<T as frame_system::Trait>::BlockNumber>;

/// This structure is used to encode metadata about an organization.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrgInfo<AccountId, Balance> {
    /// A set of accounts of an organization.
    pub members: Vec<AccountId>,

    /// params for every organization,will set on create organization
    pub param:  OrgRuleParam<Balance>,
    /// Which voting system is in place. `executors` do not need to go
    /// through it due to their higher privilege permission.
    // pub voting: VotingSystem,
}

impl<AccountId: Ord, Balance> OrgInfo<AccountId, Balance> {
    /// Sort all the vectors inside the strutcture.
    pub fn sort(&mut self) {
        self.members.sort();
    }
}

/// Represent a proposal as stored by the pallet.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Proposal<Call, ProposalDetail, OrganizationId> {
    pub org: OrganizationId,
    pub call: Call,
    pub detail: ProposalDetail,
    // pub voting: VotingSystem,
}

impl<Call,Metadata, OrganizationId> Proposal<Call,Metadata, OrganizationId> {
    // get proposal id by hash the content in the proposal
    pub fn id(&mut self) -> Trait::Hash {
        Trait::Hashing::hash_of(&[self.encode()])
    }
}




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
    fn base_create_proposal(oid: T::AccountId,detail: ProposalDetailOf<T>,call: Box<<T as Trait>::Call>) -> dispatch::DispatchResult {

        let proposal = Proposal{
            org:    oid.clone(),
            call:   call.clone(),
            detail: detail.clone(),
        };
        let proposal_id = proposal.clone().id();
        if Proposals::<T>::contains_key(proposal_id) {
            return Err(Error::<T>::ProposalDuplicate.into());
        }
        Proposals::<T>::insert(&proposal_id, proposal.clone());
        Self::deposit_event(RawEvent::ProposalCreated(target_org_id, proposal_id,detail.creator.clone()));
    }
    fn make_proposal_id(proposal: ProposalOf<T>) -> ProposalIdOf<T> {
        proposal.clone().id()
    }
    // add a member into a organization by orgid
    fn base_add_member_on_orgid(oid: T::AccountId,memberID: T::AccountId) -> dispatch::DispatchResult {
        OrgInfo::<T>::try_mutate(oid,|infos| -> dispatch::DispatchResult {
            match infos.members
                .iter()
                .find(|&x| x==memberID) {
                None => {
                    infos.members.push(memberID);
                    Ok(())
                },
                _ => Ok(())
            }
        })
    }
    fn get_proposal_by_id(pid: ProposalIdOf<T>) -> Result<ProposalOf<T>, dispatch::DispatchError> {
        match Proposals::<T>::get(pid) {
            Some(proposal) => Ok(proposal),
            None => Err(Error::<T>::ProposalNotFound.into()),
        }
    }
    fn is_pass(proposal: ProposalOf<T>) -> bool {
        return true
    }
    // a proposal has been voted,it will be finalized once by anyone in org,
    fn base_proposal_finalize(pid: ProposalIdOf<T>) -> dispatch::DispatchResult {
        let proposal = Self::get_proposal_by_id(pid)?;
        if Self::is_pass(proposal.clone()) {
            let call = <T as Trait>::Call::decode(&mut &proposal.clone().call[..]).map_err(|_| Error::<T>::ProposalDecodeFailed)?;
            let res = call.dispatch(frame_system::RawOrigin::Signed(proposal.clone().org).into());
            Self::deposit_event(RawEvent::ProposalFinalized(pid, res.map(|_| ()).map_err(|e| e.error)));
        }
        // remove the proposal
        Proposals::<T>::remove(pid);
        Self::deposit_event(RawEvent::ProposalPassed(pid));
        Ok(())
    }
}