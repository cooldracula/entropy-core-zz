
//! Autogenerated weights for pallet_programs
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-02, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// pallet_programs
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --template
// .maintain/frame-weight-template.hbs
// --output
// pallets/programs/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_programs.
pub trait WeightInfo {
	fn set_program() -> Weight;
	fn remove_program(p: u32) -> Weight;
}

/// Weights for pallet_programs using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Programs::Bytecode` (r:1 w:1)
	/// Proof: `Programs::Bytecode` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::OwnedPrograms` (r:1 w:1)
	/// Proof: `Programs::OwnedPrograms` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_program() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `142`
		//  Estimated: `3607`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3607))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Programs::Bytecode` (r:1 w:1)
	/// Proof: `Programs::Bytecode` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::OwnedPrograms` (r:1 w:1)
	/// Proof: `Programs::OwnedPrograms` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[0, 25]`.
	fn remove_program(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `326 + p * (32 ±0)`
		//  Estimated: `3809 + p * (31 ±0)`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(26_938_669, 0)
			.saturating_add(Weight::from_parts(0, 3809))
			// Standard Error: 47_904
			.saturating_add(Weight::from_parts(136_174, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 31).saturating_mul(p.into()))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Programs::Bytecode` (r:1 w:1)
	/// Proof: `Programs::Bytecode` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::OwnedPrograms` (r:1 w:1)
	/// Proof: `Programs::OwnedPrograms` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_program() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `142`
		//  Estimated: `3607`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3607))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	/// Storage: `Programs::Bytecode` (r:1 w:1)
	/// Proof: `Programs::Bytecode` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Programs::OwnedPrograms` (r:1 w:1)
	/// Proof: `Programs::OwnedPrograms` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[0, 25]`.
	fn remove_program(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `326 + p * (32 ±0)`
		//  Estimated: `3809 + p * (31 ±0)`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(26_938_669, 0)
			.saturating_add(Weight::from_parts(0, 3809))
			// Standard Error: 47_904
			.saturating_add(Weight::from_parts(136_174, 0).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 31).saturating_mul(p.into()))
	}
}