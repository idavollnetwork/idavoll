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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit="128"]


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
	fn create_organization(m: u32) -> Weight;
	fn deposit_to_organization() -> Weight;
	fn create_proposal() -> Weight;
	fn vote_proposal() -> Weight;
	fn add_member_and_assign_token() -> Weight;
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The outer call dispatch type.
	type Call: Parameter
	+ Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo>
	+ From<frame_system::Call<Self>>
	+ GetDispatchInfo;

	/// The idavoll pallet's module id, used for deriving the organization id.
	type ModuleId: Get<ModuleId>;

	/// The asset handler will handle all asset operations.
	type TokenHandler: BaseToken<
		Self::AccountId,
		AssetId = Self::TokenId,
		Balance = Self::Balance,
	>;

	type Balance: Member + Parameter + AtLeast32BitUnsigned + MaybeSerializeDeserialize + Default + Copy;
	/// the vaults of all organizations
	type Finance: BaseFinance<Self::AccountId,Self::Balance>;
	type TokenId: Parameter + AtLeast32Bit + Default + Copy;

	/// the staking balance of local asset by user create proposal.
	type InherentStakeProposal: Get<BalanceOf<Self>>;
	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

type BalanceOf<T> = <T as Trait>::Balance;
pub type OrgCount = u32;
pub type OrgInfoOf<T> = OrgInfo<
	<T as frame_system::Trait>::AccountId,
	BalanceOf<T>,
	<T as Trait>::TokenId,
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
		/// An organization was created with the following parameters. \[organization_id, organization_number, details\]
        OrganizationCreated(AccountId, u32, OrgInfo),
		/// A proposal has been finalized with the following result. \[proposal_id, result\]
        ProposalFinalized(ProposalId, dispatch::DispatchResult),
        /// A proposal has been passed. \[proposal_id]
        ProposalPassed(ProposalId),
        /// A proposal has been created.		\[organization_id, proposal_id, creator]
        ProposalCreated(AccountId,ProposalId,AccountId),
        /// Proposal refused or expired \[proposal_id]
        ProposalRefused(ProposalId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// need the maximum number for the storage value for the fixed type.
		StorageOverflow,
		OrganizationNotFound,
		TokenBalanceLow,
		/// it is not a member in the organization
		NotMemberInOrg,
		MemberDuplicate,
		/// not found the proposal by id in the runtime storage
		ProposalNotFound,
		ProposalDecodeFailed,
		ProposalDuplicate,
		ProposalExpired,
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
		const InherentStakeProposal: BalanceOf<T> = T::InherentStakeProposal::get();
		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// Create organization with the given organization info. Creator should set assetID=0 in
		/// the `info`, new class of token with `total` amount will be created and assigned to the creator.
		/// The organization id and number will be specified in the `OrganizationCreated` event.
		#[weight = T::WeightInfo::create_organization(info.members.len() as u32)]
		pub fn create_organization(origin, total: T::Balance, info: OrgInfoOf<T>) -> dispatch::DispatchResult {
			let owner = ensure_signed(origin)?;
			let asset_id = Self::create_new_token(owner.clone(),total);
			let mut info = info;
			info.add_member(owner)?;
			info.set_asset_id(asset_id);
			Self::storage_new_organization(info)
		}

		/// Deposit `value` assets(IDV) to organization's vault, which will be assigned by proposals.
		/// Note that the `id` is the organization number, not organization id.
		#[weight = T::WeightInfo::deposit_to_organization()]
		pub fn deposit_to_organization(origin, id: u32, value: T::Balance) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::on_reserve_to_vault(id, who, value)
		}

		/// Vote the proposal `pid`.
		/// The proposal id `id` is specified in the `ProposalCreated` event.
		/// Note that only members in the organization can vote. To take `value` vote weight,
		/// voter should lock `value` tokens. Tokens will be unlocked after the proposal is finish.
		/// And if the result is satisfied the rule, the proposal will be executed.
		#[weight = T::WeightInfo::vote_proposal()]
		pub fn vote_proposal(origin, pid: ProposalIdOf<T>, value: T::Balance, vote_for: bool) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::on_vote_proposal(pid, who, value, vote_for, frame_system::Module::<T>::block_number())
		}

		/// Add a new member to the organization and assign tokens to the new member.
		/// All members in the organization `id` can add accounts `target` into the organization.
		/// The member can assign `assigned_value` tokens to the new member.
		/// Note that the `id` is the organization number, not organization id.
		#[weight = T::WeightInfo::add_member_and_assign_token()]
		pub fn add_member_and_assign_token(origin, target: <T::Lookup as StaticLookup>::Source, id: u32,
		assigned_value: T::Balance) -> dispatch::DispatchResult {
			let owner = ensure_signed(origin)?;
			let who = T::Lookup::lookup(target)?;

			Self::on_add_member_and_assign_token(owner, who, id, assigned_value)
		}

		/// Create a proposal to vote. The creator must be the member of the organization,
		/// and to prevent "spamming", creating a new proposal could require some assets(The quantity 
		/// is specified by `InherentStakeProposal`).
		/// `length` is the voting time (metric in block numbers), expired time is set to the
		/// block number the proposal created plus `length`. The `sub_param` is the vote rule
		/// and statisfied by the organization's rule, more details in the 'RULE' Module
		/// Note that the `id` is the organization number, not organization id,The successful 
		/// creation of the proposal will lock some assets, and the closing of the proposal
		/// will unlock the assets.
		#[weight = T::WeightInfo::create_proposal()]
		pub fn create_proposal(origin, id: u32, length: T::BlockNumber, sub_param: OrgRuleParamOf<T>,
		call: Box<<T as Trait>::Call>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let cur = frame_system::Module::<T>::block_number();
			let expire = cur.saturating_add(length);
			Self::on_create_proposal(id,who,expire,sub_param,call)
		}

		/// Transfer the assets(IDV) from the vault of the organization to the dest account.
		/// The only way to use the vault of the organization is to propose a proposal and vote for it.
		#[weight = 100_000]
		pub fn vault_transfer(
						origin,
		        		dest: <T::Lookup as StaticLookup>::Source,
						#[compact] value: T::Balance) -> dispatch::DispatchResult {
			let send = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			Self::handle_transfer_by_decision(send, dest, value)
		}
	}
}

