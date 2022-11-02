// This file is part of Acala.

// Copyright (C) 2020-2022 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

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

//! Mocks for the transaction pause module.

#![cfg(test)]

use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, Everything},
};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use super::*;

pub type AccountId = u128;
pub type Balance = u128;

pub const ALICE: AccountId = 1;
mod transaction_pause {
    pub use super::super::*;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
    type BlockHashCount = ConstU64<250>;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type DbWeight = ();
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<AccountId>;
    type MaxConsumers = ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type PalletInfo = PalletInfo;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<10>;
    type MaxLocks = ();
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = ();
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

parameter_types! {
  pub const MaxWhitelist: u32 = 3;
  pub const MaxAddressLength: u32 = 2;
}

impl pallet_constraints::Config for Runtime {
    type MaxAddressLength = MaxAddressLength;
    type MaxWhitelist = MaxWhitelist;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

ord_parameter_types! {
  pub const One: AccountId = 1;
}

impl Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type UpdateOrigin = EnsureSignedBy<One, AccountId>;
    type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
  pub enum Runtime where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic
  {
    System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    TransactionPause: transaction_pause::{Pallet, Storage, Call, Event<T>},
    Balances: pallet_balances::{Pallet, Storage, Call, Event<T>},
    Constraints: pallet_constraints::{Pallet, Call, Storage, Event<T>},
  }
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self { ExtBuilder }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

        t.into()
    }
}
