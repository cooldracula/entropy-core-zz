use std::{env, fs, path::PathBuf, sync::Arc};

use bip39::{Language, Mnemonic, MnemonicType};
use entropy_constraints::{Architecture, Evm, Parse};
use entropy_shared::{Message, OCWMessage, SigRequest};
use ethers_core::types::{Address, TransactionRequest};
use futures::{future::join_all, join, Future};
use hex_literal::hex as h;
use kvdb::{clean_tests, encrypted_sled::PasswordMethod, kv_manager::value::KvManager};
use parity_scale_codec::Encode;
use rocket::{
    http::{ContentType, Status},
    tokio::{
        task::JoinSet,
        time::{sleep, Duration},
    },
    Ignite, Rocket,
};
use serial_test::serial;
use sp_core::{crypto::Ss58Codec, sr25519, Bytes, Pair, H160};
use sp_keyring::{AccountKeyring, Sr25519Keyring};
use subxt::{ext::sp_runtime::AccountId32, tx::PairSigner, OnlineClient};
use testing_utils::context::{test_context, test_context_stationary, TestContext};
use tokio::task::JoinHandle;
use x25519_dalek::{PublicKey, StaticSecret};

use super::UserInputPartyInfo;
use crate::{
    chain_api::{entropy, get_api, EntropyConfig},
    drain, get_signature, get_signer,
    helpers::{
        launch::{
            setup_mnemonic, Configuration, DEFAULT_BOB_MNEMONIC, DEFAULT_ENDPOINT, DEFAULT_MNEMONIC,
        },
        signing::{create_unique_tx_id, SignatureState},
        tests::setup_client,
    },
    load_kv_store,
    message::{derive_static_secret, mnemonic_to_pair, new_mnemonic, SignedMessage},
    new_party, new_user,
    r#unsafe::api::{delete, get, put, remove_keys, UnsafeQuery},
    signing_client::{tests::put_tx_request_on_chain, SignerState},
    store_tx, subscribe_to_me,
    validator::api::get_random_server_info,
    Message as SigMessage,
};

#[rocket::async_test]
#[serial]
async fn test_get_signer_does_not_throw_err() {
    clean_tests();
    let kv_store = load_kv_store(false, false).await;
    let mnemonic = setup_mnemonic(&kv_store, false, false).await;
    assert!(mnemonic.is_ok());
    get_signer(&kv_store).await.unwrap();
    clean_tests();
}

