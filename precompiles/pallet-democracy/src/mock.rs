// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Test utilities
use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, OnFinalize, OnInitialize},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_democracy::VoteThreshold;
use pallet_evm::{
	AddressMapping, EnsureAddressNever, EnsureAddressRoot, SubstrateBlockHashMapping,
};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub type AccountId = TestAccount;
pub type Balance = u128;
pub type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

#[derive(
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Clone,
	Encode,
	Decode,
	Debug,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	derive_more::Display,
)]
pub enum TestAccount {
	Alice,
	Bob,
	Charlie,
	Bogus,
}

impl Default for TestAccount {
	fn default() -> Self {
		Self::Bogus
	}
}

impl AddressMapping<TestAccount> for TestAccount {
	fn into_account_id(h160_account: H160) -> TestAccount {
		match h160_account {
			a if a == H160::repeat_byte(0xAA) => Self::Alice,
			a if a == H160::repeat_byte(0xBB) => Self::Bob,
			a if a == H160::repeat_byte(0xCC) => Self::Charlie,
			_ => Self::Bogus,
		}
	}
}

impl From<TestAccount> for H160 {
	fn from(value: TestAccount) -> H160 {
		match value {
			TestAccount::Alice => H160::repeat_byte(0xAA),
			TestAccount::Bob => H160::repeat_byte(0xBB),
			TestAccount::Charlie => H160::repeat_byte(0xCC),
			TestAccount::Bogus => Default::default(),
		}
	}
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Config, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = TestAccount;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

/// The democracy precompile is available at address 1 in the mock runtime.
pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}
pub type Precompiles = (DemocracyWrapper<Test>,);

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<TestAccount>;
	type WithdrawOrigin = EnsureAddressNever<TestAccount>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = Precompiles;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
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

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 10;
	pub const VotingPeriod: BlockNumber = 10;
	pub const FastTrackVotingPeriod: BlockNumber = 5;
	pub const EnactmentPeriod: BlockNumber = 10;
	pub const CooloffPeriod: BlockNumber = 10;
	pub const MinimumDeposit: Balance = 10;
	pub const MaxVotes: u32 = 10;
	pub const MaxProposals: u32 = 10;
	pub const PreimageByteDeposit: Balance = 10;
	pub const InstantAllowed: bool = false;
}

impl pallet_democracy::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	type ExternalOrigin = EnsureRoot<AccountId>;
	type ExternalMajorityOrigin = EnsureRoot<AccountId>;
	type ExternalDefaultOrigin = EnsureRoot<AccountId>;
	type FastTrackOrigin = EnsureRoot<AccountId>;
	type InstantOrigin = EnsureRoot<AccountId>;
	type CancellationOrigin = EnsureRoot<AccountId>;
	type CancelProposalOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type VetoOrigin = EnsureSigned<AccountId>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = ();
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = EnsureSigned<AccountId>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = MaxProposals;
}
impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = ();
	type ScheduleOrigin = EnsureRoot<TestAccount>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
}

/// Build test externalities, prepopulated with data for testing democracy precompiles
pub(crate) struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	/// Referenda that already exist (don't need a proposal and launch period delay)
	referenda: Vec<(H256, VoteThreshold, BlockNumber)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			referenda: vec![],
		}
	}
}

impl ExtBuilder {
	/// Fund some accounts before starting the test
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	/// Put some referenda into storage before starting the test
	pub(crate) fn with_referenda(
		mut self,
		referenda: Vec<(H256, VoteThreshold, BlockNumber)>,
	) -> Self {
		self.referenda = referenda;
		self
	}

	/// Build the test externalities for use in tests
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances.clone(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);

			// Pallet democracy doesn't have a meaningful genesis config, so we use
			// its helper method to initialize the referenda
			for (hash, thresh, delay) in self.referenda {
				Democracy::internal_start_referendum(hash, thresh, delay);
			}
		});
		ext
	}
}

pub(crate) fn roll_to(n: u64) {
	// We skip timestamp's on_finalize because it requires that the timestamp inherent be set
	// We may be able to simulate this by poking its storage directly, but I don't see any value
	// added from doing that.
	while System::block_number() < n {
		Scheduler::on_finalize(System::block_number());
		Democracy::on_finalize(System::block_number());
		// Timestamp::on_finalize(System::block_number());
		Evm::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());

		System::set_block_number(System::block_number() + 1);

		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		Evm::on_initialize(System::block_number());
		Timestamp::on_initialize(System::block_number());
		Democracy::on_initialize(System::block_number());
		Scheduler::on_initialize(System::block_number());
	}
}

pub(crate) fn events() -> Vec<Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}

// Helper function to give a simple evm context suitable for tests.
// We can remove this once https://github.com/rust-blockchain/evm/pull/35
// is in our dependency graph.
pub fn evm_test_context() -> evm::Context {
	evm::Context {
		address: Default::default(),
		caller: Default::default(),
		apparent_value: From::from(0),
	}
}

#[test]
fn test_account_id_mapping_works() {
	// Bidirectional conversions for normal accounts
	assert_eq!(
		TestAccount::Alice,
		TestAccount::into_account_id(TestAccount::Alice.into())
	);
	assert_eq!(
		TestAccount::Bob,
		TestAccount::into_account_id(TestAccount::Bob.into())
	);
	assert_eq!(
		TestAccount::Charlie,
		TestAccount::into_account_id(TestAccount::Charlie.into())
	);

	// Bidirectional conversion between bogus and default H160
	assert_eq!(
		TestAccount::Bogus,
		TestAccount::into_account_id(H160::default())
	);
	assert_eq!(H160::default(), TestAccount::Bogus.into());

	// All other H160s map to bogus
	assert_eq!(
		TestAccount::Bogus,
		TestAccount::into_account_id(H160::zero())
	);
	assert_eq!(
		TestAccount::Bogus,
		TestAccount::into_account_id(H160::repeat_byte(0x12))
	);
	assert_eq!(
		TestAccount::Bogus,
		TestAccount::into_account_id(H160::repeat_byte(0xFF))
	);
}
