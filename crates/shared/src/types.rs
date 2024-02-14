// Copyright (C) 2023 Entropy Cryptography Inc.
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

#![allow(dead_code)]
#[cfg(not(feature = "wasm"))]
use codec::alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm-no-std")]
use sp_runtime::RuntimeDebug;
#[cfg(feature = "std")]
use strum_macros::EnumIter;

/// X25519 public key used by the client in non-interactive ECDH to authenticate/encrypt
/// interactions with the threshold server (eg distributing threshold shares).
pub type X25519PublicKey = [u8; 32];

/// This should match the type found in `entropy-runtime`. We define it ourselves manually here
/// since we don't want to pull that whole crate it just for a `u32`.
type BlockNumber = u32;

/// Defines an application's accessibility
/// Public -> Anyone can request a signature
/// Permissioned -> Only permissioned users can request a signature
/// Private -> Requires the keyshare holder to participate in the threshold signing process
#[cfg_attr(not(feature = "wasm-no-std"), derive(Debug))]
#[cfg_attr(feature = "wasm-no-std", derive(RuntimeDebug))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum KeyVisibility {
    Public,
    Permissioned,
    Private(X25519PublicKey),
}

/// Information from the validators in signing party
#[cfg(not(feature = "wasm"))]
#[derive(
    Clone,
    Encode,
    Decode,
    Debug,
    Eq,
    PartialEq,
    TypeInfo,
    sp_runtime::Serialize,
    sp_runtime::Deserialize,
)]
pub struct ValidatorInfo {
    pub x25519_public_key: X25519PublicKey,
    pub ip_address: codec::alloc::vec::Vec<u8>,
    pub tss_account: codec::alloc::vec::Vec<u8>,
}

/// Offchain worker message for initiating a dkg
#[cfg(not(feature = "wasm"))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, TypeInfo)]
pub struct OcwMessageDkg {
    pub block_number: BlockNumber,
    pub sig_request_accounts: Vec<Vec<u8>>,
    pub validators_info: Vec<ValidatorInfo>,
}

/// Offchain worker message for initiating a proactive refresh
#[cfg(not(feature = "wasm"))]
#[derive(
    Clone,
    Encode,
    Decode,
    Debug,
    Eq,
    PartialEq,
    TypeInfo,
    sp_runtime::Serialize,
    sp_runtime::Deserialize,
)]
pub struct OcwMessageProactiveRefresh {
    /// Information of the validators to participate
    pub validators_info: Vec<ValidatorInfo>,
    /// Accounts to take part in the proactive refresh
    pub proactive_refresh_keys: Vec<Vec<u8>>,
}

/// 256-bit hashing algorithms for deriving the point to be signed.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, EnumIter))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", serde(rename = "hash"))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum HashingAlgorithm {
    Sha1,
    Sha2,
    Sha3,
    Keccak,
    Custom(usize),
}