#[rocket::async_test]
#[serial]
async fn test_unsigned_tx_endpoint() {
    clean_tests();

    let ports = vec![3001i64, 3002];

    // spawn threshold servers
    join_all(ports.iter().map(|&port| async move {
        tokio::spawn(async move { create_clients(port).await.launch().await.unwrap() })
    }))
    .await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let validator_ips: Vec<String> =
        ports.iter().map(|port| format!("127.0.0.1:{}", port)).collect();

    let transaction_requests = vec![
        // alice
        TransactionRequest::new().to(Address::from([1u8; 20])).value(1),
        // bob
        TransactionRequest::new().to(Address::from([2u8; 20])).value(5),
    ];
    let keyrings = vec![AccountKeyring::Alice, AccountKeyring::Bob];

    let raw_ocw_messages = transaction_requests
        .iter()
        .zip(keyrings.clone())
        .map(|(tx_req, keyring)| Message {
            sig_request: SigRequest { sig_hash: tx_req.sighash().as_bytes().to_vec() },
            account: keyring.to_raw_public_vec(),
            ip_addresses: validator_ips
                .iter()
                .map(|url| url.clone().into_bytes())
                .collect::<Vec<Vec<u8>>>(),
        })
        .collect::<Vec<_>>();

    let place_sig_messages = raw_ocw_messages
        .iter()
        .zip(keyrings.clone())
        .map(|(raw_ocw_message, keyring)| UnsafeQuery {
            key: create_unique_tx_id(
                &keyring.to_account_id().to_ss58check(),
                &hex::encode(raw_ocw_message.sig_request.sig_hash.clone()),
            ),
            value: serde_json::to_string(&raw_ocw_message).unwrap(),
        })
        .collect::<Vec<_>>();

    let mock_client = reqwest::Client::new();
    // put proper data in kvdb
    join_all(place_sig_messages.iter().map(|place_sig_messages| async {
        let res = mock_client
            .post("http://127.0.0.1:3001/unsafe/put")
            .json(place_sig_messages)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);

        let res_2 = mock_client
            .post("http://127.0.0.1:3002/unsafe/put")
            .json(place_sig_messages)
            .send()
            .await
            .unwrap();
        assert_eq!(res_2.status(), 200);
    }))
    .await;

    let validator_urls: Arc<Vec<String>> =
        Arc::new(validator_ips.iter().map(|ip| format!("http://{}", ip)).collect());

    // construct json bodies for transaction requests
    let tx_req_bodies = transaction_requests
        .iter()
        .zip(keyrings.clone())
        .map(|(tx_req, keyring)| {
            serde_json::json!({
                "arch": "evm",
                "transaction_request": tx_req.rlp_unsigned().to_string(),
                "signing_address": keyring.to_account_id()
            })
        })
        .collect::<Vec<_>>();

    // mock client signature requests
    let submit_transaction_requests =
        |validator_urls: Arc<Vec<String>>, tx_req_body: serde_json::Value| async move {
            let mock_client = reqwest::Client::new();
            join_all(
                validator_urls
                    .iter()
                    .map(|url| async {
                        let url = format!("{}/user/tx", url.clone());
                        let res = mock_client.post(url).json(&tx_req_body).send().await;
                        assert_eq!(res.unwrap().status(), 200);
                    })
                    .collect::<Vec<_>>(),
            )
            .await
        };

    // send alice's tx req, then bob's
    submit_transaction_requests(validator_urls.clone(), tx_req_bodies[0].clone()).await;
    submit_transaction_requests(validator_urls.clone(), tx_req_bodies[1].clone()).await;

    // poll a validator and validate the threshold signatures
    let get_sig_messages = raw_ocw_messages
        .iter()
        .map(|raw_ocw_message| SigMessage {
            message: hex::encode(raw_ocw_message.sig_request.sig_hash.clone()),
        })
        .collect::<Vec<_>>();

    join_all(get_sig_messages.iter().map(|get_sig_message| async {
        let res = mock_client
            .post("http://127.0.0.1:3001/signer/signature")
            .json(get_sig_message)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 202);
        assert_eq!(res.content_length().unwrap(), 88);
    }))
    .await;

    // if unsafe, then also validate signature deletion
    // delete all signatures from the servers
    join_all(validator_urls.iter().map(|url| async {
        let url = format!("{}/signer/drain", url.clone());
        let res = mock_client.get(url).send().await;
        assert_eq!(res.unwrap().status(), 200);
    }))
    .await;

    // query the signature again, should error since we just deleted them
    join_all(get_sig_messages.iter().map(|get_sig_message| async {
        let res = mock_client
            .post("http://127.0.0.1:3001/signer/signature")
            .json(get_sig_message)
            .send()
            .await;
        assert_eq!(res.as_ref().unwrap().status(), 500);
    }))
    .await;

    clean_tests();
}

