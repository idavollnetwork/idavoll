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



use frame_support::{dispatch::{DispatchResult,Parameter} };
// #[cfg(feature = "std")]
// use std::collections::{HashMap as Map, hash_map::Entry as MapEntry};
use sp_runtime::{
    RuntimeDebug,Perbill,
    traits::{Member,AtLeast32BitUnsigned},
};
use sp_std::{cmp::PartialOrd, marker};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};



pub trait BaseRule {
    type AccountId;
    type BlockNumber;
    type Params;
    type Data;

    fn on_proposal_pass(height: Self::BlockNumber,content: Self::Data,detail: Self::Params) -> bool;
    fn on_proposal_expired(height: Self::BlockNumber,detail: Self::Params) -> DispatchResult;
    fn on_can_close(creator: Self::AccountId,detail: Self::Params) -> DispatchResult;
}

/// OrgRuleParam was used to vote by decision, it passed by all 'TRUE',
/// passed by more than 60% 'Yes' votes and less than 5% 'no' votes.
/// 'pass' = 'yes > min_affirmative%' and 'no <= max_dissenting' and 'nul <= abstention'
///
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrgRuleParam<Balance>
where
    Balance: Parameter + Member + PartialOrd + AtLeast32BitUnsigned,
{
    /// Minimum approval votes threshold in a organization
    pub min_affirmative: u32,
    /// Maximum negative votes threshold in a organization
    pub max_dissenting:  u32,
    /// the abstention votes threshold in a organization,More than a certain
    /// number of abstentions on the proposal then it will gradually become invalid
    pub abstention: u32,
    _phantom: marker::PhantomData<Balance>,
}

impl<Balance: Parameter + Member + PartialOrd + AtLeast32BitUnsigned> OrgRuleParam<Balance> {
    pub fn default() -> Self {
        Self{
            min_affirmative: 0 as u32,
            max_dissenting: 0 as u32,
            abstention: 0 as u32,
            _phantom: marker::PhantomData,
        }
    }
    pub fn new(a: u32,d: u32,s: u32) -> Self {
        Self{
            min_affirmative: a,
            max_dissenting: d,
            abstention: s,
            _phantom: marker::PhantomData,
        }
    }
    pub fn is_pass(&self,yes_amount: Balance,no_amount: Balance,nu_amount: Balance,total: Balance) -> bool {

        (self.min_affirmative == 0 || yes_amount > Perbill::from_percent(self.min_affirmative) * total.clone()) &&
            (self.max_dissenting == 0 || !(no_amount > Perbill::from_percent(self.max_dissenting) * total.clone())) &&
            (self.abstention == 0 || !(nu_amount > Perbill::from_percent(self.abstention) * total.clone()))

    }
    pub fn inherit_valid(&self,subparam: OrgRuleParam<Balance>) -> bool {
        return subparam.min_affirmative >= self.min_affirmative
            && subparam.max_dissenting <= self.max_dissenting
            && subparam.abstention <= self.abstention ;
    }
}






