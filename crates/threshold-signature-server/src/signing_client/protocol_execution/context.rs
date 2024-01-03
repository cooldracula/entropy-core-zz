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

use entropy_protocol::KeyParams;
use synedrion::{sessions::PrehashedMessage, KeyShare};

use crate::sign_init::SignInit;

/// Context for Signing Protocol execution.
#[derive(Debug, Clone)]
pub struct SignContext {
    /// Party context from block proposer
    pub sign_init: SignInit,
    /// Signing key share
    pub key_share: KeyShare<KeyParams>,
}

impl SignContext {
    pub fn new(sign_init: SignInit, key_share: KeyShare<KeyParams>) -> Self {
        Self { sign_init, key_share }
    }

    pub fn msg_to_sign(&self) -> &PrehashedMessage {
        &self.sign_init.signing_session_info.message_hash
    }
}
