#![allow(clippy::all)]

//! Autogenerated weights for `pallet_free_tx`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-12, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MacBook-Pro-9.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_free_tx
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

/// Weight functions for `pallet_free_tx`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_free_tx::WeightInfo for WeightInfo<T> {
	/// Storage: FreeTx MaxUserElectricityUsagePerEra (r:1 w:0)
	/// Proof: FreeTx MaxUserElectricityUsagePerEra (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: FreeTx ElectricalAccount (r:1 w:1)
	/// Proof: FreeTx ElectricalAccount (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: Staking CurrentEra (r:1 w:0)
	/// Proof: Staking CurrentEra (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn call_using_electricity() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `715`
		//  Estimated: `3529`
		// Minimum execution time: 43_000_000 picoseconds.
		Weight::from_parts(43_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3529))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: FreeTx MaxUserElectricityUsagePerEra (r:0 w:1)
	/// Proof: FreeTx MaxUserElectricityUsagePerEra (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn set_individual_electricity_era_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: FreeTx ElectricalAccount (r:1 w:1)
	/// Proof: FreeTx ElectricalAccount (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	fn set_battery_count() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `3529`
		// Minimum execution time: 12_000_000 picoseconds.
		Weight::from_parts(13_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3529))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: FreeTx ElectricalAccount (r:1 w:1)
	/// Proof: FreeTx ElectricalAccount (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	fn give_zaps() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `3529`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3529))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
