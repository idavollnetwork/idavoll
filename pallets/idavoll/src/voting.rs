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
    dispatch::{DispatchResult},
};
use crate::{ProposalIdOf, Error,Module, RawEvent, Trait,BalanceOf};
use idavoll_asset::{token::BaseToken,finance::BaseFinance};
use frame_support::traits::Get;


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
            return Err(Error::<T>::NotMemberInOrg.into());
        }
        let oinfo = Self::get_orginfo_by_id(oid.clone())?;
        let aid = oinfo.get_asset_id();
        let proposal = Self::get_proposal_by_id(pid)?;
        if proposal.detail.is_expire(height) {
            Self::try_close_proposal(oid.clone(),aid,pid,height)?;
            return Err(Error::<T>::ProposalExpired.into());
        }
        // lock the voter's balance
        T::AssetHandle::lock(aid,&voter,value)?;
        Self::base_vote_on_proposal(pid,voter,value,yesorno)?;
        // check the proposal can closed
        Self::try_close_proposal(oid.clone(),aid,pid,height)
    }
    /// close the proposal when the proposal was expire or passed.
    /// it will auto unlocked the voter's asset
    pub fn try_close_proposal(oid: T::AccountId,aid: T::AssetId,pid: ProposalIdOf<T>,height: T::BlockNumber) -> DispatchResult {
        let proposal = Self::get_proposal_by_id(pid)?;
        let is_expire = proposal.detail.is_expire(height);
        let is_pass = Self::is_pass(proposal.clone());
        if is_pass && !is_expire {
            Self::base_call_dispatch(pid,proposal.clone())?;
        }
        if is_expire || is_pass {
            Self::remove_proposal_by_id(pid);
            proposal.detail.votes.iter().for_each(|val|{
                match T::AssetHandle::unlock(aid,&val.0.clone(),val.1.0) {
                    _ => {},
                }
            });
            let proposal_creator = proposal.creator();
            let locked_balance = T::InherentStakeProposal::get();
            T::Finance::unlock_balance(oid,proposal_creator,locked_balance)?;
            if is_expire {
                Self::deposit_event(RawEvent::ProposalRefuse(pid));
            }
            if is_pass {
                Self::deposit_event(RawEvent::ProposalPassed(pid));
            }
        }
        Ok(())
    }
    /// create new token with new organization
    pub fn create_new_token(owner: T::AccountId,total: T::Balance) -> T::AssetId {
        T::AssetHandle::create(owner,total)
    }

    pub fn handle_transfer_by_decision(oid: T::AccountId,to: T::AccountId,value: T::Balance) -> DispatchResult {
        T::Finance::transfer_by_vault(oid,to,value)
    }
}
