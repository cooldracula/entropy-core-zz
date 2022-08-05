// #![cfg_attr(not(feature = "std"), no_std)]
/// Code that is shared by clients and substrate nodes,
/// i.e. messages sent from one to the other and structs contained in those messages
///
/// This helps ensures those structs are synced among clients and nodes.
///
/// I think this needs to be separate because this is the only conde that we can include in the
/// substrate code. Client code includes extrinsic calls and those can not be included in nodes due
/// to wasm.
// use codec::{Decode, Encode};
// use scale_info::TypeInfo;
mod common;
mod cm_info;
use codec::alloc::vec::Vec;

pub type RegistrationMessage = common::RegistrationMessage;
pub type RegistrationResponse = common::RegistrationResponse;
pub type SigRequest = common::SigRequest;
pub type SigResponse = common::SigResponse;
pub type OCWMessage = Vec<common::Message>;
pub type Message = common::Message;

// crypto
pub type CMInfo = cm_info::CMInfo;
pub type CMInfoUnchecked = cm_info::CMInfoUnchecked;
pub type KvKeyshareInfo = cm_info::KvKeyshareInfo;
pub type PartyUid = usize;
