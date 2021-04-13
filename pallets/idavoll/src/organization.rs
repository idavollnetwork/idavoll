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
    sp_std::collections::btree_map::BTreeMap,
};
use crate::{Counter, OrgInfos,Proposals,ProposalOf,ProposalIdOf,Error,
            Module, RawEvent, Trait,
            OrgCount,OrgInfoOf};
use crate::utils::*;
use crate::rules::{BaseRule,OrgRuleParam};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use sp_runtime::{
    RuntimeDebug,traits::{Saturating, Zero,Hash},
};
use sp_std::{cmp::PartialOrd,prelude::Vec, collections::btree_map::BTreeMap, marker};

pub type OrganizationId = u64;

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
    /// the end datetime of the proposal,it set by created.
    pub end_dt: BlockNumber,
}

impl<AccountId, Balance, BlockNumber> ProposalDetail<AccountId, Balance, BlockNumber> {
    pub fn new(creator: AccountId,end: BlockNumber) -> Self {
        ProposalDetail{
            votes: BTreeMap::<AccountId, (Balance, bool)>::new(),
            creator: creator.clone(),
            end_dt: end,
        }
    }
    pub fn vote(&mut self,voter: AccountId,value: Balance,yesorno: bool) -> dispatch::DispatchResult {
        if let some(val) = self.votes.get_mut(voter) {
            if val.1 == yesorno {
                *val = (value.saturating_add(val.0),yesorno);
            } else {
                *val = (value.saturating_add(val.0),val.1);
            }
        } else {
            self.votes.insert(voter.clone(),(value,yesorno));
        }
        Ok(())
    }
    pub fn summary(&self) -> (Balance,Balance) {
        let (yes_balance,no_balance) = (Zero::zero(),Zero::zero());
        self.votes.iter().for_each(|val|{
            if val.1.1 {
                yes_balance = yes_balance.saturating_add(val.1.0.clone());
            } else {
                no_balance = no_balance.saturating_add(val.1.0.clone());
            }
        });
        return (yes_balance,no_balance)
    }
    pub fn is_expire(&self,current: BlockNumber) -> bool {
        return self.end_dt < current
    }
}

pub type ProposalDetailOf<T> = ProposalDetail<<T as frame_system::Trait>::AccountId,
    <<T as frame_system::Trait>::AccountId>::Balance,<T as frame_system::Trait>::BlockNumber>;

/// the assetInfo use to the organization manage it's asset, support multiAsset
/// in a organization, usually it use to vote a proposal.
#[derive(Eq, PartialEq, RuntimeDebug, Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetInfo<AssetId> {
    /// kind of asset
    pub id: AssetId,
}
impl<AssetId> AssetInfo<AssetId> {
    pub fn id(&self) -> AssetId {
        self.id.clone()
    }
}

/// This structure is used to encode metadata about an organization.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrgInfo<AccountId, Balance,AssetId> {
    /// A set of accounts of an organization.
    pub members: Vec<AccountId>,
    /// params for every organization,will set on create organization
    pub param:  OrgRuleParam<Balance>,
    /// only one asset for one organization
    pub asset: AssetInfo<AssetId>,
}

impl<AccountId: Ord, Balance,AssetId> OrgInfo<AccountId, Balance,AssetId> {
    /// Sort all the vectors inside the strutcture.
    pub fn sort(&mut self) {
        self.members.sort();
    }
    pub fn is_member(&self,mid: AccountId) -> bool {
        match self.members.iter().find(|&&x| mid.eq(&x)) {
            Some(v) => true,
            _ =>   false,
        }
    }
    pub fn get_asset_id(&self) -> AssetId {
        self.asset.id()
    }
}



/// Represent a proposal as stored by the pallet.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Proposal<Call, ProposalDetail, OrganizationId> {
    pub org: OrganizationId,
    pub call: Call,
    pub detail: ProposalDetail,
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
    pub fn get_orginfo_by_id(oid: T::AccountId) -> Result<OrgInfoOf<T>, dispatch::DispatchError> {
        match OrgInfo::<T>::try_get(oid) {
            Err(e) => Err(Error::<T>::OrganizationNotFound.into()),
            Ok(org) => Ok(org),
        }
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
            call:   call.clone().encode(),
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
    pub fn is_member(oid: T::AccountId,who: &T::AccountId) -> bool {
        match OrgInfo::<T>::try_get(oid) {
            Ok(org) => org.is_member(who),
            Err(e) => false,
        }
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
    pub fn get_proposal_by_id(pid: ProposalIdOf<T>) -> Result<ProposalOf<T>, dispatch::DispatchError> {
        match Proposals::<T>::get(pid) {
            Some(proposal) => Ok(proposal),
            None => Err(Error::<T>::ProposalNotFound.into()),
        }
    }
    fn is_pass(proposal: ProposalOf<T>) -> bool {
        return true
    }
}