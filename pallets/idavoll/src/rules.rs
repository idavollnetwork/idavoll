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



use frame_support::{ ensure,dispatch::{DispatchResult} };
#[cfg(feature = "std")]
use std::collections::{HashMap as Map, hash_map::Entry as MapEntry};
use sp_runtime::{
    RuntimeDebug,Perbill,
    traits::{Saturating, Zero,Hash},
};
#[cfg(not(feature = "std"))]
use sp_std::collections::btree_map::{BTreeMap as Map, Entry as MapEntry};

use crate::utils::*;
use crate::{ Error,Module, RawEvent, Trait,
            OrgCount,OrgInfoOf,BalanceOf};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};

pub const LengthLimit01: i32 = 32;
pub const MaxRuleNumber: u32 = 10_000;




pub trait BaseRule {
    type AccountId;
    type BlockNumber;
    type Params;
    type Data;

    /// i
    fn on_proposal_pass(height: Self::BlockNumber,content: Self::Data,detail: Self::Params) -> bool;
    fn on_proposal_expired(height: Self::BlockNumber,detail: Self::Params) -> DispatchResult;
    fn on_can_close(creator: Self::AccountId,detail: Self::Params) -> DispatchResult;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrgRuleParam<Balance> {
    /// Minimum approval votes threshold in a organization
    pub minAffirmative: Balance,
    /// Maximum negative votes threshold in a organization
    pub maxDissenting:  Balance,
    /// the abstention votes threshold in a organization,More than a certain
    /// number of abstentions on the proposal then it will gradually become invalid
    pub abstention: Balance,
}

impl<Balance> OrgRuleParam<Balance> {
    pub fn default() -> Self {
        Self{
            minAffirmative: Zero::zero(),
            maxDissenting: Zero::zero(),
            abstention: Zero::zero(),
        }
    }
    pub fn is_pass(&self,yes_amount: Balance,no_amount: Balance,nu_amount: Balance,total: Balance) -> bool {
        if self.minAffirmative == Zero::zero() {
            // must be 100% approval
            return yes_amount >= total
        } else {
            if self.maxDissenting != Zero::zero() {
                return !(no_amount > Perbill::from_percent(self.maxDissenting.clone()) * total)
            }
            if self.abstention != Zero::zero() {
                return !(nu_amount > Perbill::from_percent(self.abstention.clone()) * total)
            }
            return yes_amount > Perbill::from_percent(self.minAffirmative.clone()) * total
        }
    }
    pub fn inherit_valid(&self,subparam: OrgRuleParam<Balance>) -> DispatchResult {
        if subparam.minAffirmative >= self.minAffirmative
            && subparam.maxDissenting <= self.maxDissenting
            && subparam.abstention <= self.abstention {
            Ok(())
        }
        Err(Error::<Trait>::WrongRuleParam.into())
    }
}

/*
    Action  ==> innerAction
    Action  ==> financeAction
*/
pub trait DefaultAction {
    fn change_organization_name() -> DispatchResult;
    fn transfer() -> DispatchResult;
}

pub trait InnerAction: DefaultAction {

}
pub trait FinanceAction: DefaultAction {

}






