//! Communicate with other threshold servers and carry out the signing protocol
pub mod api;
mod errors;
pub(crate) mod new_party;
pub(crate) mod protocol_transport;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use self::{
    errors::*,
    new_party::SigningMessage,
    protocol_transport::{Listener, SubscribeMessage},
};

/// The state used by this node to create signatures
#[derive(Default, Debug, Clone)]
pub struct SignerState {
    /// Mapping of PartyIds to `SubscriberManager`s, one entry per active party.
    // TODO(TK): SubscriberManager to be replaced with None when subscribing phase ends.
    pub listeners: Arc<Mutex<HashMap<String, Listener>>>,
}

impl SignerState {
    /// Create a new `SignerState`
    pub fn contains_listener(&self, session_id: &String) -> Result<bool, SubscribeErr> {
        Ok(self
            .listeners
            .lock()
            .map_err(|e| SubscribeErr::LockError(e.to_string()))?
            .contains_key(session_id))
    }
}
