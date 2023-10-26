#![allow(clippy::all)]

//! Autogenerated weights for `pallet_relayer`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-10-20, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `hcastano`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/debug/entropy
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_relayer
// --extrinsic=*
// --steps=50
// --repeat=20
// --header=./file_header.txt
// --output=./runtime/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_relayer`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_relayer::WeightInfo for WeightInfo<T> {
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
		// Minimum execution time: 259_000_000 picoseconds.
		Weight::from_parts(269_633_444, 0)
			.saturating_add(Weight::from_parts(0, 3598))
			// Standard Error: 3
			.saturating_add(Weight::from_parts(535, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: StakingExtension ThresholdToStash (r:1 w:0)
	/// Proof Skipped: StakingExtension ThresholdToStash (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: StakingExtension SigningGroups (r:1 w:0)
	/// Proof Skipped: StakingExtension SigningGroups (max_values: None, max_size: None, mode: Measured)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registering(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16478`
		//  Estimated: `19943`
		// Minimum execution time: 300_000_000 picoseconds.
		Weight::from_parts(351_356_077, 0)
			.saturating_add(Weight::from_parts(0, 19943))
			// Standard Error: 1_167_125
			.saturating_add(Weight::from_parts(472_928, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: StakingExtension ThresholdToStash (r:1 w:0)
	/// Proof Skipped: StakingExtension ThresholdToStash (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registering (r:1 w:1)
	/// Proof Skipped: Relayer Registering (max_values: None, max_size: None, mode: Measured)
	/// Storage: StakingExtension SigningGroups (r:1 w:0)
	/// Proof Skipped: StakingExtension SigningGroups (max_values: None, max_size: None, mode: Measured)
	/// Storage: Relayer Registered (r:0 w:1)
	/// Proof Skipped: Relayer Registered (max_values: None, max_size: None, mode: Measured)
	/// Storage: Programs V2Bytecode (r:0 w:1)
	/// Proof Skipped: Programs V2Bytecode (max_values: None, max_size: None, mode: Measured)
	/// Storage: Programs AllowedToModifyPrograms (r:0 w:1)
	/// Proof Skipped: Programs AllowedToModifyPrograms (max_values: None, max_size: None, mode: Measured)
	/// The range of component `c` is `[0, 2]`.
	fn confirm_register_registered(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16479`
		//  Estimated: `19944`
		// Minimum execution time: 388_000_000 picoseconds.
		Weight::from_parts(411_493_093, 0)
			.saturating_add(Weight::from_parts(0, 19944))
			// Standard Error: 1_057_201
			.saturating_add(Weight::from_parts(9_067_127, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