#[rocket::async_test]
#[serial]
async fn test_store_share() {
    clean_tests();
    let alice = AccountKeyring::Alice;
    let alice_constraint = AccountKeyring::Charlie;
    let alice_stash_id: AccountId32 =
        h!["be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f"].into();

    let value: Vec<u8> = vec![0];

    let cxt = test_context_stationary().await;
    let client = setup_client().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();

    let threshold_servers_query =
        entropy::storage().staking_extension().threshold_servers(&alice_stash_id);
    let query_result = api.storage().fetch(&threshold_servers_query, None).await.unwrap();
    assert!(query_result.is_some());

    let res = query_result.unwrap();
    let server_public_key = PublicKey::from(res.x25519_public_key);
    let user_input = SignedMessage::new(&alice.pair(), &Bytes(value.clone()), &server_public_key)
        .unwrap()
        .to_json();

    // fails to add not registering or swapping
    let response = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input.clone())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(
        response.into_string().await.unwrap(),
        "Not Registering error: Register Onchain first"
    );

    // signal registering
    make_register(&api, &alice, &alice_constraint).await;

    let response_2 = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input.clone())
        .dispatch()
        .await;
    assert_eq!(response_2.status(), Status::Ok);
    assert_eq!(response_2.into_string().await, None);
    // make sure there is now one confirmation
    check_if_confirmation(&api, &alice).await;

    // fails to add already added share
    let response_3 = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input.clone())
        .dispatch()
        .await;

    assert_eq!(response_3.status(), Status::InternalServerError);
    assert_eq!(response_3.into_string().await.unwrap(), "Kv error: Recv Error: channel closed");

    // fails with wrong node key
    let bob_stash_id: AccountId32 =
        h!["fe65717dad0447d715f660a0a58411de509b42e6efb8375f562f58a554d5860e"].into();

    let query_bob = entropy::storage().staking_extension().threshold_servers(&bob_stash_id);
    let query_result_bob = api.storage().fetch(&query_bob, None).await.unwrap();
    let res_bob = query_result_bob.unwrap();
    let server_public_key_bob = PublicKey::from(res_bob.x25519_public_key);
    let user_input_bob =
        SignedMessage::new(&alice.pair(), &Bytes(value.clone()), &server_public_key_bob)
            .unwrap()
            .to_json();

    let response_4 = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input_bob.clone())
        .dispatch()
        .await;

    assert_eq!(response_4.status(), Status::InternalServerError);
    let expected_err = "ChaCha20 decryption error: aead::Error";
    assert_eq!(response_4.into_string().await.unwrap(), expected_err);
    let sig: [u8; 64] = [0; 64];
    let slice: [u8; 32] = [0; 32];
    let nonce: [u8; 12] = [0; 12];
    let user_input_bad = SignedMessage::new_test(
        Bytes(value),
        sr25519::Signature::from_raw(sig),
        slice,
        slice,
        slice,
        nonce,
    )
    .to_json();

    let response_5 = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input_bad.clone())
        .dispatch()
        .await;

    assert_eq!(response_5.status(), Status::InternalServerError);
    assert_eq!(response_5.into_string().await.unwrap(), "Invalid Signature: Invalid signature.");
    clean_tests();
}

#[rocket::async_test]
#[serial]
async fn test_update_keys() {
    clean_tests();
    let dave = AccountKeyring::Dave;
    let alice_stash_id: AccountId32 =
        h!["be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f"].into();

    let key: AccountId32 = dave.to_account_id();
    let value: Vec<u8> = vec![0];
    let new_value: Vec<u8> = vec![1];
    let cxt = test_context_stationary().await;
    let client = setup_client().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();

    let threshold_servers_query =
        entropy::storage().staking_extension().threshold_servers(&alice_stash_id);
    let query_result = api.storage().fetch(&threshold_servers_query, None).await.unwrap();
    assert!(query_result.is_some());

    let res = query_result.unwrap();
    let server_public_key = PublicKey::from(res.x25519_public_key);
    let user_input =
        SignedMessage::new(&dave.pair(), &Bytes(new_value.clone()), &server_public_key)
            .unwrap()
            .to_json();

    let put_query =
        UnsafeQuery::new(key.to_string(), serde_json::to_string(&value).unwrap()).to_json();
    // manually add dave's key to replace it
    let response = client
        .post("/unsafe/put")
        .header(ContentType::JSON)
        .body(put_query.clone())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    // fails to add not registering or swapping
    let response_2 = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input.clone())
        .dispatch()
        .await;

    assert_eq!(response_2.status(), Status::InternalServerError);
    assert_eq!(
        response_2.into_string().await.unwrap(),
        "Not Registering error: Register Onchain first" /* "Generic Substrate error:
                                                         * Metadata: Pallet Relayer Storage
                                                         * Relayer has incompatible
                                                         * metadata" */
    );

    // signal registering
    make_swapping(&api, &dave).await;

    let response_3 = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(user_input.clone())
        .dispatch()
        .await;
    assert_eq!(response_3.status(), Status::Ok);
    assert_eq!(response_3.into_string().await, None);
    // make sure there is now one confirmation
    check_if_confirmation(&api, &dave).await;

    // check dave has new key
    let response_4 = client
        .post("/unsafe/get")
        .header(ContentType::JSON)
        .body(put_query.clone())
        .dispatch()
        .await;

    assert_eq!(
        response_4.into_string().await,
        Some(std::str::from_utf8(&new_value).unwrap().to_string())
    );
    clean_tests();
}

#[rocket::async_test]
#[serial]
async fn test_store_share_fail_wrong_data() {
    clean_tests();
    // Construct a client to use for dispatching requests.
    let client = setup_client().await;
    let response = client
        .post("/user/new")
        .header(ContentType::JSON)
        .body(
            r##"{
		"name": "John Doe",
		"email": "j.doe@m.com",
		"password": "123456"
	}"##,
        )
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::UnprocessableEntity);
    clean_tests();
}