impl<T: Trait> Module<T>  {

	/// Generate organization id, which is used as the key of organization info storage
	pub fn counter_2_orgid(c: OrgCount) -> T::AccountId {
		T::ModuleId::get().into_sub_account(c)
	}
	pub fn counter_of() -> OrgCount {
		OrgCounter::get()
	}
	/// Get the count of the proposals in storage
	pub fn count_of_proposals() -> u32 {
		<Proposals<T>>::iter().map(|(v, _)| v).count() as u32
	}
	/// Get the count of the proposals in storage
	pub fn count_of_organizations() -> u32 {
		<OrgInfos<T>>::iter().map(|(v, _)| v).count() as u32
	}
	pub fn get_orginfo_by_id(oid: T::AccountId) -> Result<OrgInfoOf<T>, dispatch::DispatchError> {
		if OrgInfos::<T>::contains_key(oid.clone()) {
			match <OrgInfos<T>>::get(oid) {
				Some(val) => Ok(val),
				None => Err(Error::<T>::OrganizationNotFound.into()),
			}
		}else {
			Err(Error::<T>::OrganizationNotFound.into())
		}
	}
	/// Check whether the user belongs to the organization
	pub fn is_member(oid: T::AccountId, who: &T::AccountId) -> bool {
		match <OrgInfos<T>>::get(oid) {
			Some(val) => val.is_member(who.clone()),
			None => false,
		}
	}
	/// Get the info of proposal `pid`
	pub fn get_proposal_by_id(pid: ProposalIdOf<T>) -> Result<ProposalOf<T>, dispatch::DispatchError> {
		match Proposals::<T>::get(pid) {
			Some(proposal) => Ok(proposal),
			None => Err(Error::<T>::ProposalNotFound.into()),
		}
	}

