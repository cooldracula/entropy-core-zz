#![allow(clippy::all)]

//! Autogenerated weights for `pallet_relayer`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-10-30, STEPS: `5`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Jesses-MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet=pallet_relayer
// --extrinsic=*
// --steps=5
// --repeat=2
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
	/// Storage: `Relayer::Registered` (r:1 w:0)
	/// Proof: `Relayer::Registered` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Registering` (r:1 w:1)
	/// Proof: `Relayer::Registering` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Relayer::Dkg` (r:1 w:1)
	/// Proof: `Relayer::Dkg` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[0, 1000000]`.
	fn register(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `133`
		//  Estimated: `3598`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(15_700_000, 0)
			.saturating_add(Weight::from_parts(0, 3598))
			// Standard Error: 58
			.saturating_add(Weight::from_parts(600, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
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
		//  Measured:  `16499`
		//  Estimated: `19964`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(25_833_333, 0)
			.saturating_add(Weight::from_parts(0, 19964))
			// Standard Error: 721_687
			.saturating_add(Weight::from_parts(500_000, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
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
		//  Measured:  `16501`
		//  Estimated: `19966`
		// Minimum execution time: 24_000_000 picoseconds.
		Weight::from_parts(24_583_333, 0)
			.saturating_add(Weight::from_parts(0, 19966))
			// Standard Error: 954_703
			.saturating_add(Weight::from_parts(1_250_000, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
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
	fn confirm_register_registered(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `16500`
		//  Estimated: `19965`
		// Minimum execution time: 29_000_000 picoseconds.
		Weight::from_parts(29_833_333, 0)
			.saturating_add(Weight::from_parts(0, 19965))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
