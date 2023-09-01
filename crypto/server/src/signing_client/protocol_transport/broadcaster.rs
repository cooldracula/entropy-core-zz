//! Listener becomes Broadcaster when all other parties have subscribed.

use tokio::sync::broadcast::{self, error::SendError};

use crate::signing_client::ProtocolMessage;

#[derive(Debug)]
pub struct Broadcaster(pub broadcast::Sender<ProtocolMessage>);

impl Broadcaster {
    pub fn send(&self, msg: ProtocolMessage) -> Result<usize, Box<SendError<ProtocolMessage>>> {
        Ok(self.0.send(msg)?)
    }
}
