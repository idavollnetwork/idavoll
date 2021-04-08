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





impl<T: Trait> Module<T> {
    /// 
    pub fn vote_on_proposal(oid: T::AccountId,pid: ProposalIdOf<T>) -> DispatchResult {
        Ok(())
    }

    pub fn close() -> DispatchResult {
        Ok(())
    }
}