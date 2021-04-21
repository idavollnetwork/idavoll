// This file is part of Idavoll Node.

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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit="128"]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error,
					dispatch, traits::{Get,EnsureOrigin},
					Parameter,ensure,weights::{GetDispatchInfo, Weight},
};
use frame_system::ensure_signed;
use sp_runtime::{Permill, ModuleId, RuntimeDebug,
				 traits::{Zero, StaticLookup, AccountIdConversion,
						  Saturating,AtLeast32BitUnsigned,AtLeast32Bit,
						  Member,MaybeSerializeDeserialize,
				 }};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod organization;
mod rules;
mod voting;
mod utils;
mod default_weights;

pub use organization::{OrgInfo, Proposal,ProposalDetailOf};
use idavoll_asset::{token::BaseToken,finance::BaseFinance,LocalBalance,Trait as AssetTrait};
use rules::{OrgRuleParam};

pub trait WeightInfo {
	fn create_origanization(b: u32) -> Weight;
	fn create_proposal() -> Weight;
	fn veto_proposal(b: u32, c: u32) -> Weight;
	fn finish_proposal(b: u32, c: u32) -> Weight;
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	/// The idavoll's module id, used for deriving its sovereign account ID,use to organization id.
	type ModuleId: Get<ModuleId>;
	/// the Asset Handler will handle all op in the voting about asset operation.
	type AssetHandle: BaseToken<Self::AccountId>;

	type Balance: Member + Parameter + AtLeast32BitUnsigned + MaybeSerializeDeserialize + Default + Copy;
	/// keep the local asset(idv) of the organization
	type Finance: BaseFinance<Self::AccountId,Self::Balance>;
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
}

type BalanceOf<T> = <T as Trait>::Balance;
pub type OrgCount = u32;
pub type OrgInfoOf<T> = OrgInfo<
	<T as frame_system::Trait>::AccountId,
	BalanceOf<T>,
	<T as Trait>::AssetId,
>;
pub type ProposalIdOf<T> = <T as frame_system::Trait>::Hash;
pub type ProposalOf<T> = Proposal<
	Vec<u8>,
	<T as frame_system::Trait>::AccountId,
	BalanceOf<T>,
	<T as frame_system::Trait>::BlockNumber,
	ProposalIdOf<T>,
>;
pub type OrgRuleParamOf<T> = OrgRuleParam<BalanceOf<T>>;

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait> as IdavollModule {
		pub Counter get(fn counter): OrgCount = 0;
		pub OrgInfos get(fn OrgInfos): map hasher(blake2_128_concat) T::AccountId => Option<OrgInfoOf<T>>;
        pub Proposals get(fn proposals): map hasher(blake2_128_concat) ProposalIdOf<T> => Option<ProposalOf<T>>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T>
	where
	AccountId = <T as frame_system::Trait>::AccountId,
	ProposalId = ProposalIdOf<T>,
	OrgInfo = OrgInfoOf<T>,
	{
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
		/// An organization was created with the following parameters. \[organizationId, details\]
        OrganizationCreated(AccountId, OrgInfo),
		/// A proposal has been finalized with the following result. \[proposal id, result\]
        ProposalFinalized(ProposalId, dispatch::DispatchResult),
        /// A proposal has been passed. \[proposal id]
        ProposalPassed(ProposalId),
        /// create a proposal.		\[organization id,proposal id,creator]
        ProposalCreated(AccountId,ProposalId,AccountId),
        /// Proposal Refused \[proposal id]
        ProposalRefuse(AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// need the maximum number for the storage value for the fixed type.
		StorageOverflow,
		OrganizationNotFound,
		NotOwnerByOrg,
		/// not found the proposal by id in the runtime storage
		ProposalNotFound,
		ProposalDecodeFailed,
		ProposalDuplicate,
		ProposalExpired,
		NotMember,
		WrongRuleParam,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;
		const ModuleId: ModuleId = T::ModuleId::get();
		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// create organization with the assetID=0,this will create new token for voting proposal
		/// and the token will assgined to the creator
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn create_origanization(origin,total: T::Balance,info: OrgInfoOf<T>) -> dispatch::DispatchResult {
			let owner = ensure_signed(origin)?;
			let asset_id = Self::create_new_token(owner.clone(),total);
			let info_clone = info.clone();
			info_clone.add_member(owner.clone());
			info_clone.set_asset_id(asset_id.clone());
			Self::create_org(info_clone.clone())
		}
		/// reserve the local asset(idv) to organization's Vault, it used to assigned by the proposal
		/// of call function
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn deposit_to_origanization(origin,id: u32,value: T::Balance) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::reserve_to_Vault(id,who,value)
		}

		/// voting on the proposal by the members in the organization
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn vote_proposal(origin,pid: ProposalIdOf<T>,value: T::Balance,yesorno: bool) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::on_vote_proposal(pid,who,value,yesorno,frame_system::Module::<T>::block_number())
		}
		/// voting on the proposal by the members in the organization
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn add_member_by_onwer(origin,target: <T::Lookup as StaticLookup>::Source,id: u32) -> dispatch::DispatchResult {
			let owner = ensure_signed(origin)?;
			let who = T::Lookup::lookup(target)?;

			Self::on_add_member(owner,who,id)
		}
		/// create proposal in the organization for voting by members
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn create_proposal(origin,id: u32,length: T::BlockNumber,sub_param: OrgRuleParamOf<T>,
		call: Box<<T as frame_system::Trait>::Call>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let cur = frame_system::Module::<T>::block_number();
			let expire = cur.saturating_add(length);
			Self::on_create_proposal(id,who,expire,call)
		}
	}
}
