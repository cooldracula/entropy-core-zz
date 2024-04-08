// Copyright (C) 2023 Entropy Cryptography Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for `pallet_transaction_storage`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 29.0.0
//! DATE: 2024-04-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --header=.maintain/AGPL-3.0-header.txt
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
		//  Measured:  `105`
		//  Estimated: `38351`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(33_000_000, 0)
			.saturating_add(Weight::from_parts(0, 38351))
			// Standard Error: 4
			.saturating_add(Weight::from_parts(6_621, 0).saturating_mul(l.into()))
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
		//  Measured:  `255`
		//  Estimated: `40351`
		// Minimum execution time: 48_000_000 picoseconds.
		Weight::from_parts(52_000_000, 0)
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
		//  Measured:  `37074`
		//  Estimated: `40351`
		// Minimum execution time: 79_000_000 picoseconds.
		Weight::from_parts(90_000_000, 0)
			.saturating_add(Weight::from_parts(0, 40351))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}