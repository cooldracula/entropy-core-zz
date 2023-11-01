use std::{
    env, fs,
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use axum::http::StatusCode;
use bip39::{Language, Mnemonic, MnemonicType};
use entropy_protocol::{
    protocol_transport::{noise::noise_handshake_initiator, SubscribeMessage, WsConnection},
    user::{user_participates_in_dkg_protocol, user_participates_in_signing_protocol},
    KeyParams, PartyId, ValidatorInfo,
};
use entropy_shared::{KeyVisibility, OcwMessage};
use futures::{
    future::{self, join_all},
    join, Future, SinkExt, StreamExt,
};
use hex_literal::hex;
use kvdb::{clean_tests, encrypted_sled::PasswordMethod, kv_manager::value::KvManager};
use more_asserts as ma;
use parity_scale_codec::Encode;
use serial_test::serial;
use sp_core::{crypto::Ss58Codec, Pair as OtherPair, H160};
use sp_keyring::{AccountKeyring, Sr25519Keyring};
use subxt::{
    backend::legacy::LegacyRpcMethods,
    ext::{
        sp_core::{sr25519, sr25519::Signature, Bytes, Pair},
        sp_runtime::AccountId32,
    },
    tx::PairSigner,
    utils::{AccountId32 as subxtAccountId32, Static},
    Config, OnlineClient,
};
use synedrion::{
    k256::ecdsa::{RecoveryId, Signature as k256Signature, VerifyingKey},
    KeyShare,
};
use testing_utils::{
    constants::{
        ALICE_STASH_ADDRESS, BAREBONES_PROGRAM_WASM_BYTECODE, MESSAGE_SHOULD_FAIL,
        MESSAGE_SHOULD_SUCCEED, TSS_ACCOUNTS, X25519_PUBLIC_KEYS,
    },
    substrate_context::{
        test_context_stationary, test_node_process_testing_state, SubstrateTestingContext,
    },
};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use x25519_dalek::{PublicKey, StaticSecret};

use super::UserInputPartyInfo;
use crate::{
    chain_api::{entropy, get_api, get_rpc, EntropyConfig},
    get_signer,
    helpers::{
        launch::{
            setup_mnemonic, Configuration, DEFAULT_BOB_MNEMONIC, DEFAULT_CHARLIE_MNEMONIC,
            DEFAULT_ENDPOINT, DEFAULT_MNEMONIC,
        },
        signing::{create_unique_tx_id, Hasher},
        substrate::{get_subgroup, make_register, return_all_addresses_of_subgroup},
        tests::{
            check_if_confirmation, create_clients, keyring_to_subxt_signer_and_x25519,
            setup_client, spawn_testing_validators, update_programs,
        },
        user::send_key,
    },
    load_kv_store, new_user,
    r#unsafe::api::UnsafeQuery,
    signing_client::ListenerState,
    user::api::{recover_key, UserRegistrationInfo, UserTransactionRequest},
    validation::{derive_static_secret, mnemonic_to_pair, new_mnemonic, SignedMessage},
    validator::api::get_random_server_info,
};

#[tokio::test]
#[serial]
async fn test_get_signer_does_not_throw_err() {
    clean_tests();
    let kv_store = load_kv_store(false, false, false).await;
    let mnemonic = setup_mnemonic(&kv_store, false, false).await;
    assert!(mnemonic.is_ok());
    get_signer(&kv_store).await.unwrap();
    clean_tests();
}
#[tokio::test]
#[serial]
async fn test_sign_tx_no_chain() {
    clean_tests();
    let one = AccountKeyring::Dave;
    let two = AccountKeyring::Two;

    let signing_address = one.clone().to_account_id().to_ss58check();
    let (validator_ips, _validator_ids, keyshare_option) =
        spawn_testing_validators(Some(signing_address.clone()), false).await;
    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();

    update_programs(
        &entropy_api,
        &one.pair(),
        &one.pair(),
        BAREBONES_PROGRAM_WASM_BYTECODE.to_owned(),
    )
    .await;

    let message_should_succeed_hash = Hasher::keccak(MESSAGE_SHOULD_SUCCEED);

    let validators_info = vec![
        ValidatorInfo {
            ip_address: "localhost:3001".to_socket_addrs().unwrap().next().unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone(),
        },
        ValidatorInfo {
            ip_address: SocketAddr::from_str("127.0.0.1:3002").unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let converted_transaction_request: String = hex::encode(MESSAGE_SHOULD_SUCCEED);

    let signing_address = one.to_account_id().to_ss58check();
    let hash_as_string = hex::encode(&message_should_succeed_hash);
    let sig_uid = create_unique_tx_id(&signing_address, &hash_as_string);

    let mut generic_msg = UserTransactionRequest {
        transaction_request: converted_transaction_request.clone(),
        validators_info,
        timestamp: SystemTime::now(),
    };

    let submit_transaction_requests =
        |validator_urls_and_keys: Vec<(String, [u8; 32])>,
         generic_msg: UserTransactionRequest,
         keyring: Sr25519Keyring| async move {
            let mock_client = reqwest::Client::new();
            join_all(
                validator_urls_and_keys
                    .iter()
                    .map(|validator_tuple| async {
                        let server_public_key = PublicKey::from(validator_tuple.1);
                        let signed_message = SignedMessage::new(
                            &keyring.pair(),
                            &Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
                            &server_public_key,
                        )
                        .unwrap();
                        let url = format!("http://{}/user/sign_tx", validator_tuple.0.clone());
                        mock_client
                            .post(url)
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&signed_message).unwrap())
                            .send()
                            .await
                    })
                    .collect::<Vec<_>>(),
            )
            .await
        };
    let validator_ips_and_keys = vec![
        (validator_ips[0].clone(), X25519_PUBLIC_KEYS[0]),
        (validator_ips[1].clone(), X25519_PUBLIC_KEYS[1]),
    ];

    generic_msg.timestamp = SystemTime::now();
    let test_user_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    verify_signature(test_user_res, message_should_succeed_hash, keyshare_option.clone()).await;

    generic_msg.timestamp = SystemTime::now();
    generic_msg.validators_info = generic_msg.validators_info.into_iter().rev().collect::<Vec<_>>();
    let test_user_res_order =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    verify_signature(test_user_res_order, message_should_succeed_hash, keyshare_option.clone())
        .await;

    generic_msg.timestamp = SystemTime::now();
    // test failing cases
    let test_user_res_not_registered =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), two).await;

    for res in test_user_res_not_registered {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Not Registering error: Register Onchain first"
        );
    }

    // Test attempting to connect over ws by someone who is not in the signing group
    let validator_ip_and_key = validator_ips_and_keys[0].clone();
    let connection_attempt_handle = tokio::spawn(async move {
        // Wait for the "user" to submit the signing request
        tokio::time::sleep(Duration::from_millis(500)).await;
        let ws_endpoint = format!("ws://{}/ws", validator_ip_and_key.0);
        let (ws_stream, _response) = connect_async(ws_endpoint).await.unwrap();

        let (ferdie_keypair, ferdie_x25519_sk) =
            keyring_to_subxt_signer_and_x25519(&AccountKeyring::Ferdie);

        // create a SubscribeMessage from a party who is not in the signing commitee
        let subscribe_message_vec =
            serde_json::to_vec(&SubscribeMessage::new(&sig_uid, &ferdie_keypair)).unwrap();

        // Attempt a noise handshake including the subscribe message in the payload
        let mut encrypted_connection = noise_handshake_initiator(
            ws_stream,
            &ferdie_x25519_sk,
            validator_ip_and_key.1,
            subscribe_message_vec,
        )
        .await
        .unwrap();

        // Check the response as to whether they accepted our SubscribeMessage
        let response_message = encrypted_connection.recv().await.unwrap();
        let subscribe_response: Result<(), String> =
            serde_json::from_str(&response_message).unwrap();

        assert_eq!(Err("NoListener(\"no listener\")".to_string()), subscribe_response);
        // The stream should not continue to send messages
        // returns true if this part of the test passes
        encrypted_connection.recv().await.is_err()
    });

    generic_msg.timestamp = SystemTime::now();
    let test_user_bad_connection_res = submit_transaction_requests(
        vec![validator_ips_and_keys[1].clone()],
        generic_msg.clone(),
        one,
    )
    .await;

    for res in test_user_bad_connection_res {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "{\"Err\":\"Timed out waiting for remote party\"}"
        );
    }

    assert!(connection_attempt_handle.await.unwrap());

    // Bad Account ID - an account ID is given which is not in the signing group
    generic_msg.timestamp = SystemTime::now();
    let mut generic_msg_bad_account_id = generic_msg.clone();
    generic_msg_bad_account_id.validators_info[0].tss_account =
        subxtAccountId32(AccountKeyring::Dave.into());

    let test_user_failed_tss_account = submit_transaction_requests(
        validator_ips_and_keys.clone(),
        generic_msg_bad_account_id,
        one,
    )
    .await;

    for res in test_user_failed_tss_account {
        let res = res.unwrap();
        assert_eq!(res.status(), 500);
        assert_eq!(res.text().await.unwrap(), "Invalid Signer: Invalid Signer in Signing group");
    }

    // Test a transcation which does not pass constaints
    generic_msg.transaction_request = hex::encode(MESSAGE_SHOULD_FAIL);
    generic_msg.timestamp = SystemTime::now();

    let test_user_failed_programs_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_user_failed_programs_res {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Runtime error: Runtime(Error::Evaluation(\"Length of data is too short.\"))"
        );
    }

    let mock_client = reqwest::Client::new();
    // fails verification tests
    // wrong key for wrong validator
    let server_public_key = PublicKey::from(X25519_PUBLIC_KEYS[1]);
    let failed_signed_message = SignedMessage::new(
        &one.pair(),
        &Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        &server_public_key,
    )
    .unwrap();
    let failed_res = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&failed_signed_message).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(failed_res.status(), 500);
    assert_eq!(
        failed_res.text().await.unwrap(),
        "Validation error: ChaCha20 decryption error: aead::Error"
    );

    let sig: [u8; 64] = [0; 64];
    let slice: [u8; 32] = [0; 32];
    let nonce: [u8; 12] = [0; 12];

    let user_input_bad = SignedMessage::new_test(
        Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        sr25519::Signature::from_raw(sig),
        AccountKeyring::Eve.pair().public().into(),
        slice,
        slice,
        nonce,
    );

    let failed_sign = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&user_input_bad).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(failed_sign.status(), 500);
    assert_eq!(failed_sign.text().await.unwrap(), "Invalid Signature: Invalid signature.");

    // checks that sig not needed with public key visibility
    let user_input_bad = SignedMessage::new_test(
        Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        sr25519::Signature::from_raw(sig),
        AccountKeyring::Dave.pair().public().into(),
        slice,
        slice,
        nonce,
    );

    let failed_sign = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&user_input_bad).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(failed_sign.status(), 500);
    // fails lower down in stack because no sig needed on pub account
    // fails when tries to decode the nonsense message
    assert_ne!(failed_sign.text().await.unwrap(), "Invalid Signature: Invalid signature.");
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_fail_signing_group() {
    clean_tests();
    let dave = AccountKeyring::Dave;
    let _ = spawn_testing_validators(None, false).await;

    let _substrate_context = test_node_process_testing_state().await;
    let message_raw = MESSAGE_SHOULD_SUCCEED.to_owned();

    let validators_info = vec![
        ValidatorInfo {
            ip_address: SocketAddr::from_str("127.0.0.1:3001").unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: hex!["a664add5dfaca1dd560b949b5699b5f0c3c1df3a2ea77ceb0eeb4f77cc3ade04"]
                .into(),
        },
        ValidatorInfo {
            ip_address: SocketAddr::from_str("127.0.0.1:3002").unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let generic_msg = UserTransactionRequest {
        transaction_request: hex::encode(message_raw),
        validators_info,
        timestamp: SystemTime::now(),
    };
    let server_public_key = PublicKey::from(X25519_PUBLIC_KEYS[0]);
    let signed_message = SignedMessage::new(
        &dave.pair(),
        &Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        &server_public_key,
    )
    .unwrap();

    let mock_client = reqwest::Client::new();
    let response = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&signed_message).unwrap())
        .send()
        .await;
    assert_eq!(
        response.unwrap().text().await.unwrap(),
        "Invalid Signer: Invalid Signer in Signing group"
    );
    clean_tests();
}

