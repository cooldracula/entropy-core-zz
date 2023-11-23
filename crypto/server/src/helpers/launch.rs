//! Utilities for starting and running the server.

use std::{fs, path::PathBuf};

use bip39::{Language, Mnemonic, MnemonicType};
use clap::Parser;
use kvdb::{
    encrypted_sled::PasswordMethod,
    kv_manager::{error::KvError, KvManager},
};
use serde::Deserialize;
use subxt::ext::sp_core::{crypto::AccountId32, sr25519, Pair};

use crate::validation::{derive_static_secret, mnemonic_to_pair};

pub const DEFAULT_MNEMONIC: &str =
    "alarm mutual concert decrease hurry invest culture survey diagram crash snap click";
pub const DEFAULT_BOB_MNEMONIC: &str =
    "where sight patient orphan general short empower hope party hurt month voice";
pub const DEFAULT_ALICE_MNEMONIC: &str =
    "alarm mutual concert decrease hurry invest culture survey diagram crash snap click";
pub const DEFAULT_CHARLIE_MNEMONIC: &str =
    "lake carry still awful point mention bike category tornado plate brass lock";

pub const LATEST_BLOCK_NUMBER_NEW_USER: &str = "LATEST_BLOCK_NUMBER_NEW_USER";
pub const LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH: &str = "LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH";

#[cfg(test)]
pub const DEFAULT_ENDPOINT: &str = "ws://localhost:9944";

pub const FORBIDDEN_KEYS: [&str; 3] = ["MNEMONIC", "SHARED_SECRET", "DH_PUBLIC"];

/// Initializes the tracing `Subscriber` for the process.
pub fn init_tracing() {
    // We set up the logger to only print out logs of `ERROR` or higher by default, otherwise
    // we fall back to the user's `RUST_LOG` settings.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
}

// Deafult name for TSS server
// Will set mnemonic and db path
#[derive(Debug, PartialEq)]
pub enum ValidatorName {
    Alice,
    Bob,
    Charlie,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub endpoint: String,
}

impl Configuration {
    pub fn new(endpoint: String) -> Configuration {
        Configuration { endpoint }
    }
}

pub async fn load_kv_store(validator_name: &Option<ValidatorName>, no_password: bool) -> KvManager {
    let mut root: PathBuf = PathBuf::from(kvdb::get_db_path(false));
    if cfg!(test) {
        return KvManager::new(
            kvdb::get_db_path(true).into(),
            PasswordMethod::NoPassword.execute().unwrap(),
        )
        .unwrap();
    }
    if validator_name == &Some(ValidatorName::Bob) {
        root.push("bob");
        return KvManager::new(root, PasswordMethod::NoPassword.execute().unwrap()).unwrap();
    };
    if validator_name == &Some(ValidatorName::Alice) {
        return KvManager::new(root, PasswordMethod::NoPassword.execute().unwrap()).unwrap();
    };
    // TODO remove and force password
    if no_password {
        return KvManager::new(root, PasswordMethod::NoPassword.execute().unwrap()).unwrap();
    }
    let password = PasswordMethod::Prompt.execute().unwrap();
    // this step takes a long time due to password-based decryption
    KvManager::new(root, password).unwrap()
}

#[derive(Parser, Debug, Clone)]
pub struct StartupArgs {
    /// Wether to sync the keystore.
    #[arg(short = 's', long = "sync")]
    pub sync: bool,
    /// Use the developer key Bob.
    #[arg(short = 'b', long = "bob")]
    pub bob: bool,
    /// Use the developer key Alice.
    #[arg(short = 'a', long = "alice")]
    pub alice: bool,
    /// Websocket endpoint for the entropy blockchain.
    #[arg(
        short = 'c',
        long = "chain-endpoint",
        required = false,
        default_value = "ws://localhost:9944"
    )]
    pub chain_endpoint: String,
    /// Url to host threshold (axum) server on.
    #[arg(short = 'u', long = "threshold-url", required = false, default_value = "127.0.0.1:3001")]
    pub threshold_url: String,

    /// Wether to allow a validator key to be null.
    #[arg(short = 'd', long = "dev")]
    pub dev: bool,

    /// Whether or not to execute a specific test
    #[cfg(test)]
    #[arg(long = "test", required = false, default_value = "*")]
    pub test: String,

    /// Whether or not to print stdout during testing
    #[arg(long = "nocapture")]
    pub nocapture: bool,

    /// TODO remove and force password
    #[arg(long = "nopassword")]
    pub no_password: bool,
}

