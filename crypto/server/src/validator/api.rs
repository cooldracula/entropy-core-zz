use std::str::FromStr;

use hex_literal::hex;
use kvdb::kv_manager::{
    error::{InnerKvError, KvError},
    value::PartyInfo,
    KeyReservation, KvManager,
};
use parity_scale_codec::Decode;
use reqwest;
use rocket::{
    http::{ContentType, Status},
    response::{self, content, stream::EventStream, Responder, Response},
    serde::json::Json,
    Shutdown, State,
};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::AccountId32, sr25519, Pair, Public};
use subxt::{tx::PairSigner, OnlineClient};
use tokio::sync::{mpsc, oneshot};

use crate::{
    chain_api::{entropy, get_api, EntropyConfig},
    validator::errors::ValidatorErr,
    Configuration,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Keys {
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Values {
    pub values: Vec<Vec<u8>>,
}

/// Endpoint to allow a new node to sync their kvdb with a member of their subgroup
#[post("/sync_kvdb", format = "json", data = "<keys>")]
pub async fn sync_kvdb(
    keys: Json<Keys>,
    state: &State<KvManager>,
    config: &State<Configuration>,
) -> Json<Values> {
    // let api = get_api(&config.endpoint).await.unwrap();
    // validate on chain that this user in your subgroup
    // validate the message comes from individual
    // validate the message is intended for me

    // encrypt message and send to other validator
    let mut values = vec![];
    for key in keys.keys.clone() {
        let result = state.kv().get(&key).await.unwrap();
        values.push(result);
    }
    let values_json = Values { values };
    Json(values_json)
}

/// As a node is joining the network should get all keys that are registered
/// This is done by reading the registered mapping and getting all the keys of that mapping
pub async fn get_all_keys(
    api: &OnlineClient<EntropyConfig>,
    batch_size: usize,
) -> Result<Vec<String>, ValidatorErr> {
    // zero batch size will cause infinite loop, also not needed
    assert_ne!(batch_size, 0);
    let mut result_length = batch_size;
    let mut addresses: Vec<String> = vec![];
    while result_length == batch_size {
        result_length = 0;
        // query the registered mapping in the relayer pallet
        let storage_address = subxt::dynamic::storage_root("Relayer", "Registered");
        let mut iter = api.storage().iter(storage_address, batch_size as u32, None).await?;
        while let Some((key, account)) = iter.next().await? {
            let new_key = hex::encode(key);
            let len = new_key.len();
            let final_key = &new_key[len - 64..];

            let address: AccountId32 =
                AccountId32::from_str(final_key).expect("Account conversion error");

            // todo add validation
            // dbg!(address.to_string(), bool::decode(mut account));
            // if account.to_value()? {
            if addresses.contains(&address.to_string()) {
                result_length = 0;
            } else {
                addresses.push(address.to_string());
                result_length += 1;
            }
        }
    }
    Ok(addresses)
}

/// get a url of someone in your signing group
pub async fn get_key_url(
    api: &OnlineClient<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
    my_subgroup: u8,
) -> Result<String, ValidatorErr> {
    let signing_group_addresses_query =
        entropy::storage().staking_extension().signing_groups(my_subgroup);
    let signing_group_addresses = api
        .storage()
        .fetch(&signing_group_addresses_query, None)
        .await?
        .ok_or_else(|| ValidatorErr::OptionUnwrapError("Querying Signing Groups Error"))?;

    // TODO: Just gets first person in subgroup, maybe do this randomly?
    // find kvdb that isn't syncing and get their URL
    let mut server_sync_state = false;
    let mut server_to_query = 0;
    let mut server_info: Option<
        entropy::runtime_types::pallet_staking_extension::pallet::ServerInfo<
            sp_core::crypto::AccountId32,
        >,
    > = None;
    while !server_sync_state {
        let server_info_query = entropy::storage()
            .staking_extension()
            .threshold_servers(&signing_group_addresses[server_to_query]);
        server_info = Some(
            api.storage()
                .fetch(&server_info_query, None)
                .await?
                .ok_or_else(|| ValidatorErr::OptionUnwrapError("Server Info Fetch Error"))?,
        );
        let server_state_query = entropy::storage()
            .staking_extension()
            .is_validator_synced(&signing_group_addresses[server_to_query]);
        server_sync_state = api
            .storage()
            .fetch(&server_state_query, None)
            .await?
            .ok_or_else(|| ValidatorErr::OptionUnwrapError("Server State Fetch Error"))?;
        server_to_query += 1;
    }

    let ip_address = String::from_utf8(server_info.unwrap().endpoint)?;
    Ok(ip_address)
}

/// from keys of registered account get their corresponding entropy threshold keys
pub async fn get_and_store_values(
    all_keys: Vec<String>,
    kv: &KvManager,
    url: String,
    batch_size: usize,
    dev: bool,
) -> Result<(), ValidatorErr> {
    let mut keys_stored = 0;
    while keys_stored < all_keys.len() {
        let mut keys_to_send_slice = batch_size + keys_stored;
        if keys_to_send_slice > all_keys.len() {
            keys_to_send_slice = all_keys.len();
        }
        let keys_to_send = Keys { keys: all_keys[keys_stored..(keys_to_send_slice)].to_vec() };
        let client = reqwest::Client::new();
        let formatted_url = format!("http://{url}/validator/sync_kvdb");
        let result = client
            .post(formatted_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&keys_to_send)?)
            .send()
            .await?;

        if result.status() == 500 && dev {
            keys_stored += 1;
            continue;
        }
        // handle no value better? or don't maybe good to fail
        let returned_values: Values = result.json().await?;
        if returned_values.values.is_empty() {
            break;
        }
        for (i, value) in returned_values.values.iter().enumerate() {
            let reservation = kv.kv().reserve_key(keys_to_send.keys[i].clone()).await?;
            kv.kv().put(reservation, value.to_vec()).await?;
            keys_stored += 1
        }
    }
    Ok(())
}
/// Sends a transaction telling the chain it is fully synced
pub async fn tell_chain_syncing_is_done(
    api: &OnlineClient<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
) -> Result<(), ValidatorErr> {
    let synced_tx = entropy::tx().staking_extension().declare_synced(true);
    let _ = api
        .tx()
        .sign_and_submit_then_watch_default(&synced_tx, signer)
        .await?
        .wait_for_in_block()
        .await?
        .wait_for_success()
        .await?;
    Ok(())
}

/// Validation for if an account can cover tx fees for a tx
pub async fn check_balance_for_fees(
    api: &OnlineClient<EntropyConfig>,
    address: &AccountId32,
    min_balance: u128,
) -> Result<bool, ValidatorErr> {
    let balance_query = entropy::storage().system().account(address);
    let account_info =
        api.storage().fetch(&balance_query, None).await?.ok_or_else(|| {
            ValidatorErr::OptionUnwrapError("Account does not exist, add balance")
        })?;
    let balance = account_info.data.free;
    let mut is_min_balance = false;
    if balance >= min_balance {
        is_min_balance = true
    };
    Ok(is_min_balance)
}
