use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin,impl_outer_dispatch, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,ModuleId,
};
use frame_system as system;

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

type System = frame_system::Module<Test>;
type IdvBalances = pallet_balances::Module<Test>;
type IdavollAsset = idavoll_asset::Module<Test>;

const A: u64 = 100;
const B: u64 = 200;
const ORGID: u64 = 1000;
const ORGID2: u64 = 2000;

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			frame_system::System,
        }
    }

parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
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
	type AccountId = u64;
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


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
