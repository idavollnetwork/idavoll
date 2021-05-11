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

use frame_support::{ensure, dispatch::{self, Parameter}, StorageValue};
use crate::rules::{BaseRule,OrgRuleParam};
use crate::{OrgCounter, OrgInfos,Proposals,ProposalOf,ProposalIdOf,Error,
            Module, RawEvent, Trait, OrgCount,OrgInfoOf,OrgRuleParamOf,
            BalanceOf};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug, traits::{Hash as FrameHash,Saturating,AtLeast32BitUnsigned,Member, Zero}, DispatchResult};
use sp_std::{cmp::PartialOrd,prelude::Vec, boxed::Box,collections::btree_map::BTreeMap, marker};
use idavoll_asset::{token::BaseToken,finance::BaseFinance};
use frame_support::sp_runtime::DispatchError;

// pub type OrganizationId = u64;

/// this is the free proposal,every one in the organization can create
/// the proposal for pay a little fee, it not staking any asset to do this.
#[derive(Eq, PartialEq, RuntimeDebug, Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ProposalDetail<AccountId, Balance, BlockNumber>
    where
        AccountId: Ord + Clone,
        Balance: Member + Parameter + AtLeast32BitUnsigned + Copy + Default,
        BlockNumber: Eq + PartialOrd + Clone,
{
    /// A map of voter => (coins, in agree or against)
    pub votes: BTreeMap<AccountId, (Balance, bool)>,
    /// the creator of the proposal
    pub creator: AccountId,
    /// the end datetime(block number) of the proposal,it set by created.
    pub end_dt: BlockNumber,
    /// the user-default param for the vote rule in the proposal.
    /// it must be in range of the organization's param
    pub sub_param: OrgRuleParam<Balance>,
}

impl<AccountId: Ord + Clone,
    Balance: Member + Parameter + AtLeast32BitUnsigned + Copy + Default,
    BlockNumber: Eq + PartialOrd + Clone,
    > ProposalDetail<AccountId, Balance, BlockNumber> {
    pub fn new(creator: AccountId,end: BlockNumber,subparam: OrgRuleParam<Balance>) -> Self {
        ProposalDetail{
            votes: BTreeMap::<AccountId, (Balance, bool)>::new(),
            creator: creator.clone(),
            end_dt: end,
            sub_param: subparam.clone(),
        }
    }
    pub fn vote(&mut self,voter: AccountId,value: Balance,yesorno: bool) -> dispatch::DispatchResult {
        if let Some(val) = self.votes.get_mut(&voter.clone()) {
            if val.1 == yesorno {
                *val = (value.saturating_add(val.0.clone()),yesorno);
            } else {
                *val = (value.saturating_add(val.0.clone()),val.1);
            }
        } else {
            self.votes.insert(voter.clone(),(value,yesorno));
        }
        Ok(())
    }
    pub fn summary(&self) -> (Balance,Balance) {
        let (mut yes_balance,mut no_balance) = (Balance::default(),Balance::default());
        self.votes.iter().for_each(|val|{
            if val.1.1 {
                yes_balance = yes_balance.saturating_add(val.1.0.clone());
            } else {
                no_balance = no_balance.saturating_add(val.1.0.clone());
            }
        });
        return (yes_balance.clone(),no_balance.clone())
    }
    pub fn is_expire(&self,current: BlockNumber) -> bool {
        return  current > self.end_dt
    }
    pub fn pass(&self,total_balance: Balance) -> bool {
        let (yes_balance,no_balance) = self.summary();
        let nu_balance = Zero::zero();
        return self.sub_param.is_pass(yes_balance,no_balance,nu_balance,total_balance)
    }
    pub fn creator(&self) -> AccountId {
        self.creator.clone()
    }
}

pub type ProposalDetailOf<T> = ProposalDetail<<T as frame_system::Trait>::AccountId,
    BalanceOf<T>,<T as frame_system::Trait>::BlockNumber>;

/// the assetInfo use to the organization manage it's asset, support multiAsset
/// in a organization, usually it use to vote a proposal.
#[derive(Eq, PartialEq, RuntimeDebug, Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetInfo<AssetId: Clone + Default> {
    /// kind of asset
    pub id: AssetId,
}
impl<AssetId: Clone + Default> AssetInfo<AssetId> {
    pub fn id(&self) -> AssetId {
        self.id.clone()
    }
    pub fn set_id(&mut self,id: AssetId) {
        self.id = id
    }
}

/// This structure is used to encode metadata about an organization.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrgInfo<AccountId, Balance,AssetId>
where
    AccountId: Ord + Clone,
    Balance: Parameter + Member + PartialOrd + AtLeast32BitUnsigned,
    AssetId: Clone + Default,
{
    /// A set of accounts of an organization.
    pub members: Vec<AccountId>,
    /// params for every organization,will set on create organization
    pub param:  OrgRuleParam<Balance>,
    /// only one asset for one organization
    pub asset: AssetInfo<AssetId>,
}

impl<
    AccountId: Ord + Clone,
    Balance: Parameter + Member + PartialOrd + AtLeast32BitUnsigned,
    AssetId: Clone + Default,