pub async fn make_register(
    api: &OnlineClient<EntropyConfig>,
    sig_req_keyring: &Sr25519Keyring,
    constraint_keyring: &Sr25519Keyring,
) {
    clean_tests();
    let sig_req_account =
        PairSigner::<EntropyConfig, sp_core::sr25519::Pair>::new(sig_req_keyring.pair());
    let registering_query =
        entropy::storage().relayer().registering(sig_req_keyring.to_account_id());
    let is_registering_1 = api.storage().fetch(&registering_query, None).await.unwrap();
    assert!(is_registering_1.is_none());

    let registering_tx = entropy::tx().relayer().register(constraint_keyring.to_account_id(), None);

    api.tx()
        .sign_and_submit_then_watch_default(&registering_tx, &sig_req_account)
        .await
        .unwrap()
        .wait_for_in_block()
        .await
        .unwrap()
        .wait_for_success()
        .await
        .unwrap();

    let is_registering_2 = api.storage().fetch(&registering_query, None).await;
    assert!(is_registering_2.unwrap().unwrap().is_registering);
}

pub async fn make_swapping(api: &OnlineClient<EntropyConfig>, key: &Sr25519Keyring) {
    let signer = PairSigner::new(key.pair());
    let registering_query = entropy::storage().relayer().registering(key.to_account_id());
    let is_registering_1 = api.storage().fetch(&registering_query, None).await.unwrap();
    assert!(is_registering_1.is_none());

    let registering_tx = entropy::tx().relayer().swap_keys();

    api.tx()
        .sign_and_submit_then_watch_default(&registering_tx, &signer)
        .await
        .unwrap()
        .wait_for_in_block()
        .await
        .unwrap()
        .wait_for_success()
        .await
        .unwrap();

    let is_registering_2 = api.storage().fetch(&registering_query, None).await;
    assert!(is_registering_2.unwrap().unwrap().is_registering);
}

pub async fn check_if_confirmation(api: &OnlineClient<EntropyConfig>, key: &Sr25519Keyring) {
    let registering_query = entropy::storage().relayer().registering(key.to_account_id());
    let registered_query = entropy::storage().relayer().registered(key.to_account_id());
    let is_registering = api.storage().fetch(&registering_query, None).await.unwrap();
    // make sure there is one confirmation
    assert_eq!(is_registering.unwrap().confirmations.len(), 1);
    let _ = api.storage().fetch(&registered_query, None).await.unwrap();
}

async fn create_clients(port: i64) -> Rocket<Ignite> {
    let config = rocket::Config::figment().merge(("port", port));

    let signer_state = SignerState::default();
    let configuration = Configuration::new(DEFAULT_ENDPOINT.to_string());
    let signature_state = SignatureState::new();

    let path = format!("test_db_{}", port.to_string());
    let _ = std::fs::remove_dir_all(path.clone());

    let kv_store =
        KvManager::new(path.into(), PasswordMethod::NoPassword.execute().unwrap()).unwrap();

    // Shortcut: store the shares manually
    let root = project_root::get_project_root().unwrap();
    let share_id = i32::from(port != 3001);
    let path: PathBuf =
        [root, "test_data".into(), "key_shares".into(), share_id.to_string().into()]
            .into_iter()
            .collect();
    let v_serialized = fs::read(path).unwrap();
    let alice_key = AccountKeyring::Alice.to_account_id();
    let bob_key = AccountKeyring::Bob.to_account_id();
    let alice_reservation = kv_store.kv().reserve_key(alice_key.to_string()).await.unwrap();
    let bob_reservation = kv_store.kv().reserve_key(bob_key.to_string()).await.unwrap();
    // alice and bob reuse the same keyshares for testing
    let _ = kv_store.kv().put(alice_reservation, v_serialized.clone()).await;
    let _ = kv_store.kv().put(bob_reservation, v_serialized).await;

    rocket::custom(config)
        .mount("/signer", routes![new_party, subscribe_to_me, get_signature, drain])
        .mount("/user", routes![store_tx, new_user])
        .mount("/unsafe", routes![remove_keys, get, put, delete])
        .manage(signer_state)
        .manage(configuration)
        .manage(kv_store)
        .manage(signature_state)
        .ignite()
        .await
        .unwrap()
}
