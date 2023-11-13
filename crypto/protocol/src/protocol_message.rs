use std::str;

use serde::{Deserialize, Serialize};
use synedrion::sessions::SignedMessage;

use crate::{
    execute_protocol::SignatureWrapper, protocol_transport::errors::ProtocolMessageErr, PartyId,
};

/// A Message send during the signing or DKG protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ProtocolMessage {
    /// Identifier of the author of this message
    pub from: PartyId,
    /// If `None`, it's a broadcast message sent to all parties
    pub to: Option<PartyId>,
    /// The signed protocol message
    pub payload: SignedMessage<SignatureWrapper>,
}

impl TryFrom<&[u8]> for ProtocolMessage {
    type Error = ProtocolMessageErr;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let parsed_msg: ProtocolMessage = bincode::deserialize(value)?;
        Ok(parsed_msg)
    }
}

impl ProtocolMessage {
    pub(crate) fn new_bcast(from: &PartyId, payload: SignedMessage<SignatureWrapper>) -> Self {
        Self { from: from.clone(), to: None, payload }
    }

    pub(crate) fn new_p2p(
        from: &PartyId,
        to: &PartyId,
        payload: SignedMessage<SignatureWrapper>,
    ) -> Self {
        Self { from: from.clone(), to: Some(to.clone()), payload }
    }
}
