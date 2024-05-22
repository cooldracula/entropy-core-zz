// Copyright (C) 2023 Entropy Cryptography Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for `pallet_registry`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 33.0.0
//! DATE: 2024-05-21, STEPS: `5`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Jesses-MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --pallet=pallet_registry
// --extrinsic=*
// --steps=5
// --repeat=2
// --header=.maintain/AGPL-3.0-header.txt
// --output=./runtime/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_registry`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_registry::WeightInfo for WeightInfo<T> {
	/// Storage: `Registry::Registering` (r:1 w:1)
	/// Proof: `Registry::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::Programs` (r:1 w:1)
	/// Proof: `Programs::Programs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::Dkg` (r:1 w:1)
	/// Proof: `Registry::Dkg` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[1, 5]`.
	fn register(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `358`
		//  Estimated: `3823`
		// Minimum execution time: 19_000_000 picoseconds.
		Weight::from_parts(17_650_000, 0)
			.saturating_add(Weight::from_parts(0, 3823))
			// Standard Error: 307_713
			.saturating_add(Weight::from_parts(1_850_000, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Registry::Registering` (r:1 w:1)
	/// Proof: `Registry::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::Programs` (r:1 w:1)
	/// Proof: `Programs::Programs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[1, 5]`.
	fn prune_registration(_p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `480`
		//  Estimated: `3945`
		// Minimum execution time: 15_000_000 picoseconds.
		Weight::from_parts(16_650_000, 0)
			.saturating_add(Weight::from_parts(0, 3945))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Programs::Programs` (r:2 w:2)
	/// Proof: `Programs::Programs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::Registered` (r:1 w:1)
	/// Proof: `Registry::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[1, 5]`.
	/// The range of component `o` is `[1, 5]`.
	fn change_program_instance(_n: u32, o: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `614 + o * (33 ±0)`
		//  Estimated: `6554 + o * (33 ±0)`
		// Minimum execution time: 34_000_000 picoseconds.
		Weight::from_parts(45_500_000, 0)
			.saturating_add(Weight::from_parts(0, 6554))
			// Standard Error: 1_762_184
			.saturating_add(Weight::from_parts(1_728_571, 0).saturating_mul(o.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 33).saturating_mul(o.into()))
	}
	/// Storage: `Registry::Registered` (r:1 w:1)
	/// Proof: `Registry::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::ModifiableKeys` (r:1 w:1)
	/// Proof: `Registry::ModifiableKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[1, 25]`.
	fn change_program_modification_account(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `328 + n * (1 ±0)`
		//  Estimated: `3793 + n * (1 ±0)`
		// Minimum execution time: 15_000_000 picoseconds.
		Weight::from_parts(14_875_000, 0)
			.saturating_add(Weight::from_parts(0, 3793))
			// Standard Error: 35_477
			.saturating_add(Weight::from_parts(125_000, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(n.into()))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::Registering` (r:1 w:1)
	/// Proof: `Registry::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::ValidatorToSubgroup` (r:1 w:0)
	/// Proof: `StakingExtension::ValidatorToSubgroup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registering(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1410`
		//  Estimated: `4875`
		// Minimum execution time: 27_000_000 picoseconds.
		Weight::from_parts(35_250_000, 0)
			.saturating_add(Weight::from_parts(0, 4875))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::Registering` (r:1 w:1)
	/// Proof: `Registry::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::ValidatorToSubgroup` (r:1 w:0)
	/// Proof: `StakingExtension::ValidatorToSubgroup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_failed_registering(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1412`
		//  Estimated: `4877`
		// Minimum execution time: 24_000_000 picoseconds.
		Weight::from_parts(29_166_666, 0)
			.saturating_add(Weight::from_parts(0, 4877))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::Registering` (r:1 w:1)
	/// Proof: `Registry::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::ValidatorToSubgroup` (r:1 w:0)
	/// Proof: `StakingExtension::ValidatorToSubgroup` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::ModifiableKeys` (r:1 w:1)
	/// Proof: `Registry::ModifiableKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Registry::Registered` (r:0 w:1)
	/// Proof: `Registry::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registered(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1436`
		//  Estimated: `4901`
		// Minimum execution time: 31_000_000 picoseconds.
		Weight::from_parts(44_583_333, 0)
			.saturating_add(Weight::from_parts(0, 4901))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
