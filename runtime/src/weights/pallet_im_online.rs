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

//! Autogenerated weights for `pallet_im_online`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 33.0.0
//! DATE: 2024-05-01, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-172-31-39-223.us-east-2.compute.internal`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet=pallet_im_online
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

/// Weight functions for `pallet_im_online`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_im_online::WeightInfo for WeightInfo<T> {
	/// Storage: `Session::Validators` (r:1 w:0)
	/// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Session::CurrentIndex` (r:1 w:0)
	/// Proof: `Session::CurrentIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ImOnline::Keys` (r:1 w:0)
	/// Proof: `ImOnline::Keys` (`max_values`: Some(1), `max_size`: Some(320002), added: 320497, mode: `MaxEncodedLen`)
	/// Storage: `ImOnline::ReceivedHeartbeats` (r:1 w:1)
	/// Proof: `ImOnline::ReceivedHeartbeats` (`max_values`: None, `max_size`: Some(25), added: 2500, mode: `MaxEncodedLen`)
	/// Storage: `ImOnline::AuthoredBlocks` (r:1 w:0)
	/// Proof: `ImOnline::AuthoredBlocks` (`max_values`: None, `max_size`: Some(56), added: 2531, mode: `MaxEncodedLen`)
	/// The range of component `k` is `[1, 1000]`.
	fn validate_unsigned_and_then_heartbeat(k: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `460 + k * (32 ±0)`
		//  Estimated: `321487 + k * (1761 ±0)`
		// Minimum execution time: 74_969_000 picoseconds.
		Weight::from_parts(92_709_666, 0)
			.saturating_add(Weight::from_parts(0, 321487))
			// Standard Error: 517
			.saturating_add(Weight::from_parts(38_911, 0).saturating_mul(k.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 1761).saturating_mul(k.into()))
	}
}