// TODO negative validation tests on user/tx

#[tokio::test]
#[serial]
async fn test_store_share() {
    clean_tests();
    let alice = AccountKeyring::Alice;
    let alice_program = AccountKeyring::Charlie;

    let cxt = test_context_stationary().await;
    let (_validator_ips, _validator_ids, _) = spawn_testing_validators(None, false).await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let mut block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number + 1;
    let validators_info = vec![
        entropy_shared::ValidatorInfo {
            ip_address: b"127.0.0.1:3001".to_vec(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone().encode(),
        },
        entropy_shared::ValidatorInfo {
            ip_address: b"127.0.0.1:3002".to_vec(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone().encode(),
        },
    ];
    let mut onchain_user_request =
        OcwMessage { sig_request_accounts: vec![alice.encode()], block_number, validators_info };
    let client = reqwest::Client::new();

    put_register_request_on_chain(
        &api,
        &alice,
        alice_program.to_account_id().into(),
        KeyVisibility::Public,
    )
    .await;

    run_to_block(&rpc, block_number + 1).await;

    // succeeds
    let response = client
        .post("http://127.0.0.1:3002/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response.text().await.unwrap(), "");

    // Wait until user is confirmed as registered
    let alice_account_id: <EntropyConfig as Config>::AccountId = alice.to_account_id().into();
    let registered_query = entropy::storage().relayer().registered(alice_account_id);
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let block_hash = rpc.chain_get_block_hash(None).await.unwrap().unwrap();
        let query_registered_status = api.storage().at(block_hash).fetch(&registered_query).await;
        if query_registered_status.unwrap().is_some() {
            break;
        }
    }

    let get_query = UnsafeQuery::new(alice.to_account_id().to_string(), "".to_string()).to_json();

    // check alice has new key
    let response_2 = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    ma::assert_gt!(response_2.text().await.unwrap().len(), 1000);

    // fails repeated data
    let response_3 = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_3.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_3.text().await.unwrap(), "Data is repeated");

    run_to_block(&rpc, block_number + 3).await;
    onchain_user_request.block_number = block_number + 1;
    // fails stale data
    let response_4 = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_4.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_4.text().await.unwrap(), "Data is stale");

    block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number + 1;
    put_register_request_on_chain(
        &api,
        &alice_program,
        alice_program.to_account_id().into(),
        KeyVisibility::Public,
    )
    .await;
    onchain_user_request.block_number = block_number;
    run_to_block(&rpc, block_number + 1).await;

    // fails not verified data
    let response_5 = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_5.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_5.text().await.unwrap(), "Data is not verifiable");

    onchain_user_request.validators_info[0].tss_account = TSS_ACCOUNTS[1].clone().encode();
    // fails not in validator group data
    let response_6 = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_6.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_6.text().await.unwrap(), "Invalid Signer: Invalid Signer in Signing group");

    check_if_confirmation(&api, &rpc, &alice.pair()).await;
    // TODO check if key is in other subgroup member
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_return_addresses_of_subgroup() {
    let cxt = test_context_stationary().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let result = return_all_addresses_of_subgroup(&api, &rpc, 0u8).await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_send_and_receive_keys() {
    clean_tests();
    let alice = AccountKeyring::Alice;

    let cxt = test_context_stationary().await;
    setup_client().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let user_registration_info = UserRegistrationInfo {
        key: alice.to_account_id().to_string(),
        value: vec![10],
        proactive_refresh: false,
    };

    let p_alice = <sr25519::Pair as Pair>::from_string(DEFAULT_MNEMONIC, None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);
    let client = reqwest::Client::new();
    // sends key to alice validator, while filtering out own key
    let _ = send_key(
        &api,
        &rpc,
        &alice.to_account_id().into(),
        &mut vec![ALICE_STASH_ADDRESS.clone(), alice.to_account_id().into()],
        user_registration_info.clone(),
        &signer_alice,
    )
    .await
    .unwrap();

    let get_query = UnsafeQuery::new(user_registration_info.key.clone(), "".to_string()).to_json();

    // check alice has new key
    let response_2 = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(
        response_2.text().await.unwrap(),
        std::str::from_utf8(&user_registration_info.value.clone()).unwrap().to_string()
    );
    let server_public_key = PublicKey::from(X25519_PUBLIC_KEYS[0]);

    let signed_message = SignedMessage::new(
        &signer_alice.signer(),
        &Bytes(serde_json::to_vec(&user_registration_info.clone()).unwrap()),
        &PublicKey::from(server_public_key),
    )
    .unwrap()
    .to_json();

    // fails key already stored
    let response_3 = client
        .post("http://127.0.0.1:3001/user/receive_key")
        .header("Content-Type", "application/json")
        .body(signed_message.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response_3.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_3.text().await.unwrap(), "User already registered");

    // TODO negative validation tests

    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_recover_key() {
    clean_tests();
    let cxt = test_node_process_testing_state().await;
    setup_client().await;
    let (_, bob_kv) = create_clients("validator2".to_string(), vec![], vec![], false, true).await;

    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();
    let unsafe_query = UnsafeQuery::new("key".to_string(), "value".to_string());
    let client = reqwest::Client::new();

    let _ = client
        .post("http://127.0.0.1:3001/unsafe/put")
        .header("Content-Type", "application/json")
        .body(unsafe_query.clone().to_json())
        .send()
        .await
        .unwrap();
    let p_alice = <sr25519::Pair as Pair>::from_string(DEFAULT_CHARLIE_MNEMONIC, None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);
    let _ =
        recover_key(&api, &rpc, &bob_kv, &signer_alice, unsafe_query.key.clone()).await.unwrap();

    let value = bob_kv.kv().get(&unsafe_query.key).await.unwrap();
    assert_eq!(value, unsafe_query.value.into_bytes());
    clean_tests();
}

pub async fn put_register_request_on_chain(
    api: &OnlineClient<EntropyConfig>,
    sig_req_keyring: &Sr25519Keyring,
    program_modification_account: subxtAccountId32,
    key_visibility: KeyVisibility,
) {
    let sig_req_account =
        PairSigner::<EntropyConfig, sp_core::sr25519::Pair>::new(sig_req_keyring.pair());

    let empty_program = vec![];
    let registering_tx = entropy::tx().relayer().register(
        program_modification_account,
        Static(key_visibility),
        empty_program,
    );

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
}

pub async fn run_to_block(rpc: &LegacyRpcMethods<EntropyConfig>, block_run: u32) {
    let mut current_block = 0;
    while current_block < block_run {
        current_block = rpc.chain_get_header(None).await.unwrap().unwrap().number;
    }
}

#[tokio::test]
#[serial]
async fn test_sign_tx_user_participates() {
    clean_tests();
    let one = AccountKeyring::Eve;
    let two = AccountKeyring::Two;

    let signing_address = one.clone().to_account_id().to_ss58check();
    let (validator_ips, _validator_ids, users_keyshare_option) =
        spawn_testing_validators(Some(signing_address.clone()), true).await;
    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();

    update_programs(
        &entropy_api,
        &one.pair(),
        &one.pair(),
        BAREBONES_PROGRAM_WASM_BYTECODE.to_owned(),
    )
    .await;

    let validators_info = vec![
        ValidatorInfo {
            ip_address: SocketAddr::from_str("127.0.0.1:3001").unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone(),
        },
        ValidatorInfo {
            ip_address: SocketAddr::from_str("127.0.0.1:3002").unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let converted_transaction_request: String = hex::encode(MESSAGE_SHOULD_SUCCEED);
    let message_should_succeed_hash = Hasher::keccak(MESSAGE_SHOULD_SUCCEED);

    let signing_address = one.clone().to_account_id().to_ss58check();
    let hash_as_string = hex::encode(&message_should_succeed_hash);
    let sig_uid = create_unique_tx_id(&signing_address, &hash_as_string);

    let mut generic_msg = UserTransactionRequest {
        transaction_request: converted_transaction_request.clone(),
        validators_info: validators_info.clone(),
        timestamp: SystemTime::now(),
    };

    let submit_transaction_requests =
        |validator_urls_and_keys: Vec<(String, [u8; 32])>,
         generic_msg: UserTransactionRequest,
         keyring: Sr25519Keyring| async move {
            let mock_client = reqwest::Client::new();
            join_all(
                validator_urls_and_keys
                    .iter()
                    .map(|validator_tuple| async {
                        let server_public_key = PublicKey::from(validator_tuple.1);
                        let signed_message = SignedMessage::new(
                            &keyring.pair(),
                            &Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
                            &server_public_key,
                        )
                        .unwrap();
                        let url = format!("http://{}/user/sign_tx", validator_tuple.0.clone());
                        mock_client
                            .post(url)
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&signed_message).unwrap())
                            .send()
                            .await
                    })
                    .collect::<Vec<_>>(),
            )
            .await
        };
    let validator_ips_and_keys = vec![
        (validator_ips[0].clone(), X25519_PUBLIC_KEYS[0]),
        (validator_ips[1].clone(), X25519_PUBLIC_KEYS[1]),
    ];
    generic_msg.timestamp = SystemTime::now();

    let (one_keypair, one_x25519_sk) = keyring_to_subxt_signer_and_x25519(&one);

    // Submit transaction requests, and connect and participate in signing
    let (test_user_res, sig_result) = future::join(
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one),
        user_participates_in_signing_protocol(
            &users_keyshare_option.clone().unwrap(),
            &sig_uid,
            validators_info.clone(),
            &one_keypair,
            message_should_succeed_hash,
            &one_x25519_sk,
        ),
    )
    .await;

    let signature_base64 = base64::encode(sig_result.unwrap().to_rsv_bytes());
    assert_eq!(signature_base64.len(), 88);

    verify_signature(test_user_res, message_should_succeed_hash, users_keyshare_option.clone())
        .await;

    generic_msg.timestamp = SystemTime::now();
    // test failing cases
    let test_user_res_not_registered =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), two).await;

    for res in test_user_res_not_registered {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Not Registering error: Register Onchain first"
        );
    }

    generic_msg.timestamp = SystemTime::now();
    let mut generic_msg_bad_validators = generic_msg.clone();
    generic_msg_bad_validators.validators_info[0].x25519_public_key = [0; 32];

    let test_user_failed_x25519_pub_key = submit_transaction_requests(
        validator_ips_and_keys.clone(),
        generic_msg_bad_validators,
        one,
    )
    .await;

    let mut responses = test_user_failed_x25519_pub_key.into_iter();
    assert_eq!(
        responses.next().unwrap().unwrap().text().await.unwrap(),
        "{\"Err\":\"Timed out waiting for remote party\"}"
    );

    assert_eq!(
        responses.next().unwrap().unwrap().text().await.unwrap(),
        "{\"Err\":\"Timed out waiting for remote party\"}"
    );

    // Test attempting to connect over ws by someone who is not in the signing group
    let validator_ip_and_key = validator_ips_and_keys[0].clone();
    let connection_attempt_handle = tokio::spawn(async move {
        // Wait for the "user" to submit the signing request
        tokio::time::sleep(Duration::from_millis(500)).await;
        let ws_endpoint = format!("ws://{}/ws", validator_ip_and_key.0);
        let (ws_stream, _response) = connect_async(ws_endpoint).await.unwrap();

        let ferdie_keypair = subxt_signer::sr25519::Keypair::from_uri(
            &subxt_signer::SecretUri::from_str(&AccountKeyring::Ferdie.to_seed()).unwrap(),
        )
        .unwrap();
        let ferdie_x25519_sk = derive_static_secret(&AccountKeyring::Ferdie.pair());

        // create a SubscribeMessage from a party who is not in the signing commitee
        let subscribe_message_vec =
            serde_json::to_vec(&SubscribeMessage::new(&sig_uid, &ferdie_keypair)).unwrap();

        // Attempt a noise handshake including the subscribe message in the payload
        let mut encrypted_connection = noise_handshake_initiator(
            ws_stream,
            &ferdie_x25519_sk,
            validator_ip_and_key.1,
            subscribe_message_vec,
        )
        .await
        .unwrap();

        // Check the response as to whether they accepted our SubscribeMessage
        let response_message = encrypted_connection.recv().await.unwrap();
        let subscribe_response: Result<(), String> =
            serde_json::from_str(&response_message).unwrap();

        assert_eq!(
            Err("Decryption(\"Public key does not match that given in UserTransactionRequest or \
                 register transaction\")"
                .to_string()),
            subscribe_response
        );
        // The stream should not continue to send messages
        // returns true if this part of the test passes
        encrypted_connection.recv().await.is_err()
    });
    generic_msg.timestamp = SystemTime::now();

    let test_user_bad_connection_res = submit_transaction_requests(
        vec![validator_ips_and_keys[1].clone()],
        generic_msg.clone(),
        one,
    )
    .await;

    for res in test_user_bad_connection_res {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "{\"Err\":\"Timed out waiting for remote party\"}"
        );
    }

    assert!(connection_attempt_handle.await.unwrap());
    generic_msg.timestamp = SystemTime::now();
    // Bad Account ID - an account ID is given which is not in the signing group
    let mut generic_msg_bad_account_id = generic_msg.clone();
    generic_msg_bad_account_id.validators_info[0].tss_account =
        subxtAccountId32(AccountKeyring::Dave.into());

    let test_user_failed_tss_account = submit_transaction_requests(
        validator_ips_and_keys.clone(),
        generic_msg_bad_account_id,
        one,
    )
    .await;

    for res in test_user_failed_tss_account {
        let res = res.unwrap();
        assert_eq!(res.status(), 500);
        assert_eq!(res.text().await.unwrap(), "Invalid Signer: Invalid Signer in Signing group");
    }

    // Test a transcation which does not pass constaints
    generic_msg.transaction_request = hex::encode(MESSAGE_SHOULD_FAIL);
    generic_msg.timestamp = SystemTime::now();

    let test_user_failed_programs_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_user_failed_programs_res {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Runtime error: Runtime(Error::Evaluation(\"Length of data is too short.\"))"
        );
    }

    let mock_client = reqwest::Client::new();
    // fails verification tests
    // wrong key for wrong validator
    let server_public_key = PublicKey::from(X25519_PUBLIC_KEYS[1]);
    let failed_signed_message = SignedMessage::new(
        &one.pair(),
        &Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        &server_public_key,
    )
    .unwrap();
    let failed_res = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&failed_signed_message).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(failed_res.status(), 500);
    assert_eq!(
        failed_res.text().await.unwrap(),
        "Validation error: ChaCha20 decryption error: aead::Error"
    );

    let sig: [u8; 64] = [0; 64];
    let slice: [u8; 32] = [0; 32];
    let nonce: [u8; 12] = [0; 12];

    let user_input_bad = SignedMessage::new_test(
        Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        sr25519::Signature::from_raw(sig),
        one.pair().public().into(),
        slice,
        slice,
        nonce,
    );

    let failed_sign = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&user_input_bad).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(failed_sign.status(), 500);
    assert_eq!(failed_sign.text().await.unwrap(), "Invalid Signature: Invalid signature.");

    // checks that sig not needed with public key visibility
    let user_input_bad = SignedMessage::new_test(
        Bytes(serde_json::to_vec(&generic_msg.clone()).unwrap()),
        sr25519::Signature::from_raw(sig),
        AccountKeyring::Dave.pair().public().into(),
        slice,
        slice,
        nonce,
    );

    let failed_sign = mock_client
        .post("http://127.0.0.1:3001/user/sign_tx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&user_input_bad).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(failed_sign.status(), 500);
    // fails lower down in stack because no sig needed on pub account
    // fails when tries to decode the nonsense message
    assert_ne!(failed_sign.text().await.unwrap(), "Invalid Signature: Invalid signature.");
    clean_tests();
}

