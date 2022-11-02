
//! Autogenerated weights for `pallet_free_tx`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-12, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_free_tx
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// ./weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};
use sp_std::marker::PhantomData;


pub trait WeightInfo {
	fn call_using_electricity() -> Weight;
	fn set_individual_electricity_era_limit() -> Weight;
	fn set_battery_count() -> Weight;
	fn give_zaps() -> Weight;
}

/// Weight functions for `pallet_free_tx`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: FreeTx FreeCallsLeft (r:1 w:1)
	fn call_using_electricity() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn set_individual_electricity_era_limit() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn set_battery_count() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn give_zaps() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

}

// For backwards compatibility and tests

impl WeightInfo for () {
	// Storage: FreeTx FreeCallsLeft (r:1 w:1)
	fn call_using_electricity() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn set_individual_electricity_era_limit() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn set_battery_count() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn give_zaps() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}