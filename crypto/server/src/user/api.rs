use bip39::{Language, Mnemonic};
use ec_constraints::{
    Architecture, Error as ConstraintsError, Evaluate, Evm, GetReceiver, GetSender, Parse,
};
use entropy_shared::{Acl, AclKind, Arch, Constraints, Message, SIGNING_PARTY_SIZE};
use kvdb::kv_manager::{
    error::{InnerKvError, KvError},
    value::PartyInfo,
    KvManager,
};
use log::info;
use parity_scale_codec::DecodeAll;
use rocket::{
    http::Status,
    response::stream::EventStream,
    serde::json::{to_string, Json},
    Shutdown, State,
};
use serde::{Deserialize, Serialize};
use subxt::{
    ext::{
        sp_core::{crypto::Ss58Codec, sr25519, Pair},
        sp_runtime::AccountId32,
    },
    tx::PairSigner,
    Config, OnlineClient,
};
use tracing::instrument;
use zeroize::Zeroize;

use super::{ParsedUserInputPartyInfo, UserErr, UserInputPartyInfo};
use crate::{
    chain_api::{
        entropy,
        entropy::{
            constraints::calls::UpdateConstraints, runtime_types::entropy_shared::constraints,
        },
        get_api, EntropyConfig,
    },
    helpers::{
        signing::{create_unique_tx_id, do_signing, SignatureState},
        substrate::{get_constraints, get_subgroup},
        validator::get_signer,
    },
    message::SignedMessage,
    signing_client::SignerState,
    Configuration,
};

/// Represents an unparsed, transaction request coming from the client.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GenericTransactionRequest {
    /// 'eth', etc.
    pub arch: String,
    /// ETH: RLP encoded transaction request
    pub transaction_request: String,
}

/// Submits a new transaction to the KVDB for inclusion in a threshold
/// signing scheme at a later block.
/// Maps a tx hash -> unsigned transaction in the kvdb.
#[post("/tx", format = "json", data = "<msg>")]
pub async fn store_tx(
    msg: Json<SignedMessage>,
    state: &State<SignerState>,
    kv: &State<KvManager>,
    signatures: &State<SignatureState>,
    config: &State<Configuration>,
) -> Result<Status, UserErr> {
    // Verifies the message contains a valid sr25519 signature from the sender.
    let signed_msg: SignedMessage = msg.into_inner();
    if !signed_msg.verify() {
        return Err(UserErr::InvalidSignature("Invalid signature."));
    }
    let signer = get_signer(kv).await?;
    let signing_address = signed_msg.account_id().to_ss58check();

    let decrypted_message =
        signed_msg.decrypt(signer.signer()).map_err(|e| UserErr::Decryption(e.to_string()))?;
    let generic_tx_req: GenericTransactionRequest = serde_json::from_slice(&decrypted_message)?;
    match generic_tx_req.arch.as_str() {
        "evm" => {
            let api = get_api(&config.endpoint);
            let parsed_tx = <Evm as Architecture>::TransactionRequest::parse(
                generic_tx_req.transaction_request.clone(),
            )?;

            let sig_hash = hex::encode(parsed_tx.sighash().as_bytes());
            let tx_id = create_unique_tx_id(&signing_address, &sig_hash);
            // check if user submitted tx to chain already
            let message_json = kv.kv().get(&tx_id).await?;
            // parse their transaction request
            let message: Message = serde_json::from_str(&String::from_utf8(message_json)?)?;
            let sig_req_account = <EntropyConfig as Config>::AccountId::from(
                <[u8; 32]>::try_from(message.account.clone()).unwrap(),
            );
            let substrate_api = api.await?;
            let evm_acl = get_constraints(&substrate_api, &sig_req_account)
                .await?
                .evm_acl
                .ok_or(UserErr::Parse("No constraints found for this account."))?;

            evm_acl.eval(parsed_tx)?;
            kv.kv().delete(&tx_id).await?;
            do_signing(message, state, kv, signatures, tx_id).await?;
        },
        _ => {
            return Err(UserErr::Parse("Unknown \"arch\". Must be one of: [\"evm\"]"));
        },
    }
    Ok(Status::Ok)
}

/// Add a new Keyshare to this node's set of known Keyshares.
/// Store in kvdb.
#[post("/new", format = "json", data = "<msg>")]
pub async fn new_user(
    msg: Json<SignedMessage>,
    kv: &State<KvManager>,
    config: &State<Configuration>,
) -> Result<Status, UserErr> {
    let api = get_api(&config.endpoint).await?;
    // Verifies the message contains a valid sr25519 signature from the sender.
    let signed_msg: SignedMessage = msg.into_inner();
    if !signed_msg.verify() {
        return Err(UserErr::InvalidSignature("Invalid signature."));
    }
    let signer = get_signer(kv).await?;
    // Checks if the user has registered onchain first.
    let key = signed_msg.account_id();
    let is_swapping = register_info(&api, &key).await?;

    let decrypted_message =
        signed_msg.decrypt(signer.signer()).map_err(|e| UserErr::Decryption(e.to_string()))?;
    // store new user data in kvdb or deletes and replaces it if swapping
    let subgroup = get_subgroup(&api, &signer)
        .await?
        .ok_or_else(|| UserErr::SubgroupError("Subgroup Error"))?;
    if is_swapping {
        kv.kv().delete(&key.to_string()).await?;
    }
    let reservation = kv.kv().reserve_key(key.to_string()).await?;
    kv.kv().put(reservation, decrypted_message).await?;
    // TODO: Error handling really complex needs to be thought about.
    confirm_registered(&api, key, subgroup, &signer).await?;
    Ok(Status::Ok)
}
/// Returns wether an account is registering or swapping. If it is not, it returns error
pub async fn register_info(
    api: &OnlineClient<EntropyConfig>,
    who: &AccountId32,
) -> Result<bool, UserErr> {
    let registering_info_query = entropy::storage().relayer().registering(who);
    let register_info = api
        .storage()
        .fetch(&registering_info_query, None)
        .await?
        .ok_or_else(|| UserErr::NotRegistering("Register Onchain first"))?;
    if !register_info.is_swapping && !register_info.is_registering {
        return Err(UserErr::NotRegistering("Declare swap Onchain first"));
    }

    Ok(register_info.is_swapping)
}

/// Confirms that a address has finished registering on chain.
pub async fn confirm_registered(
    api: &OnlineClient<EntropyConfig>,
    who: AccountId32,
    subgroup: u8,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
) -> Result<(), subxt::error::Error> {
    // TODO error handling + return error
    // TODO fire and forget, or wait for in block maybe Ddos error
    // TODO: Understand this better, potentially use sign_and_submit_default
    // or other method under sign_and_*
    let registration_tx = entropy::tx().relayer().confirm_register(who, subgroup);
    let _ = api
        .tx()
        .sign_and_submit_then_watch_default(&registration_tx, signer)
        .await?
        .wait_for_in_block()
        .await?
        .wait_for_success()
        .await?;
    Ok(())
}