/// Test demonstrating registering with private key visibility, where the user participates in DKG
/// and holds a keyshare.
#[tokio::test]
#[serial]
async fn test_register_with_private_key_visibility() {
    clean_tests();

    let one = AccountKeyring::One;
    let program_modification_account = AccountKeyring::Charlie;

    let (validator_ips, _validator_ids, _users_keyshare_option) =
        spawn_testing_validators(None, false).await;
    let substrate_context = test_context_stationary().await;
    let api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();
    let block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number + 1;

    let (one_keypair, one_x25519_sk) = keyring_to_subxt_signer_and_x25519(&one);
    let x25519_public_key = PublicKey::from(&one_x25519_sk).to_bytes();

    put_register_request_on_chain(
        &api,
        &one,
        program_modification_account.to_account_id().into(),
        KeyVisibility::Private(x25519_public_key),
    )
    .await;
    run_to_block(&rpc, block_number + 1).await;

    // Simulate the propagation pallet making a `user/new` request to the second validator
    // as we only have one chain node running
    let onchain_user_request = {
        // Since we only have two validators we use both of them, but if we had more we would
        // need to select them using same method as the chain does (based on block number)
        let validators_info: Vec<entropy_shared::ValidatorInfo> = validator_ips
            .iter()
            .enumerate()
            .map(|(i, ip)| entropy_shared::ValidatorInfo {
                ip_address: ip.as_bytes().to_vec(),
                x25519_public_key: X25519_PUBLIC_KEYS[i],
                tss_account: TSS_ACCOUNTS[i].clone().encode(),
            })
            .collect();
        OcwMessage { sig_request_accounts: vec![one.encode()], block_number, validators_info }
    };

    let client = reqwest::Client::new();
    let validators_info: Vec<ValidatorInfo> = validator_ips
        .iter()
        .enumerate()
        .map(|(i, ip)| ValidatorInfo {
            ip_address: SocketAddr::from_str(ip).unwrap(),
            x25519_public_key: X25519_PUBLIC_KEYS[i],
            tss_account: TSS_ACCOUNTS[i].clone(),
        })
        .collect();

    // Call the `user/new` endpoint, and connect and participate in the protocol
    let (new_user_response_result, keyshare_result) = future::join(
        client
            .post("http://127.0.0.1:3002/user/new")
            .body(onchain_user_request.clone().encode())
            .send(),
        user_participates_in_dkg_protocol(validators_info.clone(), &one_keypair, &one_x25519_sk),
    )
    .await;

    let response = new_user_response_result.unwrap();
    assert_eq!(response.text().await.unwrap(), "");

    assert!(keyshare_result.is_ok());
}

