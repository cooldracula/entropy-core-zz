#![allow(clippy::all)]

//! Autogenerated weights for `pallet_transaction_storage`
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
// --pallet=pallet_transaction_storage
// --extrinsic=*
// --steps=50
// --repeat=20
// --header=./file_header.txt
// --template
// .maintain/frame-weight-template.hbs
// --output=./runtime/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_transaction_storage`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_transaction_storage::WeightInfo for WeightInfo<T> {
	/// Storage: `TransactionStorage::ByteFee` (r:1 w:0)
	/// Proof: `TransactionStorage::ByteFee` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::EntryFee` (r:1 w:0)
	/// Proof: `TransactionStorage::EntryFee` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::BlockTransactions` (r:1 w:1)
	/// Proof: `TransactionStorage::BlockTransactions` (`max_values`: Some(1), `max_size`: Some(36866), added: 37361, mode: `MaxEncodedLen`)
	/// The range of component `l` is `[1, 8388608]`.
	fn store(l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `142`
		//  Estimated: `38351`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(32_000_000, 0)
			.saturating_add(Weight::from_parts(0, 38351))
			// Standard Error: 2
			.saturating_add(Weight::from_parts(5_071, 0).saturating_mul(l.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `TransactionStorage::Transactions` (r:1 w:0)
	/// Proof: `TransactionStorage::Transactions` (`max_values`: None, `max_size`: Some(36886), added: 39361, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::ByteFee` (r:1 w:0)
	/// Proof: `TransactionStorage::ByteFee` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::EntryFee` (r:1 w:0)
	/// Proof: `TransactionStorage::EntryFee` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::BlockTransactions` (r:1 w:1)
	/// Proof: `TransactionStorage::BlockTransactions` (`max_values`: Some(1), `max_size`: Some(36866), added: 37361, mode: `MaxEncodedLen`)
	fn renew() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `292`
		//  Estimated: `40351`
		// Minimum execution time: 42_000_000 picoseconds.
		Weight::from_parts(44_000_000, 0)
			.saturating_add(Weight::from_parts(0, 40351))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `TransactionStorage::ProofChecked` (r:1 w:1)
	/// Proof: `TransactionStorage::ProofChecked` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::StoragePeriod` (r:1 w:0)
	/// Proof: `TransactionStorage::StoragePeriod` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::ChunkCount` (r:1 w:0)
	/// Proof: `TransactionStorage::ChunkCount` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `System::ParentHash` (r:1 w:0)
	/// Proof: `System::ParentHash` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `TransactionStorage::Transactions` (r:1 w:0)
	/// Proof: `TransactionStorage::Transactions` (`max_values`: None, `max_size`: Some(36886), added: 39361, mode: `MaxEncodedLen`)
	fn check_proof_max() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37111`
		//  Estimated: `40351`
		// Minimum execution time: 73_000_000 picoseconds.
		Weight::from_parts(126_000_000, 0)
			.saturating_add(Weight::from_parts(0, 40351))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}