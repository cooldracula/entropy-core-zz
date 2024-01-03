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

#![allow(dead_code)]

use std::collections::HashMap;

use entropy_protocol::{
    protocol_transport::{Broadcaster, WsChannels},
    ProtocolMessage, ValidatorInfo,
};
use entropy_shared::X25519PublicKey;
use subxt::utils::AccountId32;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::signing_client::SubscribeErr;

pub type ListenerResult = Result<Broadcaster, SubscribeErr>;

/// Tracks which validators we are connected to for a particular protocol execution
/// and sets up channels for exchaning protocol messages
#[derive(Debug)]
pub struct Listener {
    /// Endpoint to create subscriptions
    tx: broadcast::Sender<ProtocolMessage>,
    /// Messages
    tx_to_others: mpsc::Sender<ProtocolMessage>,
    /// Endpoint to notify protocol execution ready-for-signing
    tx_ready: oneshot::Sender<ListenerResult>,
    /// Remaining validators we want to connect to
    // Key is subxt AccountId32 but it doesn't implement Hash so we use [u8; 32]
    pub validators: HashMap<[u8; 32], X25519PublicKey>,
}

impl Listener {
    pub(crate) fn new(
        validators_info: Vec<ValidatorInfo>,
        my_id: &AccountId32,
        user_participates: Option<(AccountId32, X25519PublicKey)>,
    ) -> (oneshot::Receiver<ListenerResult>, mpsc::Receiver<ProtocolMessage>, Self) {
        let (tx_ready, rx_ready) = oneshot::channel();
        let (tx, _rx) = broadcast::channel(1000);
        let (tx_to_others, rx_to_others) = mpsc::channel(1000);

        // Create our set of validators we want too connect to - excluding ourself
        let mut validators = HashMap::new();

        for validator in validators_info {
            if &validator.tss_account != my_id {
                validators.insert(validator.tss_account.0, validator.x25519_public_key);
            }
        }

        // If visibility is private, also expect the user to connect
        if let Some((user_id, user_x25519_pk)) = user_participates {
            validators.insert(user_id.0, user_x25519_pk);
        }

        {
            (rx_ready, rx_to_others, Self { tx, tx_to_others, tx_ready, validators })
        }
    }

    /// Check that the given account is in the signing group, and if so return channels to the
    /// protocol
    pub(crate) fn subscribe(
        &mut self,
        account_id: &AccountId32,
    ) -> Result<WsChannels, SubscribeErr> {
        if self.validators.remove(&account_id.0).is_some() {
            let broadcast = self.tx.subscribe();
            let tx = self.tx_to_others.clone();
            Ok(WsChannels { broadcast, tx, is_final: self.validators.is_empty() })
        } else {
            Err(SubscribeErr::InvalidPartyId(
                "Validator is not expected for this message".to_string(),
            ))
        }
    }

    /// When all connections are set up, convert to a broadcaster and proceed with the protocol
    pub(crate) fn into_broadcaster(self) -> (oneshot::Sender<ListenerResult>, Broadcaster) {
        (self.tx_ready, Broadcaster(self.tx))
    }
}
