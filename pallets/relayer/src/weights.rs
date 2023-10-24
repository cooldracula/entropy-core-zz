
//! Autogenerated weights for pallet_relayer
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-10-20, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `hcastano`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/debug/entropy
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
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
	fn register(p: u32, ) -> Weight;
	fn confirm_register_registering(c: u32, ) -> Weight;
	fn confirm_register_registered(c: u32, ) -> Weight;
}

/// Weights for pallet_relayer using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: Relayer Registered (r:1 w:0)
	/// Proof Skipped: Relayer Registered (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Dkg (r:1 w:1)
	/// Proof Skipped: Relayer Dkg (max_values: None, max_size: None, mode: Measured)
	/// The range of component `p` is `[0, 1000000]`.
	fn register(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `133`
		//  Estimated: `3598`
		// Minimum execution time: 268_000_000 picoseconds.
		Weight::from_parts(288_147_322, 3598)
			// Standard Error: 4
			.saturating_add(Weight::from_parts(528, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: StakingExtension ThresholdToStash (r:1 w:0)
	/// Proof Skipped: StakingExtension ThresholdToStash (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: StakingExtension SigningGroups (r:1 w:0)
	/// Proof Skipped: StakingExtension SigningGroups (max_values: None, max_size: None, mode: Measured)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registering(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16478`
		//  Estimated: `19943`
		// Minimum execution time: 303_000_000 picoseconds.
		Weight::from_parts(358_564_364, 19943)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: StakingExtension ThresholdToStash (r:1 w:0)
	/// Proof Skipped: StakingExtension ThresholdToStash (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: StakingExtension SigningGroups (r:1 w:0)
	/// Proof Skipped: StakingExtension SigningGroups (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registered (r:0 w:1)
	/// Proof Skipped: Relayer Registered (max_values: None, max_size: None, mode: Measured)
	/// Storage: Constraints V2Bytecode (r:0 w:1)
	/// Proof Skipped: Constraints V2Bytecode (max_values: None, max_size: None, mode: Measured)
	/// Storage: Constraints AllowedToModifyConstraints (r:0 w:1)
	/// Proof Skipped: Constraints AllowedToModifyConstraints (max_values: None, max_size: None, mode: Measured)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registered(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16479`
		//  Estimated: `19944`
		// Minimum execution time: 390_000_000 picoseconds.
		Weight::from_parts(435_656_077, 19944)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: Relayer Registered (r:1 w:0)
	/// Proof Skipped: Relayer Registered (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Dkg (r:1 w:1)
	/// Proof Skipped: Relayer Dkg (max_values: None, max_size: None, mode: Measured)
	/// The range of component `p` is `[0, 1000000]`.
	fn register(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `133`
		//  Estimated: `3598`
		// Minimum execution time: 268_000_000 picoseconds.
		Weight::from_parts(288_147_322, 3598)
			// Standard Error: 4
			.saturating_add(Weight::from_parts(528, 0).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: StakingExtension ThresholdToStash (r:1 w:0)
	/// Proof Skipped: StakingExtension ThresholdToStash (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: StakingExtension SigningGroups (r:1 w:0)
	/// Proof Skipped: StakingExtension SigningGroups (max_values: None, max_size: None, mode: Measured)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registering(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16478`
		//  Estimated: `19943`
		// Minimum execution time: 303_000_000 picoseconds.
		Weight::from_parts(358_564_364, 19943)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: StakingExtension ThresholdToStash (r:1 w:0)
	/// Proof Skipped: StakingExtension ThresholdToStash (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: StakingExtension SigningGroups (r:1 w:0)
	/// Proof Skipped: StakingExtension SigningGroups (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registered (r:0 w:1)
	/// Proof Skipped: Relayer Registered (max_values: None, max_size: None, mode: Measured)
	/// Storage: Constraints V2Bytecode (r:0 w:1)
	/// Proof Skipped: Constraints V2Bytecode (max_values: None, max_size: None, mode: Measured)
	/// Storage: Constraints AllowedToModifyConstraints (r:0 w:1)
	/// Proof Skipped: Constraints AllowedToModifyConstraints (max_values: None, max_size: None, mode: Measured)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registered(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16479`
		//  Estimated: `19944`
		// Minimum execution time: 390_000_000 picoseconds.
		Weight::from_parts(435_656_077, 19944)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
}
