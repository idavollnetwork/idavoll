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

use frame_support::{
	codec::{Decode, Encode},
	decl_module, decl_storage, decl_event, decl_error,
	dispatch::{
		self,Dispatchable, Parameter, PostDispatchInfo,
	},
	traits::{Get,EnsureOrigin},
	ensure,weights::{GetDispatchInfo, Weight},
};
use frame_system::ensure_signed;
use sp_runtime::{Permill, ModuleId, RuntimeDebug,
				 traits::{Zero, StaticLookup, AccountIdConversion,
						  Saturating,AtLeast32BitUnsigned,AtLeast32Bit,
						  Member,MaybeSerializeDeserialize,
				 }};
use sp_std::{boxed::Box, prelude::Vec};

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

	/// The outer call dispatch type.
	type Call: Parameter
	+ Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo>
	+ From<frame_system::Call<Self>>
	+ GetDispatchInfo;

	/// The idavoll's module id, used for deriving its sovereign account ID,use to organization id.
	type ModuleId: Get<ModuleId>;
	/// the Asset Handler will handle all op in the voting about asset operation.
	type AssetHandle: BaseToken<
		Self::AccountId,
		AssetId = Self::AssetId,
		Balance = Self::Balance,
	>;

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
>;
pub type OrgRuleParamOf<T> = OrgRuleParam<BalanceOf<T>>;

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait> as IdavollModule {
		pub OrgCounter get(fn counter): OrgCount = 0;
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
		MemberDuplicate,
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
			let mut info_clone = info.clone();
			info_clone.add_member(owner.clone());
			info_clone.set_asset_id(asset_id.clone());
			Self::storage_new_organization(info_clone.clone())
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
		call: Box<<T as Trait>::Call>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let cur = frame_system::Module::<T>::block_number();
			let expire = cur.saturating_add(length);
			Self::on_create_proposal(id,who,expire,sub_param,call)
		}
	}
}

impl<T: Trait> Module<T>  {
	// be accountid of organization id for orginfos in the storage
	pub fn counter2Orgid(c: OrgCount) -> T::AccountId {
		T::ModuleId::get().into_sub_account(c)
	}
	pub fn get_orginfo_by_id(oid: T::AccountId) -> Result<OrgInfoOf<T>, dispatch::DispatchError> {
		if OrgInfos::<T>::contains_key(oid.clone()) {
			match <OrgInfos<T>>::get(oid.clone()) {
				Some(val) => Ok(val),
				None => Err(Error::<T>::OrganizationNotFound.into()),
			}
		}else {
			Err(Error::<T>::OrganizationNotFound.into())
		}
	}
	/// write the organization info to the storage on chain by create organization
	pub fn storage_new_organization(oinfo: OrgInfoOf<T>) -> dispatch::DispatchResult {
		let counter = OrgCounter::get();
		let new_counter = counter.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		let oid = Self::counter2Orgid(counter);

		OrgInfos::<T>::insert(&oid, oinfo.clone());
		Self::deposit_event(RawEvent::OrganizationCreated(oid, oinfo));
		OrgCounter::put(new_counter);
		Ok(())
	}

	pub fn base_create_proposal(oid: T::AccountId,proposal: ProposalOf<T>) -> dispatch::DispatchResult {

		let proposal_id = Self::make_proposal_id(&proposal);
		if Proposals::<T>::contains_key(proposal_id) {
			return Err(Error::<T>::ProposalDuplicate.into());
		}
		Proposals::<T>::insert(&proposal_id, proposal.clone());
		Self::deposit_event(RawEvent::ProposalCreated(oid, proposal_id,proposal.creator()));
		Ok(())
	}

	pub fn is_member(oid: T::AccountId,who: &T::AccountId) -> bool {
		match <OrgInfos<T>>::get(oid.clone()) {
			Some(val) => val.is_member(who.clone()),
			None => false,
		}
	}
	// add a member into a organization by orgid
	pub fn base_add_member_on_orgid(oid: T::AccountId,memberID: T::AccountId) -> dispatch::DispatchResult {
		OrgInfos::<T>::try_mutate(oid,|infos| -> dispatch::DispatchResult {
			match infos {
				Some(org) => {match org.members
					.iter()
					.find(|&x| *x==memberID) {
					None => {
						org.members.push(memberID);
						Ok(())
					},
					_ => Ok(())
				}},
				None => Ok(()),
			}
		})
	}
	pub fn get_proposal_by_id(pid: ProposalIdOf<T>) -> Result<ProposalOf<T>, dispatch::DispatchError> {
		match Proposals::<T>::get(pid) {
			Some(proposal) => Ok(proposal),
			None => Err(Error::<T>::ProposalNotFound.into()),
		}
	}
	pub fn remove_proposal_by_id(pid: ProposalIdOf<T>) {
		Proposals::<T>::remove(pid);
	}
	/// add vote infos in the proposal item on the storage
	pub fn base_vote_on_proposal(pid: ProposalIdOf<T>, voter: T::AccountId,
								 value: BalanceOf<T>, yesorno: bool) -> dispatch::DispatchResult {
		Proposals::<T>::try_mutate(pid,|proposal| -> dispatch::DispatchResult {
			if let Some(p) = proposal {
				p.detail.vote(voter.clone(),value,yesorno)?;
				// *proposal = Some(p);
			};
			Ok(())
		})?;
		Ok(())
	}
	pub fn base_call_dispatch(pid: ProposalIdOf<T>,proposal: ProposalOf<T>) -> dispatch::DispatchResult {
		// remove the proposal from the storage by the proposal passed
		let call = <T as Trait>::Call::decode(&mut &proposal.clone().call[..]).map_err(|_| Error::<T>::ProposalDecodeFailed)?;
		let res = call.dispatch(frame_system::RawOrigin::Signed(proposal.clone().org).into());
		Self::deposit_event(RawEvent::ProposalFinalized(pid, res.map(|_| ()).map_err(|e| e.error)));
		Ok(())
	}
}


