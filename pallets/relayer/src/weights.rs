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

//! Autogenerated weights for pallet_relayer
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `hcastano`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet
// pallet_relayer
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --template
// .maintain/frame-weight-template.hbs
// --output
// pallets/relayer/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_relayer.
pub trait WeightInfo {
	fn register() -> Weight;
	fn prune_registration() -> Weight;
	fn change_program_pointer() -> Weight;
	fn confirm_register_registering(c: u32, ) -> Weight;
	fn confirm_register_failed_registering(c: u32, ) -> Weight;
	fn confirm_register_registered(c: u32, ) -> Weight;
}

/// Weights for pallet_relayer using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Relayer::Registered` (r:1 w:0)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Dkg` (r:1 w:1)
	/// Proof: `Relayer::Dkg` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `133`
		//  Estimated: `3598`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(17_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3598))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn prune_registration() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `167`
		//  Estimated: `3632`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_000_000, 3632)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Relayer::Registered` (r:1 w:1)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn change_program_pointer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `255`
		//  Estimated: `3720`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3720))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::SigningGroups` (r:1 w:0)
	/// Proof: `StakingExtension::SigningGroups` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registering(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16498`
		//  Estimated: `19963`
		// Minimum execution time: 34_000_000 picoseconds.
		Weight::from_parts(36_148_895, 19963)
			// Standard Error: 93_253
			.saturating_add(Weight::from_parts(1_086_740, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::SigningGroups` (r:1 w:0)
	/// Proof: `StakingExtension::SigningGroups` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_failed_registering(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16500`
		//  Estimated: `19965`
		// Minimum execution time: 33_000_000 picoseconds.
		Weight::from_parts(36_483_977, 19965)
			// Standard Error: 84_461
			.saturating_add(Weight::from_parts(57_734, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::SigningGroups` (r:1 w:0)
	/// Proof: `StakingExtension::SigningGroups` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registered` (r:0 w:1)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::AllowedToModifyProgram` (r:0 w:1)
	/// Proof: `Programs::AllowedToModifyProgram` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::Bytecode` (r:0 w:1)
	/// Proof: `Programs::Bytecode` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registered(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16499`
		//  Estimated: `19964`
		// Minimum execution time: 40_000_000 picoseconds.
		Weight::from_parts(42_758_839, 19964)
			// Standard Error: 68_268
			.saturating_add(Weight::from_parts(106_077, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Relayer::Registered` (r:1 w:0)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Dkg` (r:1 w:1)
	/// Proof: `Relayer::Dkg` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `133`
		//  Estimated: `3598`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(17_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3598))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn prune_registration() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `167`
		//  Estimated: `3632`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_000_000, 3632)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Relayer::Registered` (r:1 w:1)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn change_program_pointer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `255`
		//  Estimated: `3720`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3720))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::SigningGroups` (r:1 w:0)
	/// Proof: `StakingExtension::SigningGroups` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registering(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16498`
		//  Estimated: `19963`
		// Minimum execution time: 34_000_000 picoseconds.
		Weight::from_parts(36_148_895, 19963)
			// Standard Error: 93_253
			.saturating_add(Weight::from_parts(1_086_740, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::SigningGroups` (r:1 w:0)
	/// Proof: `StakingExtension::SigningGroups` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_failed_registering(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16500`
		//  Estimated: `19965`
		// Minimum execution time: 33_000_000 picoseconds.
		Weight::from_parts(36_483_977, 19965)
			// Standard Error: 84_461
			.saturating_add(Weight::from_parts(57_734, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `StakingExtension::ThresholdToStash` (r:1 w:0)
	/// Proof: `StakingExtension::ThresholdToStash` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `StakingExtension::SigningGroups` (r:1 w:0)
	/// Proof: `StakingExtension::SigningGroups` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registered` (r:0 w:1)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::AllowedToModifyProgram` (r:0 w:1)
	/// Proof: `Programs::AllowedToModifyProgram` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::Bytecode` (r:0 w:1)
	/// Proof: `Programs::Bytecode` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registered(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16499`
		//  Estimated: `19964`
		// Minimum execution time: 40_000_000 picoseconds.
		Weight::from_parts(42_758_839, 19964)
			// Standard Error: 68_268
			.saturating_add(Weight::from_parts(106_077, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
}
