use std::{convert::TryInto, str};

use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use entropy_shared::{OCWMessage, PRUNE_BLOCK};
use futures::stream::Stream;
use kvdb::kv_manager::{KvManager, PartyId};
use parity_scale_codec::Decode;
use sp_core::crypto::Ss58Codec;
use subxt::OnlineClient;
use tracing::instrument;

use crate::{
    chain_api::{entropy, get_api, EntropyConfig},
    get_signer,
    helpers::signing::create_unique_tx_id,
    signing_client::{
        subscribe::{Listener, Receiver},
        SigningErr, SubscribeErr, SubscribeMessage,
    },
    validation::SignedMessage,
    AppState,
};

const SUBSCRIBE_TIMEOUT_SECONDS: u64 = 10;
/// Execute a signing protocol with a new party.
/// This endpoint is called by the blockchain.
#[instrument(skip(app_state))]
#[axum_macros::debug_handler]
pub async fn new_party(
    State(app_state): State<AppState>,
    encoded_data: Bytes,
) -> Result<StatusCode, SigningErr> {
    let data = OCWMessage::decode(&mut encoded_data.as_ref())?;
    let api = get_api(&app_state.configuration.endpoint).await?;
    if data.messages.is_empty() {
        prune_old_tx_from_kvdb(&api, &app_state.kv_store, data.block_number).await?;
        return Ok(StatusCode::NO_CONTENT);
    }
    // TODO: removed because complicated to fix and we are removing this path anyways
    // that being said if we do not remove this pathway this needs to be fixed
    // validate_new_party(&data, &api).await?;
    for message in data.messages {
        let address_slice: &[u8; 32] = &message
            .account
            .clone()
            .try_into()
            .map_err(|_| SigningErr::AddressConversionError("Invalid Length".to_string()))?;
        let user = sp_core::crypto::AccountId32::new(*address_slice);

        // TODO: get proper ss58 number when chosen
        let address = user.to_ss58check();

        let tx_id = create_unique_tx_id(&address, &hex::encode(&message.sig_request.sig_hash));
        let reservation = app_state.kv_store.kv().reserve_key(tx_id).await?;
        let value = serde_json::to_string(&message)?;
        app_state.kv_store.kv().put(reservation, value.into()).await?;
    }
    prune_old_tx_from_kvdb(&api, &app_state.kv_store, data.block_number).await?;
    Ok(StatusCode::OK)
}

// /// Other nodes in the party call this method to subscribe to this node's broadcasts.
// /// The SigningProtocol begins when all nodes in the party have called this method on this node.
#[axum_macros::debug_handler]
pub async fn subscribe_to_me(
    State(app_state): State<AppState>,
    signed_msg: Json<SignedMessage>,
) -> Result<Sse<impl Stream<Item = Result<Event, SubscribeErr>>>, SubscribeErr> {
    if !signed_msg.verify() {
        return Err(SubscribeErr::InvalidSignature("Invalid signature."));
    }
    let signer = get_signer(&app_state.kv_store)
        .await
        .map_err(|e| SubscribeErr::UserError(e.to_string()))?;

    let decrypted_message =
        signed_msg.decrypt(signer.signer()).map_err(|e| SubscribeErr::Decryption(e.to_string()))?;
    let msg: SubscribeMessage = serde_json::from_slice(&decrypted_message)?;

    tracing::info!("got subscribe, with message: {msg:?}");

    let party_id = msg.party_id().map_err(SubscribeErr::InvalidPartyId)?;

    let signing_address = signed_msg.account_id();

    // TODO: should we also check if party_id is in signing group -> limited spots in steam so yes
    if PartyId::new(signing_address) != party_id {
        return Err(SubscribeErr::InvalidSignature("Signature does not match party id."));
    }

    if !app_state.signer_state.contains_listener(&msg.session_id)? {
        // Chain node hasn't yet informed this node of the party. Wait for a timeout and proceed
        // or fail below
        tokio::time::sleep(std::time::Duration::from_secs(SUBSCRIBE_TIMEOUT_SECONDS)).await;
    };

    let rx = {
        let mut listeners = app_state
            .signer_state
            .listeners
            .lock()
            .map_err(|e| SubscribeErr::LockError(e.to_string()))?;
        let listener =
            listeners.get_mut(&msg.session_id).ok_or(SubscribeErr::NoListener("no listener"))?;
        let rx_outcome = listener.subscribe(party_id)?;

        // If this is the last subscriber, remove the listener from state
        match rx_outcome {
            Receiver::Receiver(rx) => rx,
            Receiver::FinalReceiver(rx) => {
                // all subscribed, wake up the waiting listener in new_party
                let listener = listeners
                    .remove(&msg.session_id)
                    .ok_or(SubscribeErr::NoListener("listener remove"))?;
                let (tx, broadcaster) = listener.into_broadcaster();
                let _ = tx.send(Ok(broadcaster));
                rx
            },
        }
    };

    Ok(Listener::create_event_stream(rx))
}

