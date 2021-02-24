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
	ensure,
};

pub const LengthLimit: i32 = 32;
pub const InitOrgID: u64 = 1000;
pub const MaxMembers: i32 = 1000;

pub trait DefaultAction {

    fn change_organization_name() -> Error;
    fn transfer() -> Error;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Organization<AccountId> {
    /// The origanization id assigned by system
    pub id: u64
    /// The total balance of the origanization.
	pub total: u64,
	/// The user friendly name of this origanization. Limited in length by `LengthLimit`.
	pub name: Vec<u8>,
    /// The description of the origanization. Limited in length by `3 * LengthLimit`.
    pub description: Vec<u8>,
    pub creator:   AccountId, 
}

impl<AccountId> Default for Organization<AccountId> {

	fn default() -> Organization {
        InitOrgID = InitOrgID + 1；
		Organization {
			id: InitOrgID,
			total: 0,
			name: "default".Into(),
			description: "this is a org".Into(),
            creator: AccountId::default(),
		}
	}
}

impl<AccountId> Organization<AccountId> {

    #[inline]
    pub fn build( total: u64, name: Vec<u8>, description: Vec<u8>,aid: AccountId) -> Result<Self,Error> {

        InitOrgID = InitOrgID + 1；
        ensure!(name.len() <= LengthLimit as usize, Error::BadMetadata);
        ensure!(description.len() <= (LengthLimit * 3) as usize, Error::BadMetadata);
        Ok(Organization {
            id: InitOrgID,
            total: total,
            name: name.clone(),
            description: description.clone(),
            creator: aid.clone(),
        })
    }
}
