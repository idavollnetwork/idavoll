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

/// Organization represents an organization
///
///

use frame_support::{ensure, dispatch::{self, Parameter}};
use crate::rules::{OrgRuleParam};
use crate::{
    ProposalOf,ProposalIdOf,Error,
    Module, Trait, OrgRuleParamOf,
    BalanceOf};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug, traits::{Hash as FrameHash,AtLeast32BitUnsigned,Member, Zero}, DispatchResult};
use sp_std::{cmp::PartialOrd,prelude::Vec, boxed::Box,collections::btree_map::BTreeMap};
use idavoll_asset::{token::BaseToken,finance::BaseFinance};
use frame_support::sp_runtime::DispatchError;
use frame_support::traits::Get;

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
    pub fn new(who: AccountId,end: BlockNumber,subparam: OrgRuleParam<Balance>) -> Self {
        ProposalDetail{
            votes: BTreeMap::<AccountId, (Balance, bool)>::new(),
            creator: who,
            end_dt: end,
            sub_param: subparam,
        }
    }
    pub fn vote(&mut self,voter: AccountId,value: Balance,yesorno: bool) {
        if let Some(val) = self.votes.get_mut(&voter) {
            if val.1 == yesorno {
                *val = (value.saturating_add(val.0),yesorno);
            } else {
                *val = (value.saturating_add(val.0),val.1);
            }
        } else {
            self.votes.insert(voter,(value,yesorno));
        }
    }
    pub fn summary(&self) -> (Balance,Balance) {
        let (mut yes_balance,mut no_balance) = (Balance::default(),Balance::default());
        self.votes.iter().for_each(|val|{
            if val.1.1 {
                yes_balance = yes_balance.saturating_add(val.1.0);
            } else {
                no_balance = no_balance.saturating_add(val.1.0);
            }
        });
        (yes_balance,no_balance)
    }
    pub fn is_expire(&self,current: BlockNumber) -> bool {
        current > self.end_dt
    }
    pub fn pass(&self,total_balance: Balance) -> bool {
        let (yes_balance,no_balance) = self.summary();
        let nu_balance = Zero::zero();
        self.sub_param.is_pass(yes_balance,no_balance,nu_balance,total_balance)
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
        self.members.iter().find(|&x| mid.eq(&x)).is_some()
    }
    pub fn get_asset_id(&self) -> AssetId {
        self.asset.id()
    }
    pub fn set_asset_id(&mut self,id: AssetId) {
        self.asset.set_id(id)
    }
    pub fn add_member(&mut self, member: AccountId) -> DispatchResult {
        if !self.is_member(member.clone()) {
            self.members.push(member);
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
    pub fn new(id: AccountId,calldata: Call,info: ProposalDetail<AccountId, Balance, BlockNumber>) -> Self {
        Self{
            org: id,
            call: calldata,
            detail: info,
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
        match Self::get_orginfo_by_id(oid) {
            Ok(org) => {
                org.counts()
            },
            Err(_) => 0,
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
    pub fn get_free_balance_on_token_by_user(oid: T::AccountId,who: T::AccountId) -> Result<T::Balance,DispatchResult> {
        let org = Self::get_orginfo_by_id(oid)?;
        Ok(T::AssetHandle::free_balance_of(org.get_asset_id(),&who))
    }
    pub fn get_local_balance(id: T::AccountId) -> Result<T::Balance,DispatchError> {
        T::Finance::balance_of(id)
    }

    pub fn is_pass(proposal: ProposalOf<T>) -> bool {
        let total_balance = Self::get_total_token_by_oid(proposal.org);
        match total_balance {
            Ok(balance) => proposal.detail.pass(balance),
            Err(_) => false,
        }
    }

    pub fn reserve_to_vault(oid: T::AccountId,who: T::AccountId,value: T::Balance) -> DispatchResult {
        T::Finance::reserve_to_org(oid,who,value)
    }
    pub fn on_reserve_to_vault(id: u32,who: T::AccountId,value: T::Balance) -> DispatchResult {
        let oid = Self::counter_2_orgid(id);
        // make sure the oid was exist
        Self::get_orginfo_by_id(oid.clone())?;
        Self::reserve_to_vault(oid,who,value)
    }

    pub fn on_create_proposal(id:u32,who: T::AccountId,expire: T::BlockNumber,sub_param: OrgRuleParamOf<T>
                              ,call: Box<<T as Trait>::Call>) ->DispatchResult {
        let oid = Self::counter_2_orgid(id);
        let org = Self::get_orginfo_by_id(oid.clone())?;
        if !org.param.inherit_valid(sub_param.clone()) {
            return Err(Error::<T>::WrongRuleParam.into());
        }

        if !Self::is_member(oid.clone(),&who) {
            return Err(Error::<T>::NotMemberInOrg.into());
        }
        let locked_balance = T::InherentStakeProposal::get();
        T::Finance::lock_balance(oid.clone(),who.clone(),locked_balance)?;

        let proposal = Proposal {
            org:    oid.clone(),
            call: call.encode(),
            detail: ProposalDetail::new(who,expire,sub_param),
        };
        Self::base_create_proposal(oid,proposal)
    }
    pub fn on_vote_proposal(pid: ProposalIdOf<T>,who: T::AccountId,value: T::Balance,yesorno: bool,cur: T::BlockNumber) -> DispatchResult {
        let proposal = Self::get_proposal_by_id(pid)?;
        Self::vote_on_proposal(proposal.org,pid,who,value,yesorno,cur)
    }
    pub fn on_add_member_and_assigned_token(owner: T::AccountId,who: T::AccountId,id: u32,value: T::Balance) -> dispatch::DispatchResult {
        let oid = Self::counter_2_orgid(id);
        match Self::get_token_id_by_oid(oid.clone()) {
            Ok(asset_id) => {
                let free = T::AssetHandle::free_balance_of(asset_id,&owner);

                ensure!(free >= value && value >= Zero::zero(),Error::<T>::TokenBalanceLow);
                ensure!(Self::is_member(oid.clone(),&owner),Error::<T>::NotMemberInOrg);
                ensure!(!Self::is_member(oid.clone(),&who),Error::<T>::MemberDuplicate);
                Self::base_add_member_on_orgid(oid.clone(),who.clone())?;
                if value > Zero::zero() {
                    T::AssetHandle::transfer(asset_id,&owner,&who,value)
                } else {
                    Ok(())
                }
            },
            Err(e) => e,
        }
    }

}
