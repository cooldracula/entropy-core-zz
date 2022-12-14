#![cfg_attr(not(feature = "std"), no_std)]
//! Types that is shared by clients and substrate nodes,
//! i.e. messages sent from one to the other and structs contained in those messages
//!
//! This helps ensures those structs are synced among clients and nodes.

pub mod types;
use codec::alloc::vec::Vec;

pub type RegistrationMessage = types::RegistrationMessage;
pub type RegistrationResponse = types::RegistrationResponse;
pub type SigRequest = types::SigRequest;
pub type SigResponse = types::SigResponse;
pub type OCWMessage = Vec<types::Message>;
pub type Message = types::Message;

pub const SIGNING_PARTY_SIZE: usize = 2;

// min balance 12 decimal chain = 0.1
pub const MIN_BALANCE: u128 = 10000000000;