// /// Validates new party endpoint
// /// Checks the chain for validity of data and block number of data matches current block
// pub async fn validate_new_party(
//     chain_data: &OCWMessage,
//     api: &OnlineClient<EntropyConfig>,
// ) -> Result<(), SigningErr> {
//     let latest_block_number = api
//         .rpc()
//         .block(None)
//         .await?
//         .ok_or_else(|| SigningErr::OptionUnwrapError("Failed to get block number"))?
//         .block
//         .header
//         .number;

//     // we subtract 1 as the message info is coming from the previous block
//     if latest_block_number.saturating_sub(1) != chain_data.block_number {
//         return Err(SigningErr::StaleData);
//     }

//     let mut hasher_chain_data = Blake2s256::new();
//     hasher_chain_data.update(chain_data.messages.encode());
//     let chain_data_hash = hasher_chain_data.finalize();
//     let mut hasher_verifying_data = Blake2s256::new();

//     let verifying_data_query = entropy::storage().relayer().messages(chain_data.block_number);
//     let verifying_data = api
//         .storage()
//         .fetch(&verifying_data_query, None)
//         .await?
//         .ok_or_else(|| SigningErr::OptionUnwrapError("Failed to get verifying data"))?;

//     hasher_verifying_data.update(verifying_data.encode());

//     let verifying_data_hash = hasher_verifying_data.finalize();
//     if verifying_data_hash != chain_data_hash {
//         return Err(SigningErr::InvalidData);
//     }
//     Ok(())
// }

/// Prunes old tx from DB
pub async fn prune_old_tx_from_kvdb(
    api: &OnlineClient<EntropyConfig>,
    kv: &KvManager,
    block_number: u32,
) -> Result<(), SigningErr> {
    if block_number < PRUNE_BLOCK {
        return Ok(());
    }
    let chain_data_query =
        entropy::storage().relayer().messages(block_number.saturating_sub(PRUNE_BLOCK));
    let chain_data = api.storage().fetch(&chain_data_query, None).await?;

    if chain_data.is_none() {
        return Ok(());
    }

    for message in
        chain_data.ok_or_else(|| SigningErr::OptionUnwrapError("Failed to get verifying data"))?
    {
        let address_slice: &[u8; 32] = &message
            .account
            .clone()
            .try_into()
            .map_err(|_| SigningErr::AddressConversionError("Invalid Length".to_string()))?;
        let user = sp_core::crypto::AccountId32::new(*address_slice);
        // TODO: get proper ss58 number when chosen
        let address = user.to_ss58check();
        let tx_id = create_unique_tx_id(&address, &hex::encode(&message.sig_request.sig_hash));
        let exists_result = kv.kv().exists(&tx_id).await?;
        if exists_result {
            kv.kv().delete(&tx_id).await?;
        }
    }
    Ok(())
}

use serde::{Deserialize, Serialize};

// TODO: JA remove all below temporary
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Clone)]
pub struct Message {
    pub message: String,
}

/// Returns the signature of the requested sighash
///
/// This will be removed when after client participates in signing
pub async fn get_signature(
    State(app_state): State<AppState>,
    Json(msg): Json<Message>,
) -> (StatusCode, String) {
    let sig = match app_state.signature_state.get(&hex::decode(msg.message).unwrap()) {
        Some(sig) => sig,
        None => return (StatusCode::NOT_FOUND, "".to_string()),
    };
    (StatusCode::ACCEPTED, base64::encode(sig.to_rsv_bytes()))
}

/// Drains all signatures from the state
/// Client calls this after they receive the signature at `/signature`
///
/// This will be removed when after client participates in signing
pub async fn drain(State(app_state): State<AppState>) -> Result<StatusCode, ()> {
    app_state.signature_state.drain();
    Ok(StatusCode::OK)
}
