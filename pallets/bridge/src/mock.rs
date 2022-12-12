use crate as pallet_bridge;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;
use frame_system::{limits, EnsureRoot};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use primitives::types::{AccountId, Balance, BlockNumber, TokenId};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Storage, Config<T>, Event<T>},
		Bridge: pallet_bridge::{Pallet, Call, Storage, Event<T>},
		TimestampPallet: pallet_timestamp::{Pallet, Call, Storage, Inherent},
	}
);

parameter_types! {
	pub BlockLength: limits::BlockLength = limits::BlockLength::max(2 * 1024);
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const AssetDeposit: Balance = 1_000_000;
	pub const AssetAccountDeposit: Balance = 16;
	pub const ApprovalDeposit: Balance = 1;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 1 * 68;
	pub const MetadataDepositPerByte: Balance = 1;
}
pub type AssetsForceOrigin = EnsureRoot<AccountId>;

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = TokenId;
	type Currency = Balances;
	type ForceOrigin = AssetsForceOrigin;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
	type AssetAccountDeposit = AssetAccountDeposit;
}

parameter_types! {
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ();
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

// Time is measured by number of blocks.
pub const MILLISECS_PER_BLOCK: u64 = 4_000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

parameter_types! {
	pub const TxChallengePeriod: u32 = 10 * MINUTES;
	pub const ClearTxPeriod: u32 = 10 * DAYS;
	pub const TokenAssetId: TokenId = 2;
}

impl pallet_bridge::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ChallengePeriod = TxChallengePeriod;
	type ClearTxPeriod = ClearTxPeriod;
	type UnixTime = TimestampPallet;
	type ApproveOrigin = EnsureRoot<Self::AccountId>;
	type Assets = Assets;
	type AssetId = TokenAssetId;
}
// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		Assets::force_create(RuntimeOrigin::root(), 2, create_account(b"6490B68F1116BFE87DDD"), true, 1).unwrap();
		Assets::force_set_metadata(
			RuntimeOrigin::root(),
			2,
			b"JUR".to_vec(),
			b"JUR".to_vec(),
			6,
			false,
		).unwrap();
	});
	ext
}


/// Helper function to create an AccountId from  a slice
pub fn create_account(address: &[u8]) -> AccountId {
	AccountId::from(H160::from_slice(address))
}

pub fn token(d: u128) -> u128 {
	d.saturating_mul(10_u128.pow(6))
}