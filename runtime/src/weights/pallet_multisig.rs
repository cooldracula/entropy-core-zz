#![allow(clippy::all)]

//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-30, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `hcastano`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet=pallet_multisig
// --extrinsic=*
// --steps=50
// --repeat=20
// --header=.maintain/ignore_clippy_header.txt
// --template
// .maintain/frame-weight-template.hbs
// --output=./runtime/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_000_000 picoseconds.
		Weight::from_parts(10_083_884, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 8
			.saturating_add(Weight::from_parts(298, 0).saturating_mul(z.into()))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `329 + s * (2 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 35_000_000 picoseconds.
		Weight::from_parts(30_081_908, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 3_581
			.saturating_add(Weight::from_parts(81_133, 0).saturating_mul(s.into()))
			// Standard Error: 35
			.saturating_add(Weight::from_parts(1_293, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `348`
		//  Estimated: `6811`
		// Minimum execution time: 21_000_000 picoseconds.
		Weight::from_parts(16_169_516, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 2_375
			.saturating_add(Weight::from_parts(70_824, 0).saturating_mul(s.into()))
			// Standard Error: 23
			.saturating_add(Weight::from_parts(1_283, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `456 + s * (33 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 39_000_000 picoseconds.
		Weight::from_parts(28_717_896, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 4_418
			.saturating_add(Weight::from_parts(129_394, 0).saturating_mul(s.into()))
			// Standard Error: 43
			.saturating_add(Weight::from_parts(1_620, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `337 + s * (1 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 29_000_000 picoseconds.
		Weight::from_parts(29_696_326, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 3_395
			.saturating_add(Weight::from_parts(83_130, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `348`
		//  Estimated: `6811`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_375_346, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 1_293
			.saturating_add(Weight::from_parts(59_620, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `525 + s * (1 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 29_000_000 picoseconds.
		Weight::from_parts(28_602_938, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 2_572
			.saturating_add(Weight::from_parts(105_118, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}