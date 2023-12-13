#![allow(clippy::all)]

//! Autogenerated weights for `pallet_recovery`
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
// --pallet=pallet_recovery
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

/// Weight functions for `pallet_recovery`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_recovery::WeightInfo for WeightInfo<T> {
	/// Storage: `Recovery::Proxy` (r:1 w:0)
	/// Proof: `Recovery::Proxy` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	fn as_recovered() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `182`
		//  Estimated: `3545`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3545))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Recovery::Proxy` (r:0 w:1)
	/// Proof: `Recovery::Proxy` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	fn set_recovered() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Recovery::Recoverable` (r:1 w:1)
	/// Proof: `Recovery::Recoverable` (`max_values`: None, `max_size`: Some(351), added: 2826, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 9]`.
	fn create_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `3816`
		// Minimum execution time: 24_000_000 picoseconds.
		Weight::from_parts(24_495_556, 0)
			.saturating_add(Weight::from_parts(0, 3816))
			// Standard Error: 6_635
			.saturating_add(Weight::from_parts(142_575, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Recovery::Recoverable` (r:1 w:0)
	/// Proof: `Recovery::Recoverable` (`max_values`: None, `max_size`: Some(351), added: 2826, mode: `MaxEncodedLen`)
	/// Storage: `Recovery::ActiveRecoveries` (r:1 w:1)
	/// Proof: `Recovery::ActiveRecoveries` (`max_values`: None, `max_size`: Some(389), added: 2864, mode: `MaxEncodedLen`)
	fn initiate_recovery() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `173`
		//  Estimated: `3854`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(28_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3854))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Recovery::Recoverable` (r:1 w:0)
	/// Proof: `Recovery::Recoverable` (`max_values`: None, `max_size`: Some(351), added: 2826, mode: `MaxEncodedLen`)
	/// Storage: `Recovery::ActiveRecoveries` (r:1 w:1)
	/// Proof: `Recovery::ActiveRecoveries` (`max_values`: None, `max_size`: Some(389), added: 2864, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 9]`.
	fn vouch_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `261 + n * (64 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 15_000_000 picoseconds.
		Weight::from_parts(16_751_701, 0)
			.saturating_add(Weight::from_parts(0, 3854))
			// Standard Error: 8_228
			.saturating_add(Weight::from_parts(81_482, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Recovery::Recoverable` (r:1 w:0)
	/// Proof: `Recovery::Recoverable` (`max_values`: None, `max_size`: Some(351), added: 2826, mode: `MaxEncodedLen`)
	/// Storage: `Recovery::ActiveRecoveries` (r:1 w:0)
	/// Proof: `Recovery::ActiveRecoveries` (`max_values`: None, `max_size`: Some(389), added: 2864, mode: `MaxEncodedLen`)
	/// Storage: `Recovery::Proxy` (r:1 w:1)
	/// Proof: `Recovery::Proxy` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 9]`.
	fn claim_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `293 + n * (64 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 19_000_000 picoseconds.
		Weight::from_parts(20_578_332, 0)
			.saturating_add(Weight::from_parts(0, 3854))
			// Standard Error: 10_800
			.saturating_add(Weight::from_parts(18_068, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Recovery::ActiveRecoveries` (r:1 w:1)
	/// Proof: `Recovery::ActiveRecoveries` (`max_values`: None, `max_size`: Some(389), added: 2864, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 9]`.
	fn close_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `447 + n * (32 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 31_000_000 picoseconds.
		Weight::from_parts(32_158_858, 0)
			.saturating_add(Weight::from_parts(0, 3854))
			// Standard Error: 13_134
			.saturating_add(Weight::from_parts(106_889, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Recovery::ActiveRecoveries` (r:1 w:0)
	/// Proof: `Recovery::ActiveRecoveries` (`max_values`: None, `max_size`: Some(389), added: 2864, mode: `MaxEncodedLen`)
	/// Storage: `Recovery::Recoverable` (r:1 w:1)
	/// Proof: `Recovery::Recoverable` (`max_values`: None, `max_size`: Some(351), added: 2826, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 9]`.
	fn remove_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `170 + n * (32 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 28_000_000 picoseconds.
		Weight::from_parts(28_866_310, 0)
			.saturating_add(Weight::from_parts(0, 3854))
			// Standard Error: 12_971
			.saturating_add(Weight::from_parts(15_860, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Recovery::Proxy` (r:1 w:1)
	/// Proof: `Recovery::Proxy` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	fn cancel_recovered() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `182`
		//  Estimated: `3545`
		// Minimum execution time: 10_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3545))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}