pub async fn setup_mnemonic(
    kv: &KvManager,
    validator_name: &Option<ValidatorName>,
) -> Result<(), KvError> {
    // Check if a mnemonic exists in the kvdb.
    let exists_result = kv.kv().exists(FORBIDDEN_KEYS[0]).await.expect("issue querying DB");
    if !exists_result {
        // Generate a new mnemonic
        let mut mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
        // If using a test configuration then set to the default mnemonic.
        if cfg!(test) {
            mnemonic = Mnemonic::from_phrase(DEFAULT_MNEMONIC, Language::English)
                .expect("Issue creating Mnemonic");
        }
        if validator_name == &Some(ValidatorName::Alice) {
            mnemonic = Mnemonic::from_phrase(DEFAULT_ALICE_MNEMONIC, Language::English)
                .expect("Issue creating Mnemonic");
        }
        if validator_name == &Some(ValidatorName::Bob) {
            mnemonic = Mnemonic::from_phrase(DEFAULT_BOB_MNEMONIC, Language::English)
                .expect("Issue creating Mnemonic");
        }
        if validator_name == &Some(ValidatorName::Charlie) {
            mnemonic = Mnemonic::from_phrase(DEFAULT_CHARLIE_MNEMONIC, Language::English)
                .expect("Issue creating Mnemonic");
        }

        let phrase = mnemonic.phrase();
        let pair = mnemonic_to_pair(&mnemonic).expect("Issue deriving Mnemonic");
        let static_secret = derive_static_secret(&pair);
        let dh_public = x25519_dalek::PublicKey::from(&static_secret);

        let ss_reservation = kv
            .kv()
            .reserve_key(FORBIDDEN_KEYS[1].to_string())
            .await
            .expect("Issue reserving ss key");
        kv.kv()
            .put(ss_reservation, static_secret.to_bytes().to_vec())
            .await
            .expect("failed to update secret share");

        let dh_reservation = kv
            .kv()
            .reserve_key(FORBIDDEN_KEYS[2].to_string())
            .await
            .expect("Issue reserving DH key");

        let converted_dh_public = dh_public.to_bytes().to_vec();
        kv.kv()
            .put(dh_reservation, converted_dh_public.clone())
            .await
            .expect("failed to update dh");

        let formatted_dh_public = format!("{converted_dh_public:?}").replace('"', "");
        fs::write(".entropy/public_key", formatted_dh_public)
            .expect("Failed to write public key file");

        let p = <sr25519::Pair as Pair>::from_phrase(phrase, None)
            .expect("Issue getting pair from mnemonic");
        let id = AccountId32::new(p.0.public().0);

        fs::write(".entropy/account_id", format!("{id}")).expect("Failed to write account_id file");

        // Update the value in the kvdb
        let reservation = kv
            .kv()
            .reserve_key(FORBIDDEN_KEYS[0].to_string())
            .await
            .expect("Issue reserving mnemonic");
        kv.kv()
            .put(reservation, phrase.as_bytes().to_vec())
            .await
            .expect("failed to update mnemonic");

        tracing::debug!("Starting process with account ID: `{id}`");
    }
    Ok(())
}

pub async fn setup_latest_block_number(kv: &KvManager) -> Result<(), KvError> {
    let exists_result_new_user =
        kv.kv().exists(LATEST_BLOCK_NUMBER_NEW_USER).await.expect("issue querying DB");
    if !exists_result_new_user {
        let reservation = kv
            .kv()
            .reserve_key(LATEST_BLOCK_NUMBER_NEW_USER.to_string())
            .await
            .expect("Issue reserving latest block number");
        kv.kv()
            .put(reservation, 0u32.to_be_bytes().to_vec())
            .await
            .expect("failed to update latest block number");
    }
    let exists_result_proactive_refresh =
        kv.kv().exists(LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH).await.expect("issue querying DB");
    if !exists_result_proactive_refresh {
        let reservation = kv
            .kv()
            .reserve_key(LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH.to_string())
            .await
            .expect("Issue reserving latest block number");
        kv.kv()
            .put(reservation, 0u32.to_be_bytes().to_vec())
            .await
            .expect("failed to update latest block number");
    }
    Ok(())
}
