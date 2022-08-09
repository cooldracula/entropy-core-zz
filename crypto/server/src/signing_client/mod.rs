pub mod api;
mod context;
mod errors;
mod protocol_manager;
mod subscriber;

use std::{collections::HashMap, sync::Mutex};

pub use errors::SigningProtocolError;
use kvdb::{encrypted_sled::PasswordMethod, kv_manager::KvManager};
pub use protocol_manager::{ProtocolManager, SigningMessage};
pub use subscriber::{SubscriberManager, SubscribingMessage};

use crate::PartyUid;

/// The state used by this node to create signatures
pub struct SignerState {
	/// Mapping of PartyIds to `SubscriberManager`s, one entry per active party.
	// TODO(TK): SubscriberManager to be replaced with None when subscribing phase ends.
	subscriber_manager_map: Mutex<HashMap<PartyUid, Option<SubscriberManager>>>,
	/// All shares stored by this node, see: StoredInfo (name is WIP)
	kv_manager: KvManager,
}

impl Default for SignerState {
	fn default() -> Self {
		Self { subscriber_manager_map: Mutex::default(), kv_manager: load_kv_store() }
	}
}

// exclude kv manager
impl std::fmt::Debug for SignerState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Global")
			.field("subscriber_manager_map", &self.subscriber_manager_map)
			.finish()
	}
}

pub fn load_kv_store() -> KvManager {
	if cfg!(test) {
		KvManager::new(kvdb::get_db_path().into(), PasswordMethod::NoPassword.execute().unwrap())
			.unwrap()
	} else {
		let root = project_root::get_project_root().unwrap();
		let password = PasswordMethod::Prompt.execute().unwrap();
		// this step takes a long time due to password-based decryption
		KvManager::new(root, password).unwrap()
	}
}
