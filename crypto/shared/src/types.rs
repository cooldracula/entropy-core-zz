#![allow(dead_code)]
use codec::{alloc::vec::Vec, Decode, Encode};
use scale_info::TypeInfo;

/// common structs etc, shared among the substrate-blockchain-code and the crypto-code
pub use crate::constraints::*;

/// body of a signature generation request by the user to the entropy network
#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, TypeInfo)]
pub struct SigRequest {
    /// sig_hash. this is a hash of the message to be signed
    pub sig_hash: codec::alloc::vec::Vec<u8>,
}

/// The message sent from pallets::propagation::post() to the signing-client.
// TODO(TK): rename to PropagationMessage
#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, TypeInfo)]
pub struct Message {
    pub sig_request: SigRequest,
    pub account: codec::alloc::vec::Vec<u8>,
    pub ip_addresses: codec::alloc::vec::Vec<codec::alloc::vec::Vec<u8>>,
}

pub type OCWMessage = Vec<Message>;
