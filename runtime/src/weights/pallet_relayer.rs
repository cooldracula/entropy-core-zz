
//! Autogenerated weights for `pallet_relayer`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-07-07, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// --chain=dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_relayer
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_relayer`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_relayer::WeightInfo for WeightInfo<T> {
	// Storage: Relayer Messages (r:1 w:1)
	fn prep_transaction() -> Weight {
		(34_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Relayer Registered (r:0 w:1)
	fn register() -> Weight {
		(21_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Relayer Messages (r:1 w:0)
	// Storage: Relayer Failures (r:1 w:0)
	// Storage: Relayer Responsibility (r:1 w:0)
	// Storage: Relayer Unresponsive (r:1 w:1)
	// Storage: Authorship Author (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: Relayer Pending (r:0 w:1)
	fn move_active_to_pending_no_failure(m: u32, ) -> Weight {
		(31_355_000 as Weight)
			// Standard Error: 23_000
			.saturating_add((625_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Relayer Messages (r:1 w:0)
	// Storage: Relayer Failures (r:1 w:0)
	// Storage: Relayer Responsibility (r:1 w:0)
	// Storage: Relayer Unresponsive (r:1 w:1)
	// Storage: Authorship Author (r:1 w:0)
	// Storage: System Digest (r:1 w:0)
	// Storage: Relayer Pending (r:0 w:1)
	fn move_active_to_pending_failure(_m: u32, ) -> Weight {
		(38_350_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
