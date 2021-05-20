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
	codec::{Decode},
	decl_module, decl_storage, decl_event, decl_error,
	dispatch::{
		self,Dispatchable, Parameter, PostDispatchInfo,
	},
	traits::{Get},
	weights::{GetDispatchInfo, Weight},
};
use frame_system::ensure_signed;
use sp_runtime::{
	ModuleId,
	traits::{StaticLookup, AccountIdConversion,
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
mod default_weights;

pub use organization::{OrgInfo, Proposal,ProposalDetail,ProposalDetailOf};
use idavoll_asset::{token::BaseToken,finance::BaseFinance};
use rules::{OrgRuleParam};


pub trait WeightInfo {
	fn create_origanization(m: u32) -> Weight;
	fn deposit_to_origanization() -> Weight;
	fn create_proposal() -> Weight;
	fn veto_proposal() -> Weight;
	fn add_member_by_onwer() -> Weight;
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

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
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
decl_storage! {
	trait Store for Module<T: Trait> as IdavollModule {
		pub OrgCounter get(fn counter): OrgCount = 0;
		pub OrgInfos get(fn org_infos): map hasher(blake2_128_concat) T::AccountId => Option<OrgInfoOf<T>>;
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
		/// it is not a member in the organization
		NotMemberInOrg,
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
		/// and all the tokens will assgined to the creator.
		///
		/// origin: the creator of the organization
		/// total: the total number of the new token
		/// info: the details of the new organization
		///
		#[weight = T::WeightInfo::create_origanization(info.members.len() as u32)]
		pub fn create_origanization(origin,total: T::Balance,info: OrgInfoOf<T>) -> dispatch::DispatchResult {
			let owner = ensure_signed(origin)?;
			let asset_id = Self::create_new_token(owner.clone(),total);
			let mut info_clone = info.clone();
			info_clone.add_member(owner.clone())?;
			info_clone.set_asset_id(asset_id.clone());
			Self::storage_new_organization(info_clone.clone())
		}
		/// reserve the local asset(idv) to organization's Vault, it used to assigned by the proposal
		/// of call function
		///
		/// id: Ordinal number created by the organization，it mapped whit the organization id.
		/// value: the amount of the local asset(IDV)
		///
		#[weight = T::WeightInfo::deposit_to_origanization()]
		pub fn deposit_to_origanization(origin,id: u32,value: T::Balance) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::on_reserve_to_vault(id,who,value)
		}

		/// voting on the proposal by the members in the organization,user must be lock it's 'value'
		/// of token amount to the idv-asset pallet and record the user's vote power (user's vote result)
		/// in the proposal storage. the proposal will finish and execute on the vote process when the
		/// vote result was satisfied with the rule of the proposal.
		///
		/// pid: the proposal id of the proposal return by create_proposal.
		/// value: the weight of vote power,it is the token amount of the token in the organization.
		/// yesorno: the user approve or against the proposal
		///
		#[weight = T::WeightInfo::veto_proposal()]
		pub fn vote_proposal(origin,pid: ProposalIdOf<T>,value: T::Balance,yesorno: bool) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::on_vote_proposal(pid,who,value,yesorno,frame_system::Module::<T>::block_number())
		}
		/// voting on the proposal by the members in the organization,user in the organization can add
		/// the other account into the organization, New members must obtain tokens before they can
		/// participate in voting.
		///
		/// target: the new account
		/// id: Ordinal number created by the organization，it mapped whit the organization id.
		///
		#[weight = T::WeightInfo::add_member_by_onwer()]
		pub fn add_member_by_onwer(origin,target: <T::Lookup as StaticLookup>::Source,id: u32) -> dispatch::DispatchResult {
			let owner = ensure_signed(origin)?;
			let who = T::Lookup::lookup(target)?;

			Self::on_add_member(owner,who,id)
		}
		/// create proposal in the organization for voting by members
		///
		/// id: Ordinal number created by the organization，it mapped whit the organization id.
		/// length: the block number(length) as the proposal lift time, if the current block number
		/// more than the 'length' than the proposal is expired.
		/// sub_param: the vote rule, it was satisfied with the organization's rule,more details in
		/// the 'RULE' Module
		///
		#[weight = T::WeightInfo::create_proposal()]
		pub fn create_proposal(origin,id: u32,length: T::BlockNumber,sub_param: OrgRuleParamOf<T>,
		call: Box<<T as Trait>::Call>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let cur = frame_system::Module::<T>::block_number();
			let expire = cur.saturating_add(length);
			Self::on_create_proposal(id,who,expire,sub_param,call)
		}
		/// the only way to use the vault of the organization. transfer the asset from
		/// organization'vault to the user by vote decision in the members.
		///
		#[weight = 100_000]
		pub fn transfer(
						origin,
		        		dest: <T::Lookup as StaticLookup>::Source,
						#[compact] value: T::Balance) -> dispatch::DispatchResult {
			let send = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			Self::handle_transfer_by_decision(send,dest,value)
		}
	}
}

impl<T: Trait> Module<T>  {

	/// be accountid of organization id for orginfos in the storage
	pub fn counter_2_orgid(c: OrgCount) -> T::AccountId {
		T::ModuleId::get().into_sub_account(c)
	}
	pub fn counter_of() -> OrgCount {
		OrgCounter::get()
	}
	/// get the count of the proposal in the storage
	pub fn count_of_proposals() -> u32 {
		let proposals = <Proposals<T>>::iter().map(|(v, _)| v).collect::<Vec<_>>();
		proposals.len() as u32
	}
	/// get the count of the proposal in the storage
	pub fn count_of_organizations() -> u32 {
		let orgs = <OrgInfos<T>>::iter().map(|(v, _)| v).collect::<Vec<_>>();
		orgs.len() as u32
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
	/// check the user belong to the organization
	pub fn is_member(oid: T::AccountId,who: &T::AccountId) -> bool {
		match <OrgInfos<T>>::get(oid.clone()) {
			Some(val) => val.is_member(who.clone()),
			None => false,
		}
	}
	/// return proposal info
	pub fn get_proposal_by_id(pid: ProposalIdOf<T>) -> Result<ProposalOf<T>, dispatch::DispatchError> {
		match Proposals::<T>::get(pid) {
			Some(proposal) => Ok(proposal),
			None => Err(Error::<T>::ProposalNotFound.into()),
		}
	}

	/// write the organization info to the storage on chain by create organization
	fn storage_new_organization(oinfo: OrgInfoOf<T>) -> dispatch::DispatchResult {
		let counter = OrgCounter::get();
		let new_counter = counter.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		let oid = Self::counter_2_orgid(counter);

		OrgInfos::<T>::insert(&oid, oinfo.clone());
		Self::deposit_event(RawEvent::OrganizationCreated(oid, oinfo));
		OrgCounter::put(new_counter);
		Ok(())
	}

	fn base_create_proposal(oid: T::AccountId,proposal: ProposalOf<T>) -> dispatch::DispatchResult {

		let proposal_id = Self::make_proposal_id(&proposal);
		if Proposals::<T>::contains_key(proposal_id) {
			return Err(Error::<T>::ProposalDuplicate.into());
		}
		Proposals::<T>::insert(&proposal_id, proposal.clone());
		Self::deposit_event(RawEvent::ProposalCreated(oid, proposal_id,proposal.creator()));
		Ok(())
	}

	// add a member into a organization by orgid
	fn base_add_member_on_orgid(oid: T::AccountId,member_id: T::AccountId) -> dispatch::DispatchResult {
		OrgInfos::<T>::try_mutate(oid,|infos| -> dispatch::DispatchResult {
			match infos {
				Some(org) => {match org.members
					.iter()
					.find(|&x| *x==member_id) {
					None => {
						org.members.push(member_id);
						Ok(())
					},
					_ => Ok(())
				}},
				None => Ok(()),
			}
		})
	}

	fn remove_proposal_by_id(pid: ProposalIdOf<T>) {
		Proposals::<T>::remove(pid);
	}
	/// add vote infos in the proposal item on the storage
	fn base_vote_on_proposal(pid: ProposalIdOf<T>, voter: T::AccountId,
								 value: BalanceOf<T>, yesorno: bool) -> dispatch::DispatchResult {
		Proposals::<T>::try_mutate(pid,|proposal| -> dispatch::DispatchResult {
			if let Some(p) = proposal {
				p.detail.vote(voter.clone(),value,yesorno);
				// *proposal = Some(p);
			};
			Ok(())
		})?;
		Ok(())
	}
	fn base_call_dispatch(pid: ProposalIdOf<T>,proposal: ProposalOf<T>) -> dispatch::DispatchResult {
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

	use frame_support::{
		codec::{Encode},impl_outer_origin,
		impl_outer_dispatch, assert_ok, assert_noop, parameter_types, weights::Weight};
	use sp_core::H256;
	use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup,Hash}, testing::Header,ModuleId};
	use pallet_balances;
	use organization::{OrgInfo, Proposal};
	use rules::{OrgRuleParam};
	use sp_std::{prelude::Vec, boxed::Box,collections::btree_map::BTreeMap};


	impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}
	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			frame_system::System,
			pallet_balances::IdvBalances,
			Self::IdavollModule,
        }
    }

	type System = frame_system::Module<Test>;
	type IdvBalances = pallet_balances::Module<Test>;
	type IdavollAsset = idavoll_asset::Module<Test>;
	type IdavollAssetError = idavoll_asset::Error<Test>;

	const A: u128 = 100;
	const B: u128 = 200;
	const OWNER: u128 = 88;
	const RECEIVER: u128 = 7;
	const ORGID: u128 = 1000;
	// const ORGID2: u128 = 2000;

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
	type IdavallCall = super::Call<Test>;
	impl Trait for Test {
		type Event = ();
		type Call = Call;
		type Balance = u64;
		type AssetId = u32;
		type ModuleId = IdavollModuleId;
		type AssetHandle = IdavollAsset;
		type Finance = IdavollAsset;
		type WeightInfo = ();
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let genesis = pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(A, 100000),
				(B, 200000),
			],
		};
		genesis.assimilate_storage(&mut t).unwrap();
		t.into()
	}
	fn set_block_number(n: <Test as frame_system::Trait>::BlockNumber) -> <Test as frame_system::Trait>::BlockNumber {
		System::set_block_number(n);
		n
	}
	fn get_block_number() -> <Test as frame_system::Trait>::BlockNumber {
		System::block_number()
	}
	fn make_transfer_fail_proposal(value: u64) -> Vec<u8> {
		Call::IdvBalances(pallet_balances::Call::transfer(RECEIVER.clone(), value)).encode()
	}
	fn make_transfer_proposal(value: u64) -> Vec<u8> {
		Call::IdavollModule(IdavallCall::transfer(RECEIVER.clone(),value)).encode()
	}
	fn make_system_proposal(_value: u64) -> Vec<u8> {
		Call::System(frame_system::Call::remark(vec![0; 1])).encode()
	}
	fn create_org() -> OrgInfoOf<Test> {
		let mut org = OrgInfo::new();
		org.members = vec![OWNER,1,2,3];
		org.param = OrgRuleParam::new(60,5,0);
		org.clone()
	}
	fn create_proposal(oid: <Test as frame_system::Trait>::AccountId,value: u64,
	owner: <Test as frame_system::Trait>::AccountId) -> ProposalOf<Test> {
		let sub_param = OrgRuleParam::new(60,5,0);
		Proposal {
			org:    oid.clone(),
			call: 	make_transfer_fail_proposal(value),
			detail: ProposalDetail::new(owner.clone(),5,sub_param.clone()),
		}
	}
	fn create_proposal2(call: Vec<u8>) -> ProposalOf<Test> {
		let sub_param = OrgRuleParam::new(60,5,0);
		Proposal {
			org:    ORGID.clone(),
			call: 	call.clone(),
			detail: ProposalDetail::new(OWNER.clone(),5,sub_param.clone()),
		}
	}
	fn create_proposal3(id: u128,call: Vec<u8>) -> ProposalOf<Test> {
		let sub_param = OrgRuleParam::new(60,5,0);
		Proposal {
			org:    id.clone(),
			call: 	call.clone(),
			detail: ProposalDetail::new(OWNER.clone(),5,sub_param.clone()),
		}
	}

	#[test]
	fn base_orgid_function_should_work() {
		new_test_ext().execute_with(|| {
			assert_ne!(IdavollModule::counter_2_orgid(100),IdavollModule::counter_2_orgid(101));
			println!("{}",IdavollModule::counter_2_orgid(0));
			println!("{}",IdavollModule::counter_2_orgid(1));
			println!("{}",IdavollModule::counter_2_orgid(2));
			println!("{}",IdavollModule::counter_2_orgid(4));
			println!("{}",IdavollModule::counter_2_orgid(5));
			println!("{}",IdavollModule::counter_2_orgid(6));
			println!("{}",IdavollModule::counter_2_orgid(7));
			println!("{}",IdavollModule::counter_2_orgid(8));
			println!("{}",IdavollModule::counter_2_orgid(9));
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
			assert_eq!(IdavollModule::get_orginfo_by_id(IdavollModule::counter_2_orgid(0)),Ok(org.clone()));

			assert_eq!(IdavollModule::is_member(IdavollModule::counter_2_orgid(0),&OWNER),true);
			assert_eq!(IdavollModule::is_member(IdavollModule::counter_2_orgid(0),&1),true);
			assert_eq!(IdavollModule::is_member(IdavollModule::counter_2_orgid(0),&9),false);

			for _i in 0..100 {
				let org = create_org();
				assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			}
			assert_eq!(IdavollModule::count_of_organizations(),100+1);
		});
	}

	#[test]
	fn base_organization_02_should_work() {

		new_test_ext().execute_with(|| {
			let mut org = create_org();
			let asset_id = IdavollModule::create_new_token(OWNER.clone(),100);
			assert_eq!(asset_id,0);
			org.set_asset_id(asset_id.clone());
			let org_id = IdavollModule::counter_2_orgid(0);
			assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			assert_eq!(IdavollModule::get_orginfo_by_id(org_id),Ok(org.clone()));
			assert_eq!(IdavollModule::get_count_members(org_id),4);
			// add member for the organization
			assert_noop!(IdavollModule::on_add_member(22,2,0),Error::<Test>::NotMemberInOrg);
			assert_noop!(IdavollModule::on_add_member(1,2,0),Error::<Test>::MemberDuplicate);
			assert_ok!(IdavollModule::on_add_member(1,22,0));
			assert_eq!(IdavollModule::get_count_members(org_id),5);
			assert_ok!(IdavollModule::on_add_member(OWNER,23,0));
			assert_eq!(IdavollModule::get_count_members(org_id),6);
		});
	}

	#[test]
	fn base_proposal_01_should_work() {

		new_test_ext().execute_with(|| {
			let org_id = IdavollModule::counter_2_orgid(0);
			let proposal = create_proposal(org_id,20,OWNER);
			assert_ok!(IdavollModule::base_create_proposal(org_id,proposal.clone()));
			// storage proposal repeat
			assert_noop!(IdavollModule::base_create_proposal(org_id,proposal.clone()),Error::<Test>::ProposalDuplicate);
			let proposal_id = IdavollModule::make_proposal_id(&proposal.clone());
			print!("{}",proposal_id);
			assert_eq!(IdavollModule::get_proposal_by_id(proposal_id),Ok(proposal.clone()));
			let pid2 = <Test as frame_system::Trait>::Hashing::hash_of(&0);
			assert_noop!(IdavollModule::get_proposal_by_id(pid2),Error::<Test>::ProposalNotFound);
			// remove proposal
			IdavollModule::remove_proposal_by_id(proposal_id);
			assert_noop!(IdavollModule::get_proposal_by_id(proposal_id),Error::<Test>::ProposalNotFound);
		});
	}

	#[test]
	fn base_proposal_02_should_work() {

		new_test_ext().execute_with(|| {
			set_block_number(0);
			for i in 0..10 {
				let org_id = IdavollModule::counter_2_orgid(i);
				let proposal = create_proposal(org_id,i as u64 * 100,OWNER);
				assert_ok!(IdavollModule::base_create_proposal(org_id,proposal.clone()));
				let proposal_id = IdavollModule::make_proposal_id(&proposal.clone());
				assert_eq!(IdavollModule::get_proposal_by_id(proposal_id),Ok(proposal.clone()));
				assert_eq!(proposal.detail.creator(),OWNER.clone());
				assert_eq!(set_block_number(i as u64),get_block_number());
				if get_block_number() > 5 {
					assert_eq!(proposal.detail.is_expire(get_block_number()),true);
				} else {
					assert_eq!(proposal.detail.is_expire(get_block_number()),false);
				}
			}
			assert_eq!(IdavollModule::count_of_proposals(),10);
		});
	}

	#[test]
	fn base_proposal_03_should_work() {
		new_test_ext().execute_with(||{
			// vote in the single proposal test(the proposal was a fake proposal)
			let _asset_id = IdavollModule::create_new_token(100,100);
			let mut proposal = create_proposal(100,10,OWNER);

			// passed by more than 60% 'Yes' votes and less than 5% 'no' votes

			// vote on decision 1
			for i in 0..10 {
				proposal.detail.vote(i,7,true);
			}
			for i in 10..15 {
				proposal.detail.vote(i,5,false);
			}
			assert_eq!(proposal.detail.summary(),(70,25));
			assert_eq!(proposal.detail.pass(100),false);

			// vote on decision 2
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,7,true);
			}
			for i in 10..13 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(70,3));
			assert_eq!(proposal.detail.pass(100),true);

			// vote on decision 3
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,8,true);
			}
			for i in 10..15 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(80,5));
			assert_eq!(proposal.detail.pass(100),true);

			// vote on decision 4
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,6,true);
			}
			for i in 10..12 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(60,2));
			assert_eq!(proposal.detail.pass(100),false);

			// vote on decision 5
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,7,true);
			}
			for i in 10..16 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(70,6));
			assert_eq!(proposal.detail.pass(100),false);

			// vote on decision 6
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,6,true);
			}
			for i in 10..15 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(60,5));
			assert_eq!(proposal.detail.pass(100),false);
		});
	}

	#[test]
	fn base_rule_param_should_work() {
		new_test_ext().execute_with(|| {
			let mut param = OrgRuleParam::default();
			// passed by 'min_affirmative' ,'max_dissenting' and 'abstention'
			param.min_affirmative = 70;param.max_dissenting = 0;param.abstention = 0;
			assert_eq!(param.is_pass(69 as u64,0,0,100),false);
			assert_eq!(param.is_pass(70 as u64,0,0,100),false);
			assert_eq!(param.is_pass(71 as u64,0,0,100),true);
			assert_eq!(param.is_pass(69 as u64,10,10,100),false);
			assert_eq!(param.is_pass(70 as u64,10,10,100),false);
			assert_eq!(param.is_pass(71 as u64,10,10,100),true);

			param.min_affirmative = 70;param.max_dissenting = 10;param.abstention = 0;
			assert_eq!(param.is_pass(69 as u64,10,1,100),false);
			assert_eq!(param.is_pass(70 as u64,10,1,100),false);
			assert_eq!(param.is_pass(71 as u64,10,1,100),true);
			assert_eq!(param.is_pass(69 as u64,9,1,100),false);
			assert_eq!(param.is_pass(70 as u64,10,1,100),false);
			assert_eq!(param.is_pass(71 as u64,11,1,100),false);
			assert_eq!(param.is_pass(71 as u64,9,1,100),true);
			assert_eq!(param.is_pass(71 as u64,10,1,100),true);
			assert_eq!(param.is_pass(71 as u64,9,10,100),true);
			assert_eq!(param.is_pass(71 as u64,10,10,100),true);

			param.min_affirmative = 70;param.max_dissenting = 10;param.abstention = 3;
			assert_eq!(param.is_pass(69 as u64,10,2,100),false);
			assert_eq!(param.is_pass(70 as u64,10,3,100),false);
			assert_eq!(param.is_pass(71 as u64,10,4,100),false);
			assert_eq!(param.is_pass(71 as u64,9,2,100),true);
			assert_eq!(param.is_pass(71 as u64,9,3,100),true);
			assert_eq!(param.is_pass(71 as u64,9,4,100),false);

			// sub param
			param.min_affirmative = 70;param.max_dissenting = 10;param.abstention = 3;
			let mut sub = OrgRuleParam::default();
			sub.min_affirmative = 70;sub.max_dissenting = 10;sub.abstention = 3;
			assert_eq!(param.inherit_valid(sub.clone()),true);  // same of the param

			sub.min_affirmative = 69;sub.max_dissenting = 10;sub.abstention = 3;
			assert_eq!(param.inherit_valid(sub.clone()),false);
			sub.min_affirmative = 71;sub.max_dissenting = 10;sub.abstention = 3;
			assert_eq!(param.inherit_valid(sub.clone()),true);

			sub.min_affirmative = 70;sub.max_dissenting = 9;sub.abstention = 3;
			assert_eq!(param.inherit_valid(sub.clone()),true);
			sub.min_affirmative = 70;sub.max_dissenting = 11;sub.abstention = 3;
			assert_eq!(param.inherit_valid(sub.clone()),false);

			sub.min_affirmative = 70;sub.max_dissenting = 10;sub.abstention = 2;
			assert_eq!(param.inherit_valid(sub.clone()),true);
			sub.min_affirmative = 70;sub.max_dissenting = 10;sub.abstention = 4;
			assert_eq!(param.inherit_valid(sub.clone()),false);
		});
	}

	#[test]
	fn base_dispatch_01_should_work() {
		new_test_ext().execute_with(|| {
			let proposal = create_proposal2(make_transfer_fail_proposal(10));
			assert_ok!(IdavollModule::base_create_proposal(ORGID.clone(),proposal.clone()));
			assert_ok!(IdavollModule::reserve_to_vault(ORGID.clone(),A.clone(),30));
			assert_eq!(IdavollModule::get_local_balance(ORGID),Ok(30));

			// because the real asset in the Vault of the Finance Module
			assert_eq!(IdvBalances::free_balance(ORGID.clone()),0);
			assert_noop!(IdavollModule::get_local_balance(RECEIVER),IdavollAssetError::UnknownOwnerID);

			// transfer the asset from organization id(it is fail),cause it's not transfer by direct
			let proposal_id = IdavollModule::make_proposal_id(&proposal.clone());
			assert_ok!(IdavollModule::base_call_dispatch(proposal_id,proposal.clone()));
			assert_noop!(IdavollModule::get_local_balance(RECEIVER),IdavollAssetError::UnknownOwnerID);
			assert_eq!(IdvBalances::free_balance(ORGID.clone()),0);

		});
	}

	#[test]
	fn base_dispatch_02_should_work() {
		new_test_ext().execute_with(|| {
			let proposal = create_proposal2(make_transfer_proposal(10));
			assert_ok!(IdavollModule::base_create_proposal(ORGID.clone(),proposal.clone()));
			assert_ok!(IdavollModule::reserve_to_vault(ORGID.clone(),A.clone(),30));
			assert_eq!(IdavollModule::get_local_balance(ORGID.clone()),Ok(30));
			assert_eq!(IdvBalances::free_balance(ORGID.clone()),0);


			// transfer the asset from organization id(it is success)
			let proposal_id = IdavollModule::make_proposal_id(&proposal.clone());
			assert_ok!(IdavollModule::base_call_dispatch(proposal_id,proposal.clone()));
			assert_eq!(IdavollModule::get_local_balance(ORGID.clone()),Ok(20));
			assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),10);
			// it's not store the members's balance only organization's balance in the idv-asset
			// on the local asset(idv)
			assert_noop!(IdavollModule::get_local_balance(RECEIVER),IdavollAssetError::UnknownOwnerID);

		});
	}

	#[test]
	fn base_dispatch_03_should_work() {
		new_test_ext().execute_with(|| {
			let mut sum = 0;
			for i in 10..100 {
				let proposal = create_proposal3(i as u128,make_transfer_proposal(i));
				assert_ok!(IdavollModule::base_create_proposal(i as u128,proposal.clone()));
				assert_ok!(IdavollModule::reserve_to_vault(i as u128,A.clone(),i));
				assert_eq!(IdavollModule::get_local_balance(i as u128),Ok(i));

				sum += i;
				// transfer the asset from organization id(it is success)
				let proposal_id = IdavollModule::make_proposal_id(&proposal.clone());
				assert_ok!(IdavollModule::base_call_dispatch(proposal_id,proposal.clone()));
				assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),sum);
				assert_noop!(IdavollModule::get_local_balance(RECEIVER),IdavollAssetError::UnknownOwnerID);
			}
		});
	}
}
