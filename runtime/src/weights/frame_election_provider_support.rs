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

//! Autogenerated weights for `frame_election_provider_support`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 33.0.0
//! DATE: 2024-06-26, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-172-31-30-92`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/entropy
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet=frame_election_provider_support
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

/// Weight functions for `frame_election_provider_support`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_election_provider_support::WeightInfo for WeightInfo<T> {
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `d` is `[5, 16]`.
	fn phragmen(v: u32, _t: u32, d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_649_340_000 picoseconds.
		Weight::from_parts(9_769_807_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 197_459
			.saturating_add(Weight::from_parts(8_442_336, 0).saturating_mul(v.into()))
			// Standard Error: 20_187_531
			.saturating_add(Weight::from_parts(2_001_376_701, 0).saturating_mul(d.into()))
	}
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `d` is `[5, 16]`.
	fn phragmms(v: u32, _t: u32, d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_199_091_000 picoseconds.
		Weight::from_parts(6_238_219_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 160_928
			.saturating_add(Weight::from_parts(6_428_846, 0).saturating_mul(v.into()))
			// Standard Error: 16_452_756
			.saturating_add(Weight::from_parts(1_817_091_514, 0).saturating_mul(d.into()))
	}
}