pub async fn verify_signature(
    test_user_res: Vec<Result<reqwest::Response, reqwest::Error>>,
    message_should_succeed_hash: [u8; 32],
    keyshare_option: Option<KeyShare<KeyParams>>,
) {
    let mut i = 0;
    for res in test_user_res {
        let mut res = res.unwrap();

        assert_eq!(res.status(), 200);
        let chunk = res.chunk().await.unwrap().unwrap();
        let signing_result: Result<(String, Signature), String> =
            serde_json::from_slice(&chunk).unwrap();
        assert_eq!(signing_result.clone().unwrap().0.len(), 88);
        let mut decoded_sig = base64::decode(signing_result.clone().unwrap().0).unwrap();
        let recovery_digit = decoded_sig.pop().unwrap();
        let signature = k256Signature::from_slice(&decoded_sig).unwrap();
        let recover_id = RecoveryId::from_byte(recovery_digit).unwrap();
        let recovery_key_from_sig = VerifyingKey::recover_from_prehash(
            &message_should_succeed_hash,
            &signature,
            recover_id,
        )
        .unwrap();
        assert_eq!(keyshare_option.clone().unwrap().verifying_key(), recovery_key_from_sig);
        let mnemonic = if i == 0 { DEFAULT_MNEMONIC } else { DEFAULT_BOB_MNEMONIC };
        let sk = <sr25519::Pair as Pair>::from_string(mnemonic, None).unwrap();
        let sig_recovery = <sr25519::Pair as Pair>::verify(
            &signing_result.clone().unwrap().1,
            base64::decode(signing_result.unwrap().0).unwrap(),
            &sr25519::Public(sk.public().0),
        );
        assert!(sig_recovery);
        i += 1;
    }
}
