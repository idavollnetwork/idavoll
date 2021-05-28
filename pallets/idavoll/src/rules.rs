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
            min_affirmative: 0_u32,
            max_dissenting: 0_u32,
            abstention: 0_u32,
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
            (self.abstention == 0 || !(nu_amount > Perbill::from_percent(self.abstention) * total))

    }
    pub fn inherit_valid(&self,subparam: OrgRuleParam<Balance>) -> bool {
        subparam.min_affirmative >= self.min_affirmative
            && subparam.max_dissenting <= self.max_dissenting
            && subparam.abstention <= self.abstention
    }
}






