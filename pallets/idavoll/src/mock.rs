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

// use crate::{Module, Trait,OrgInfoOf,ProposalOf,OrgInfo,OrgRuleParam,
// 			Proposal,ProposalDetail};
use super::*;
use crate as idavoll;
use frame_support::{
	codec::{Encode},
	impl_outer_origin,impl_outer_dispatch,
	parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup}, testing::Header,ModuleId};
use pallet_balances;
use frame_system::RawOrigin;
use sp_std::{prelude::Vec, boxed::Box};


impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}
impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			frame_system::System,
			// pallet_balances::IdvBalances,
			idavoll::IdavollModule,
        }
    }

pub type System = frame_system::Module<Test>;
pub type IdvBalances = pallet_balances::Module<Test>;
pub type IdavollAsset = idavoll_asset::Module<Test>;

pub const A: u128 = 100;
pub const B: u128 = 200;
pub const OWNER: u128 = 88;
pub const RECEIVER: u128 = 77;
// pub const ORGID: u128 = 1000;
// pub const ORGID2: u128 = 2000;

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

pub type IdavollModule = Module<Test>;
type IdavallCall = idavoll::Call<Test>;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
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
pub fn set_block_number(n: <Test as frame_system::Trait>::BlockNumber) -> <Test as frame_system::Trait>::BlockNumber {
	System::set_block_number(n);
	n
}
pub fn get_block_number() -> <Test as frame_system::Trait>::BlockNumber {
	System::block_number()
}
pub fn call_to_vec(call: Box<<Test as Trait>::Call>) -> Vec<u8> {
	call.encode()
}
pub fn make_transfer_proposal(value: u64) -> Box<Call> {
	Box::new(Call::IdavollModule(IdavallCall::transfer(RECEIVER.clone(),value)))
}

pub fn create_org(_creator: u128) -> OrgInfoOf<Test> {
	let mut org = OrgInfo::new();
	org.members = vec![];
	org.param = get_rule();
	org.clone()
}
pub fn get_rule() -> OrgRuleParam<u64> {
	OrgRuleParam::new(60,5,0)
}
pub fn create_proposal_without_storage(id: u128,expire: u64,call: Vec<u8>) -> ProposalOf<Test> {
	let mut cur = get_block_number();
	cur = cur + expire;
	Proposal {
		org:    id.clone(),
		call: 	call.clone(),
		detail: ProposalDetail::new(OWNER.clone(),cur,get_rule()),
	}
}

pub fn create_new_organization(creator: u128,total: u64) -> u128 {
	let info = create_org(creator);
	let c = IdavollModule::counter_of();
	match IdavollModule::create_organization(RawOrigin::Signed(creator).into(),total,info) {
		Ok(_val) => {
			IdavollModule::counter_2_orgid(c)
		},
		Err(_e) => u128::MAX,
	}
}