	/// Storage the info of the new created organization
	fn storage_new_organization(oinfo: OrgInfoOf<T>) -> dispatch::DispatchResult {
		let counter = OrgCounter::get();
		let new_counter = counter.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		let oid = Self::counter_2_orgid(counter);

		OrgInfos::<T>::insert(&oid, oinfo.clone());
		Self::deposit_event(RawEvent::OrganizationCreated(oid, counter, oinfo));
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

	/// Add a member into the organization by org id
	fn base_add_member_by_orgid(oid: T::AccountId, member_id: T::AccountId) -> dispatch::DispatchResult {
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

	/// Add vote infos in the proposal item
	fn base_vote_on_proposal(pid: ProposalIdOf<T>, voter: T::AccountId,
								 value: BalanceOf<T>, vote_for: bool) -> dispatch::DispatchResult {
		Proposals::<T>::try_mutate(pid,|proposal| -> dispatch::DispatchResult {
			if let Some(p) = proposal {
				p.detail.vote(voter.clone(),value, vote_for);
				// *proposal = Some(p);
			};
			Ok(())
		})?;
		Ok(())
	}
	fn base_call_dispatch(pid: ProposalIdOf<T>,proposal: ProposalOf<T>) -> dispatch::DispatchResult {
		// remove the proposal from the storage by the proposal passed
		let call = <T as Trait>::Call::decode(&mut &proposal.call[..]).map_err(|_| Error::<T>::ProposalDecodeFailed)?;
		let res = call.dispatch(frame_system::RawOrigin::Signed(proposal.org).into());
		Self::deposit_event(RawEvent::ProposalFinalized(pid, res.map(|_| ()).map_err(|e| e.error)));
		Ok(())
	}
}


#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::{
		A,OWNER,RECEIVER,
		set_block_number,get_block_number,create_org,new_test_ext,
	};
	use frame_support::{
		codec::{Encode},impl_outer_origin,
		impl_outer_dispatch, assert_ok, assert_noop, parameter_types, weights::Weight};
	use sp_core::H256;
	use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup,Hash}, testing::Header,ModuleId};
	use pallet_balances;
	use organization::{Proposal};
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
	const ORGID: u128 = 1000;

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
	pub const InherentStakeProposal: u64 = 1;
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
		type TokenId = u32;
		type ModuleId = IdavollModuleId;
		type TokenHandler = IdavollAsset;
		type Finance = IdavollAsset;
		type InherentStakeProposal = InherentStakeProposal;
		type WeightInfo = ();
	}

	fn make_transfer_fail_proposal(value: u64) -> Vec<u8> {
		Call::IdvBalances(pallet_balances::Call::transfer(RECEIVER.clone(), value)).encode()
	}
	fn make_transfer_proposal(value: u64) -> Vec<u8> {
		Call::IdavollModule(IdavallCall::vault_transfer(RECEIVER.clone(),value)).encode()
	}
	// fn make_system_proposal(_value: u64) -> Vec<u8> {
	// 	Call::System(frame_system::Call::remark(vec![0; 1])).encode()
	// }

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
			let mut org = create_org(vec![OWNER,1,2,3]);
			let asset_id = IdavollModule::create_new_token(OWNER.clone(),100);
			assert_eq!(asset_id,0);
			org.set_asset_id(asset_id.clone());
			assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			assert_eq!(IdavollModule::get_orginfo_by_id(IdavollModule::counter_2_orgid(0)),Ok(org.clone()));

			assert_eq!(IdavollModule::is_member(IdavollModule::counter_2_orgid(0),&OWNER),true);
			assert_eq!(IdavollModule::is_member(IdavollModule::counter_2_orgid(0),&1),true);
			assert_eq!(IdavollModule::is_member(IdavollModule::counter_2_orgid(0),&9),false);

			for _i in 0..100 {
				let org = create_org(vec![OWNER,1,2,3]);
				assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			}
			assert_eq!(IdavollModule::count_of_organizations(),100+1);
		});
	}

	#[test]
	fn base_organization_02_should_work() {

		new_test_ext().execute_with(|| {
			let mut org = create_org(vec![OWNER,1,2,3]);
			let asset_id = IdavollModule::create_new_token(OWNER.clone(),100);
			assert_eq!(asset_id,0);
			org.set_asset_id(asset_id.clone());
			let org_id = IdavollModule::counter_2_orgid(0);
			assert_ok!(IdavollModule::storage_new_organization(org.clone()));
			assert_eq!(IdavollModule::get_orginfo_by_id(org_id),Ok(org.clone()));
			assert_eq!(IdavollModule::get_count_members(org_id),4);
			// add member for the organization
			assert_noop!(IdavollModule::on_add_member_and_assign_token(22,2,0,0),Error::<Test>::NotMemberInOrg);
			assert_noop!(IdavollModule::on_add_member_and_assign_token(1,2,0,0),Error::<Test>::MemberDuplicate);
			assert_noop!(IdavollModule::on_add_member_and_assign_token(1,2,0,22),Error::<Test>::TokenBalanceLow);

			assert_ok!(IdavollModule::on_add_member_and_assign_token(OWNER,22,0,22));
			assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER),78);
			assert_eq!(IdavollAsset::free_balance(asset_id,&22),22);
			assert_eq!(IdavollModule::get_count_members(org_id),5);

			assert_ok!(IdavollModule::on_add_member_and_assign_token(OWNER,23,0,8));
			assert_eq!(IdavollModule::get_count_members(org_id),6);
			assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER),70);
			assert_eq!(IdavollAsset::free_balance(asset_id,&23),8);
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
					assert_eq!(proposal.detail.is_expired(get_block_number()), true);
				} else {
					assert_eq!(proposal.detail.is_expired(get_block_number()), false);
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
			assert_eq!(proposal.detail.is_passed(100), false);

			// vote on decision 2
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,7,true);
			}
			for i in 10..13 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(70,3));
			assert_eq!(proposal.detail.is_passed(100), true);

			// vote on decision 3
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,8,true);
			}
			for i in 10..15 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(80,5));
			assert_eq!(proposal.detail.is_passed(100), true);

			// vote on decision 4
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,6,true);
			}
			for i in 10..12 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(60,2));
			assert_eq!(proposal.detail.is_passed(100), false);

			// vote on decision 5
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,7,true);
			}
			for i in 10..16 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(70,6));
			assert_eq!(proposal.detail.is_passed(100), false);

			// vote on decision 6
			proposal.detail.votes = BTreeMap::new();
			for i in 0..10 {
				proposal.detail.vote(i,6,true);
			}
			for i in 10..15 {
				proposal.detail.vote(i,1,false);
			}
			assert_eq!(proposal.detail.summary(),(60,5));
			assert_eq!(proposal.detail.is_passed(100), false);
		});
	}

	#[test]
	fn base_rule_param_should_work() {
		new_test_ext().execute_with(|| {
			let mut param = OrgRuleParam::default();
			// passed by 'min_affirmative' ,'max_dissenting' and 'abstention'
			param.min_affirmative = 70;param.max_dissenting = 0;param.abstention = 0;
			assert_eq!(param.is_passed(69 as u64, 0, 0, 100), false);
			assert_eq!(param.is_passed(70 as u64, 0, 0, 100), false);
			assert_eq!(param.is_passed(71 as u64, 0, 0, 100), true);
			assert_eq!(param.is_passed(69 as u64, 10, 10, 100), false);
			assert_eq!(param.is_passed(70 as u64, 10, 10, 100), false);
			assert_eq!(param.is_passed(71 as u64, 10, 10, 100), true);

			param.min_affirmative = 70;param.max_dissenting = 10;param.abstention = 0;
			assert_eq!(param.is_passed(69 as u64, 10, 1, 100), false);
			assert_eq!(param.is_passed(70 as u64, 10, 1, 100), false);
			assert_eq!(param.is_passed(71 as u64, 10, 1, 100), true);
			assert_eq!(param.is_passed(69 as u64, 9, 1, 100), false);
			assert_eq!(param.is_passed(70 as u64, 10, 1, 100), false);
			assert_eq!(param.is_passed(71 as u64, 11, 1, 100), false);
			assert_eq!(param.is_passed(71 as u64, 9, 1, 100), true);
			assert_eq!(param.is_passed(71 as u64, 10, 1, 100), true);
			assert_eq!(param.is_passed(71 as u64, 9, 10, 100), true);
			assert_eq!(param.is_passed(71 as u64, 10, 10, 100), true);

			param.min_affirmative = 70;param.max_dissenting = 10;param.abstention = 3;
			assert_eq!(param.is_passed(69 as u64, 10, 2, 100), false);
			assert_eq!(param.is_passed(70 as u64, 10, 3, 100), false);
			assert_eq!(param.is_passed(71 as u64, 10, 4, 100), false);
			assert_eq!(param.is_passed(71 as u64, 9, 2, 100), true);
			assert_eq!(param.is_passed(71 as u64, 9, 3, 100), true);
			assert_eq!(param.is_passed(71 as u64, 9, 4, 100), false);

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