#[cfg(test)]
mod test {
	use super::*;

	use frame_support::{impl_outer_origin,impl_outer_dispatch, assert_ok, assert_noop, parameter_types, weights::Weight};
	use sp_core::H256;
	use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup}, testing::Header,ModuleId};
	use pallet_balances;
	use organization::{OrgInfo, Proposal,ProposalDetailOf};
	use rules::{OrgRuleParam};


	impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}
	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			frame_system::System,
        }
    }

	type System = frame_system::Module<Test>;
	type IdvBalances = pallet_balances::Module<Test>;
	type IdavollAsset = idavoll_asset::Module<Test>;

	const A: u128 = 100;
	const B: u128 = 200;
	const OWNER: u128 = 88;
	const ORGID: u128 = 1000;
	const ORGID2: u128 = 2000;

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
		pub const IdvAssetModuleId: ModuleId = ModuleId(*b"py/asset");
		pub const IdavollModuleId: ModuleId = ModuleId(*b"py/idvol");
	}
	impl frame_system::Trait for Test {
		type BaseCallFilter = ();
		type Origin = Origin;
		type Index = u64;
		type Call = Call;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u128;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = ();
		type BlockExecutionWeight = ();
		type ExtrinsicBaseWeight = ();
		type MaximumExtrinsicWeight = MaximumBlockWeight;
		type AvailableBlockRatio = AvailableBlockRatio;
		type MaximumBlockLength = MaximumBlockLength;
		type Version = ();
		type PalletInfo = ();
		type AccountData = pallet_balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
	}

	parameter_types! {
	pub const ExistentialDeposit: u64 = 10;
    }
	impl pallet_balances::Trait for Test {
		type Balance = u64;
		type DustRemoval = ();
		type Event = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type MaxLocks = ();
		type WeightInfo = ();
	}

	impl idavoll_asset::Trait for Test {
		type Event = ();
		type Balance = u64;
		type AssetId = u32;
		type Currency = IdvBalances;
		type ModuleId = IdvAssetModuleId;
	}

	type IdavollModule = Module<Test>;
	impl Trait for Test {
		type Event = ();
		type Call = Call;
		type Balance = u64;
		type AssetId = u32;
		type ModuleId = IdavollModuleId;
		type AssetHandle = IdavollAsset;
		type Finance = IdavollAsset;
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let genesis = pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(A, 100),
				(B, 200),
			],
		};
		genesis.assimilate_storage(&mut t).unwrap();
		t.into()
	}
	fn create_org() -> OrgInfoOf<Test> {
		let mut org = OrgInfo::new();
		org.members = vec![OWNER,1,2,3];
		org.param = OrgRuleParam::new(60,5,0);
		org.clone()
	}

	#[test]
	fn base_orgid_function_should_work() {
		new_test_ext().execute_with(|| {
			assert_ne!(IdavollModule::counter2Orgid(100),IdavollModule::counter2Orgid(101));
			println!("{}",IdavollModule::counter2Orgid(0));
			println!("{}",IdavollModule::counter2Orgid(1));
			println!("{}",IdavollModule::counter2Orgid(2));
			println!("{}",IdavollModule::counter2Orgid(4));
			println!("{}",IdavollModule::counter2Orgid(5));
			println!("{}",IdavollModule::counter2Orgid(6));
			println!("{}",IdavollModule::counter2Orgid(7));
			println!("{}",IdavollModule::counter2Orgid(8));
			println!("{}",IdavollModule::counter2Orgid(9));
		});
	}

	#[test]
	fn base_organization_01_should_work() {
		new_test_ext().execute_with(|| {
			let mut org = create_org();
			let asset_id = IdavollModule::create_new_token(OWNER.clone(),100);
			assert_eq!(asset_id,0);
			org.set_asset_id(asset_id.clone());
			assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			assert_eq!(IdavollModule::get_orginfo_by_id(IdavollModule::counter2Orgid(0)),Ok(org.clone()));

			assert_eq!(IdavollModule::is_member(IdavollModule::counter2Orgid(0),&OWNER),true);
			assert_eq!(IdavollModule::is_member(IdavollModule::counter2Orgid(0),&1),true);
			assert_eq!(IdavollModule::is_member(IdavollModule::counter2Orgid(0),&9),false);
		});
	}

	#[test]
	fn base_organization_02_should_work() {

		new_test_ext().execute_with(|| {
			let mut org = create_org();
			let asset_id = IdavollModule::create_new_token(OWNER.clone(),100);
			assert_eq!(asset_id,0);
			org.set_asset_id(asset_id.clone());
			let org_id = IdavollModule::counter2Orgid(0);
			assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			assert_eq!(IdavollModule::get_orginfo_by_id(org_id),Ok(org.clone()));
			assert_eq!(IdavollModule::get_count_members(org_id),4);
			// add member for the organization

			assert_noop!(IdavollModule::on_add_member(1,2,0),Error::<Test>::MemberDuplicate);
			assert_ok!(IdavollModule::on_add_member(1,22,0));
			assert_eq!(IdavollModule::get_count_members(org_id),5);
			assert_ok!(IdavollModule::on_add_member(OWNER,23,0));
			assert_eq!(IdavollModule::get_count_members(org_id),6);
		});
	}

}
