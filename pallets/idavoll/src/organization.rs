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


pub const LengthLimit: i32 = 32;
pub const InitOrgID: u64 = 1000;
pub const MaxMembers: i32 = 1000;

pub trait DefaultAction {

    fn change_organization_name() -> Error;
    fn transfer() -> Error;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// The origanization id assigned by system
    pub id: u64
    /// The total balance of the origanization.
	pub total: u64,
	/// The user friendly name of this origanization. Limited in length by `LengthLimit`.
	pub name: Vec<u8>,
    /// The description of the origanization. Limited in length by `3 * LengthLimit`.
    pub description: Vec<u8>,
}

impl Organization {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> OrganizationBuilder {
        OrganizationBuilder {
            body: Default::default(),
        }
    }

    #[inline]
    pub fn admin_get_all_orgs() -> OrganizationGetBuilder {
        OrganizationGetBuilder {
            param_page: None,
            param_limit: None,
        }
    }

    #[inline]
    pub fn org_get_all() -> OrganizationGetBuilder1 {
        OrganizationGetBuilder1 {
            param_page: None,
            param_limit: None,
        }
    }

    #[inline]
    pub fn org_get() -> OrganizationGetBuilder2<crate::generics::MissingOrg> {
        OrganizationGetBuilder2 {
            inner: Default::default(),
            _param_org: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn org_list_current_user_orgs() -> OrganizationGetBuilder3 {
        OrganizationGetBuilder3 {
            param_page: None,
            param_limit: None,
        }
    }

    #[inline]
    pub fn org_list_user_orgs() -> OrganizationGetBuilder4<crate::generics::MissingUsername> {
        OrganizationGetBuilder4 {
            inner: Default::default(),
            _param_username: core::marker::PhantomData,
        }
    }
}
