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


use frame_support::ensure;
#[cfg(feature = "std")]
use std::collections::{HashMap as Map, hash_map::Entry as MapEntry};
#[cfg(not(feature = "std"))]
use sp_std::collections::btree_map::{BTreeMap as Map, Entry as MapEntry};

use crate::utils::*;
pub const LengthLimit01: i32 = 32;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};


// #[derive(Debug, Default, Clone,Eq)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// pub struct IdvToken<AccountId> {
//     /// The balance of the idv token
//     pub balance: Map<AccountId, u64>,
//     pub allowance: Map<AccountId,Map<AccountId,u64>>,
//     /// The total balance of the token.
// 	pub total: u64,
// 	/// The user friendly name of this token. Limited in length by `LengthLimit`.
// 	pub name: Vec<u8>,
//     pub owner:   AccountId, 
// }

// impl<AccountId> IdvToken<AccountId> {

//     pub fn totalSupply(&self) -> u64 {
//         self->total
//     }

//     pub fn balanceOf(&self, owner: AccountId) -> Result<u64,Error> {
//         Ok(())
//     }

//     pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Result<u64,Error> {
//         Ok(())
//     }

//     pub fn transfer(&self, to: AccountId, value: u64) -> Result<(),Error> {
//         Ok(())
//     }

//     function approve(spender: AccountId, value: u64) -> Result<(),Error> {
//         Ok(())
//     }

//     function transferFrom(from: AccountId, to: AccountId, value: u64) -> Result<(),Error> {
//         Ok(())
//     }
// }