> OrgInfo<AccountId, Balance,AssetId> {
    pub fn new() -> Self {
        Self{
            members: Vec::new(),
            param: OrgRuleParam::default(),
            asset: AssetInfo::default(),
        }
    }
    /// Sort all the vectors inside the strutcture.
    pub fn sort(&mut self) {
        self.members.sort();
    }
    pub fn is_member(&self,mid: AccountId) -> bool {
        match self.members.iter().find(|&x| mid.eq(&x)) {
            Some(v) => true,
            _ =>   false,
        }
    }
    pub fn get_asset_id(&self) -> AssetId {
        self.asset.id()
    }
    pub fn set_asset_id(&mut self,id: AssetId) {
        self.asset.set_id(id)
    }
    pub fn add_member(&mut self, member: AccountId) -> DispatchResult {
        if !self.is_member(member.clone()) {
            self.members.push(member.clone());
        }
        Ok(())
    }
    pub fn counts(&self) -> u32 {
        self.members.len() as u32
    }
}



/// Represent a proposal as stored by the pallet.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Proposal<Call, AccountId, Balance, BlockNumber>
where
    AccountId: Ord + Clone,
    Balance: Member + Parameter + AtLeast32BitUnsigned + Copy + Default,
    BlockNumber: Eq + PartialOrd + Clone,
{
    pub org: AccountId,
    pub call: Call,
    pub detail: ProposalDetail<AccountId, Balance, BlockNumber>,
}

impl<
    Call,
    AccountId: Ord + Clone,
    Balance: Member + Parameter + AtLeast32BitUnsigned + Copy + Default,
    BlockNumber: Eq + PartialOrd + Clone,
> Proposal<Call, AccountId, Balance, BlockNumber> {
    pub fn new(id: AccountId,calldata: Call,detail: ProposalDetail<AccountId, Balance, BlockNumber>) -> Self {
        Self{
            org: id,
            call: calldata,
            detail: detail,
        }
    }
    pub fn creator(&self) -> AccountId {
        self.detail.creator()
    }
}




impl<T: Trait> Module<T>  {

    /// get proposal id by hash the content in the proposal
    pub fn make_proposal_id(proposal: &ProposalOf<T>) -> ProposalIdOf<T> {
        T::Hashing::hash_of(&[proposal.encode()])
    }
    pub fn get_count_members(oid: T::AccountId) -> u32 {
        match Self::get_orginfo_by_id(oid.clone()) {
            Ok(org) => {
                org.counts()
            },
            Err(e) => 0,
        }
    }

    pub fn get_token_id_by_oid(oid: T::AccountId) -> Result<T::AssetId,DispatchResult> {
        let org = Self::get_orginfo_by_id(oid)?;
        Ok(org.get_asset_id())
    }
    pub fn get_total_token_by_oid(oid: T::AccountId) -> Result<T::Balance,DispatchResult> {
        let org = Self::get_orginfo_by_id(oid)?;

        Ok(T::AssetHandle::total(org.get_asset_id()))
    }
    pub fn get_local_balance(id: T::AccountId) -> Result<T::Balance,DispatchError> {
        T::Finance::balance_of(id.clone())
    }

    pub fn is_pass(proposal: ProposalOf<T>) -> bool {
        let total_balance = Self::get_total_token_by_oid(proposal.org);
        match total_balance {
            Ok(balance) => proposal.detail.pass(balance),
            Err(e) => false,
        }
    }

    pub fn reserve_to_vault(oid: T::AccountId,who: T::AccountId,value: T::Balance) -> DispatchResult {
        T::Finance::reserve_to_org(oid.clone(),who.clone(),value)
    }
    pub fn on_reserve_to_vault(id: u32,who: T::AccountId,value: T::Balance) -> DispatchResult {
        let oid = Self::counter2Orgid(id);
        // make sure the oid was exist
        Self::get_orginfo_by_id(oid.clone())?;
        Self::reserve_to_vault(oid,who.clone(),value)
    }

    pub fn on_create_proposal(id:u32,who: T::AccountId,expire: T::BlockNumber,sub_param: OrgRuleParamOf<T>
                              ,call: Box<<T as Trait>::Call>) ->DispatchResult {
        let oid = Self::counter2Orgid(id);
        let org = Self::get_orginfo_by_id(oid.clone())?;
        if !org.param.inherit_valid(sub_param.clone()) {
            return Err(Error::<T>::WrongRuleParam.into());
        }

        let proposal = Proposal {
            org:    oid.clone(),
            call: call.encode(),
            detail: ProposalDetail::new(who.clone(),expire,sub_param.clone()),
        };
        Self::base_create_proposal(oid.clone(),proposal)
    }
    pub fn on_vote_proposal(pid: ProposalIdOf<T>,who: T::AccountId,value: T::Balance,yesorno: bool,cur: T::BlockNumber) -> DispatchResult {
        let proposal = Self::get_proposal_by_id(pid)?;
        Self::vote_on_proposal(proposal.org,pid,who.clone(),value,yesorno,cur)
    }
    pub fn on_add_member(owner: T::AccountId,who: T::AccountId,id: u32) -> dispatch::DispatchResult {
        let oid = Self::counter2Orgid(id);
        let org = Self::get_orginfo_by_id(oid.clone())?;
        ensure!(!Self::is_member(oid.clone(),&who),Error::<T>::MemberDuplicate);
        Self::base_add_member_on_orgid(oid.clone(),who.clone())
    }

}