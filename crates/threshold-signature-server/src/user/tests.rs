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

use std::{
    env, fs,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use axum::http::StatusCode;
use bip39::{Language, Mnemonic};
use entropy_kvdb::{
    clean_tests,
    encrypted_sled::PasswordMethod,
    kv_manager::{helpers::deserialize as keyshare_deserialize, value::KvManager},
};
use entropy_programs_runtime::{Runtime, SignatureRequest};
use entropy_protocol::{
    protocol_transport::{noise::noise_handshake_initiator, SubscribeMessage, WsConnection},
    user::{user_participates_in_dkg_protocol, user_participates_in_signing_protocol},
    KeyParams, PartyId, SessionId, SigningSessionInfo, ValidatorInfo,
};
use entropy_shared::{
    HashingAlgorithm, KeyVisibility, OcwMessageDkg, DAVE_VERIFYING_KEY, DEFAULT_VERIFYING_KEY,
    DEFAULT_VERIFYING_KEY_NOT_REGISTERED, EVE_VERIFYING_KEY, FERDIE_VERIFYING_KEY,
};
use entropy_testing_utils::{
    chain_api::{
        entropy::runtime_types::bounded_collections::bounded_vec::BoundedVec as OtherBoundedVec,
        entropy::runtime_types::pallet_registry::pallet::ProgramInstance as OtherProgramInstance,
    },
    constants::{
        ALICE_STASH_ADDRESS, AUXILARY_DATA_SHOULD_FAIL, AUXILARY_DATA_SHOULD_SUCCEED,
        PREIMAGE_SHOULD_FAIL, PREIMAGE_SHOULD_SUCCEED, TEST_BASIC_TRANSACTION,
        TEST_INFINITE_LOOP_BYTECODE, TEST_PROGRAM_CUSTOM_HASH, TEST_PROGRAM_WASM_BYTECODE,
        TSS_ACCOUNTS, X25519_PUBLIC_KEYS,
    },
    substrate_context::{
        test_context_stationary, test_node_process_testing_state, testing_context,
        SubstrateTestingContext,
    },
    test_client::{store_program, update_programs},
};
use futures::{
    future::{self, join_all},
    join, Future, SinkExt, StreamExt,
};
use hex_literal::hex;
use more_asserts as ma;
use parity_scale_codec::{Decode, DecodeAll, Encode};
use serde::{Deserialize, Serialize};
use serial_test::serial;
use sp_core::{crypto::Ss58Codec, Pair as OtherPair, H160};
use sp_keyring::{AccountKeyring, Sr25519Keyring};
use subxt::{
    backend::legacy::LegacyRpcMethods,
    events::EventsClient,
    ext::{
        sp_core::{sr25519, sr25519::Signature, Bytes, Pair},
        sp_runtime::AccountId32,
    },
    tx::PairSigner,
    utils::{AccountId32 as subxtAccountId32, Static, H256},
    Config, OnlineClient,
};
use synedrion::{
    k256::ecdsa::{RecoveryId, Signature as k256Signature, VerifyingKey},
    KeyShare,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use x25519_dalek::{PublicKey, StaticSecret};

use super::UserInputPartyInfo;
use crate::{
    chain_api::{
        entropy, entropy::runtime_types::bounded_collections::bounded_vec::BoundedVec,
        entropy::runtime_types::pallet_registry::pallet::ProgramInstance, get_api, get_rpc,
        EntropyConfig,
    },
    get_signer,
    helpers::{
        launch::{
            load_kv_store, setup_mnemonic, Configuration, ValidatorName, DEFAULT_BOB_MNEMONIC,
            DEFAULT_CHARLIE_MNEMONIC, DEFAULT_ENDPOINT, DEFAULT_MNEMONIC,
        },
        signing::Hasher,
        substrate::{
            get_subgroup, query_chain, return_all_addresses_of_subgroup, submit_transaction,
        },
        tests::{
            check_has_confirmation, check_if_confirmation, create_clients, initialize_test_logger,
            remove_program, run_to_block, setup_client, spawn_testing_validators,
        },
        user::{compute_hash, send_key},
    },
    new_user,
    r#unsafe::api::UnsafeQuery,
    signing_client::ListenerState,
    user::{
        api::{
            check_hash_pointer_out_of_bounds, confirm_registered, get_current_subgroup_signers,
            increment_or_wipe_request_limit, recover_key, request_limit_check, request_limit_key,
            RequestLimitStorage, UserRegistrationInfo, UserSignatureRequest,
        },
        UserErr,
    },
    validation::{derive_static_secret, mnemonic_to_pair, new_mnemonic, SignedMessage},
    validator::api::get_random_server_info,
};

#[tokio::test]
#[serial]
async fn test_get_signer_does_not_throw_err() {
    initialize_test_logger().await;
    clean_tests();

    let kv_store = load_kv_store(&None, None).await;
    let mnemonic = setup_mnemonic(&kv_store, &None).await;
    assert!(mnemonic.is_ok());
    get_signer(&kv_store).await.unwrap();
    clean_tests();
}
#[tokio::test]
#[serial]
async fn test_sign_tx_no_chain() {
    initialize_test_logger().await;
    clean_tests();

    let one = AccountKeyring::Dave;
    let two = AccountKeyring::Two;

    let (validator_ips, _validator_ids, keyshare_option) =
        spawn_testing_validators(Some(DAVE_VERIFYING_KEY.to_vec()), false, false).await;
    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();

    let program_hash = store_program(
        &entropy_api,
        &rpc,
        &two.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    let validators_info = vec![
        ValidatorInfo {
            ip_address: "localhost:3001".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone(),
        },
        ValidatorInfo {
            ip_address: "127.0.0.1:3002".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let message_hash = Hasher::keccak(PREIMAGE_SHOULD_SUCCEED);
    let signature_request_account = subxtAccountId32(one.pair().public().0);
    let session_id = SessionId::Sign(SigningSessionInfo {
        signature_verifying_key: DAVE_VERIFYING_KEY.to_vec(),
        message_hash,
        request_author: signature_request_account.clone(),
    });

    let mut generic_msg = UserSignatureRequest {
        message: hex::encode(PREIMAGE_SHOULD_SUCCEED),
        auxilary_data: Some(vec![
            Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED)),
            Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED)),
        ]),
        validators_info,
        timestamp: SystemTime::now(),
        hash: HashingAlgorithm::Keccak,
        signature_verifying_key: DAVE_VERIFYING_KEY.to_vec(),
    };

    let validator_ips_and_keys = vec![
        (validator_ips[0].clone(), X25519_PUBLIC_KEYS[0]),
        (validator_ips[1].clone(), X25519_PUBLIC_KEYS[1]),
    ];

    generic_msg.timestamp = SystemTime::now();
    // test points to no program
    let test_no_program =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_no_program {
        assert_eq!(res.unwrap().text().await.unwrap(), "No program pointer defined for account");
    }
    update_programs(
        &entropy_api,
        &rpc,
        DAVE_VERIFYING_KEY.to_vec(),
        &one.pair(),
        OtherBoundedVec(vec![
            OtherProgramInstance { program_pointer: program_hash, program_config: vec![] },
            OtherProgramInstance { program_pointer: program_hash, program_config: vec![] },
        ]),
    )
    .await
    .unwrap();

    generic_msg.timestamp = SystemTime::now();
    let test_user_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    verify_signature(test_user_res, message_hash, keyshare_option.clone()).await;
    let mock_client = reqwest::Client::new();
    // check request limiter increases
    let unsafe_get =
        UnsafeQuery::new(request_limit_key(hex::encode(DAVE_VERIFYING_KEY.to_vec())), vec![])
            .to_json();

    // check get key before registration to see if key gets replaced
    let get_response = mock_client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(unsafe_get.clone())
        .send()
        .await
        .unwrap();
    let serialized_request_amount = get_response.text().await.unwrap();

    let request_info: RequestLimitStorage =
        RequestLimitStorage::decode(&mut serialized_request_amount.as_ref()).unwrap();
    assert_eq!(request_info.request_amount, 1);
    generic_msg.timestamp = SystemTime::now();
    generic_msg.validators_info = generic_msg.validators_info.into_iter().rev().collect::<Vec<_>>();
    let test_user_res_order =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    verify_signature(test_user_res_order, message_hash, keyshare_option.clone()).await;

    generic_msg.timestamp = SystemTime::now();
    generic_msg.signature_verifying_key = DEFAULT_VERIFYING_KEY_NOT_REGISTERED.to_vec();
    let test_user_res_not_registered =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), two).await;

    for res in test_user_res_not_registered {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Chain Fetch: Not Registering error: Register Onchain first"
        );
    }

    // Test attempting to connect over ws by someone who is not in the signing group
    let validator_ip_and_key = validator_ips_and_keys[0].clone();
    let connection_attempt_handle = tokio::spawn(async move {
        // Wait for the "user" to submit the signing request
        tokio::time::sleep(Duration::from_millis(500)).await;
        let ws_endpoint = format!("ws://{}/ws", validator_ip_and_key.0);
        let (ws_stream, _response) = connect_async(ws_endpoint).await.unwrap();

        let ferdie_pair = AccountKeyring::Ferdie.pair();
        let ferdie_x25519_sk = derive_static_secret(&ferdie_pair);

        // create a SubscribeMessage from a party who is not in the signing commitee
        let subscribe_message_vec =
            bincode::serialize(&SubscribeMessage::new(session_id, &ferdie_pair).unwrap()).unwrap();

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
            bincode::deserialize(&response_message).unwrap();

        assert_eq!(Err("NoListener(\"no listener\")".to_string()), subscribe_response);
        // The stream should not continue to send messages
        // returns true if this part of the test passes
        encrypted_connection.recv().await.is_err()
    });

    generic_msg.timestamp = SystemTime::now();
    generic_msg.signature_verifying_key = DAVE_VERIFYING_KEY.to_vec().to_vec();
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

    // Now, test a signature request that should fail
    // The test program is written to fail when `auxilary_data` is `None`
    generic_msg.auxilary_data = None;
    generic_msg.timestamp = SystemTime::now();

    let test_user_failed_programs_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_user_failed_programs_res {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Runtime error: Runtime(Error::Evaluation(\"This program requires that `auxilary_data` be `Some`.\"))"
        );
    }

    // The test program is written to fail when `auxilary_data` is `None` but only on the second program
    generic_msg.auxilary_data = Some(vec![Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED))]);
    generic_msg.timestamp = SystemTime::now();

    let test_user_failed_aux_data =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_user_failed_aux_data {
        assert_eq!(res.unwrap().text().await.unwrap(), "Auxilary data is mismatched");
    }

    generic_msg.timestamp = SystemTime::now();
    generic_msg.hash = HashingAlgorithm::Custom(3);
    let test_user_custom_hash_out_of_bounds =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), two).await;

    for res in test_user_custom_hash_out_of_bounds {
        assert_eq!(res.unwrap().text().await.unwrap(), "Custom hash choice out of bounds");
    }

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

    let request_limit_query = entropy::storage().parameters().request_limit();
    let request_limit =
        query_chain(&entropy_api, &rpc, request_limit_query, None).await.unwrap().unwrap();

    // test request limit reached

    // gets current blocknumber, potential race condition run to block + 1
    // to reset block and give us 6 seconds to hit rate limit
    let block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number;
    run_to_block(&rpc, block_number + 1).await;
    let unsafe_put = UnsafeQuery::new(
        request_limit_key(hex::encode(DAVE_VERIFYING_KEY.to_vec())),
        RequestLimitStorage { request_amount: request_limit + 1, block_number: block_number + 1 }
            .encode(),
    )
    .to_json();

    let _ = mock_client
        .post("http://127.0.0.1:3001/unsafe/put")
        .header("Content-Type", "application/json")
        .body(unsafe_put.clone())
        .send()
        .await
        .unwrap();
    let _ = mock_client
        .post("http://127.0.0.1:3002/unsafe/put")
        .header("Content-Type", "application/json")
        .body(unsafe_put.clone())
        .send()
        .await
        .unwrap();

    let test_user_failed_request_limit =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_user_failed_request_limit {
        assert_eq!(res.unwrap().text().await.unwrap(), "Too many requests - wait a block");
    }
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_program_with_config() {
    initialize_test_logger().await;
    clean_tests();

    let one = AccountKeyring::Dave;
    let two = AccountKeyring::Two;

    let (validator_ips, _validator_ids, keyshare_option) =
        spawn_testing_validators(Some(DAVE_VERIFYING_KEY.to_vec().clone()), false, false).await;
    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();

    let program_hash = store_program(
        &entropy_api,
        &rpc,
        &two.pair(),
        TEST_BASIC_TRANSACTION.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    let validators_info = vec![
        ValidatorInfo {
            ip_address: "localhost:3001".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone(),
        },
        ValidatorInfo {
            ip_address: "127.0.0.1:3002".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    // this message is an ethereum tx rlp encoded with a proper allow listed address
    let message = "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080";

    let message_hash = Hasher::keccak(message.as_bytes());

    let mut generic_msg = UserSignatureRequest {
        message: hex::encode(message).to_string(),
        auxilary_data: Some(vec![
            Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED)),
            Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED)),
        ]),
        validators_info,
        timestamp: SystemTime::now(),
        hash: HashingAlgorithm::Keccak,
        signature_verifying_key: DAVE_VERIFYING_KEY.to_vec(),
    };

    let validator_ips_and_keys = vec![
        (validator_ips[0].clone(), X25519_PUBLIC_KEYS[0]),
        (validator_ips[1].clone(), X25519_PUBLIC_KEYS[1]),
    ];
    let config = r#"
        {
            "allowlisted_addresses": [
                "772b9a9e8aa1c9db861c6611a82d251db4fac990"
            ]
        }
    "#
    .as_bytes();

    update_programs(
        &entropy_api,
        &rpc,
        DAVE_VERIFYING_KEY.to_vec(),
        &one.pair(),
        OtherBoundedVec(vec![
            OtherProgramInstance { program_pointer: program_hash, program_config: config.to_vec() },
            OtherProgramInstance { program_pointer: program_hash, program_config: config.to_vec() },
        ]),
    )
    .await
    .unwrap();

    generic_msg.timestamp = SystemTime::now();
    let test_user_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    verify_signature(test_user_res, message_hash, keyshare_option.clone()).await;
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_fail_signing_group() {
    initialize_test_logger().await;
    clean_tests();

    let dave = AccountKeyring::Dave;
    let eve = AccountKeyring::Eve;
    let _ = spawn_testing_validators(None, false, false).await;

    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();

    let validators_info = vec![
        ValidatorInfo {
            ip_address: "127.0.0.1:3001".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: hex!["a664add5dfaca1dd560b949b5699b5f0c3c1df3a2ea77ceb0eeb4f77cc3ade04"]
                .into(),
        },
        ValidatorInfo {
            ip_address: "127.0.0.1:3002".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let program_hash = store_program(
        &entropy_api,
        &rpc,
        &eve.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    update_programs(
        &entropy_api,
        &rpc,
        DAVE_VERIFYING_KEY.to_vec(),
        &dave.pair(),
        OtherBoundedVec(vec![OtherProgramInstance {
            program_pointer: program_hash,
            program_config: vec![],
        }]),
    )
    .await
    .unwrap();

    let generic_msg = UserSignatureRequest {
        message: hex::encode(PREIMAGE_SHOULD_SUCCEED),
        auxilary_data: Some(vec![Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED))]),
        validators_info,
        timestamp: SystemTime::now(),
        hash: HashingAlgorithm::Keccak,
        signature_verifying_key: DAVE_VERIFYING_KEY.to_vec(),
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
    initialize_test_logger().await;
    clean_tests();

    let alice = AccountKeyring::Alice;
    let alice_program = AccountKeyring::Charlie;
    let program_manager = AccountKeyring::Dave;

    let cxt = test_context_stationary().await;
    let (_validator_ips, _validator_ids, _) =
        spawn_testing_validators(Some(DEFAULT_VERIFYING_KEY.to_vec()), false, false).await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let client = reqwest::Client::new();

    let program_hash = store_program(
        &api,
        &rpc,
        &program_manager.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

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
        OcwMessageDkg { sig_request_accounts: vec![alice.encode()], block_number, validators_info };

    put_register_request_on_chain(
        &api,
        &rpc,
        &alice,
        alice_program.to_account_id().into(),
        KeyVisibility::Public,
        BoundedVec(vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }]),
    )
    .await;

    run_to_block(&rpc, block_number + 1).await;

    // succeeds
    let user_registration_response = client
        .post("http://127.0.0.1:3002/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(user_registration_response.text().await.unwrap(), "");

    let mut new_verifying_key = vec![];
    // wait for registered event check that key exists in kvdb
    for _ in 0..45 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let block_hash = rpc.chain_get_block_hash(None).await.unwrap();
        let events = EventsClient::new(api.clone()).at(block_hash.unwrap()).await.unwrap();
        let registered_event = events.find::<entropy::registry::events::AccountRegistered>();
        for event in registered_event.flatten() {
            let registered_query = entropy::storage().registry().registered(&event.1);
            let query_registered_status =
                query_chain(&api, &rpc, registered_query, block_hash).await;
            if query_registered_status.unwrap().is_some() {
                if event.0 == alice.to_account_id().into() {
                    new_verifying_key = event.1 .0;
                    break;
                }
            }
        }
    }

    let get_query =
        UnsafeQuery::new(hex::encode(new_verifying_key.to_vec()), [].to_vec()).to_json();
    // check get key before registration to see if key gets replaced
    let response_key = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();
    // check to make sure keyshare is correct
    let key_share: Option<KeyShare<KeyParams>> =
        entropy_kvdb::kv_manager::helpers::deserialize(&response_key.bytes().await.unwrap());
    assert_eq!(key_share.is_some(), true);

    // fails repeated data
    let response_repeated_data = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_repeated_data.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_repeated_data.text().await.unwrap(), "Data is repeated");

    run_to_block(&rpc, block_number + 3).await;
    onchain_user_request.block_number = block_number + 1;
    // fails stale data
    let response_stale = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_stale.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_stale.text().await.unwrap(), "Data is stale");

    block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number + 1;
    put_register_request_on_chain(
        &api,
        &rpc,
        &alice_program,
        alice_program.to_account_id().into(),
        KeyVisibility::Public,
        BoundedVec(vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }]),
    )
    .await;
    onchain_user_request.block_number = block_number;
    run_to_block(&rpc, block_number + 1).await;

    // fails not verified data
    let response_not_verified = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_not_verified.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_not_verified.text().await.unwrap(), "Data is not verifiable");

    onchain_user_request.validators_info[0].tss_account = TSS_ACCOUNTS[1].clone().encode();
    // fails not in validator group data
    let response_not_validator = client
        .post("http://127.0.0.1:3001/user/new")
        .body(onchain_user_request.clone().encode())
        .send()
        .await
        .unwrap();

    assert_eq!(response_not_validator.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_not_validator.text().await.unwrap(),
        "Invalid Signer: Invalid Signer in Signing group"
    );

    check_if_confirmation(&api, &rpc, &alice.pair(), new_verifying_key).await;
    // TODO check if key is in other subgroup member
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_return_addresses_of_subgroup() {
    initialize_test_logger().await;

    let cxt = test_context_stationary().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let result = return_all_addresses_of_subgroup(&api, &rpc, 0u8).await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_send_and_receive_keys() {
    initialize_test_logger().await;
    clean_tests();

    let alice = AccountKeyring::Alice;
    let program_manager = AccountKeyring::Dave;

    let cxt = test_context_stationary().await;
    setup_client().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let share = {
        let share = &KeyShare::<KeyParams>::new_centralized(&mut rand_core::OsRng, 2, None)[0];
        entropy_kvdb::kv_manager::helpers::serialize(&share).unwrap()
    };

    let user_registration_info = UserRegistrationInfo {
        key: alice.to_account_id().to_string(),
        value: share.clone(),
        proactive_refresh: false,
    };

    let p_alice = <sr25519::Pair as Pair>::from_string(DEFAULT_MNEMONIC, None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);

    // First try sending a keyshare for a user who is not registering - should fail
    let result = send_key(
        &api,
        &rpc,
        &alice.to_account_id().into(),
        &mut vec![ALICE_STASH_ADDRESS.clone(), alice.to_account_id().into()],
        user_registration_info.clone(),
        &signer_alice,
    )
    .await;

    if let Err(UserErr::KeyShareRejected(error_message)) = result {
        assert_eq!(
            error_message,
            "Not Registering error: Provided account ID not from a registering user".to_string()
        );
    } else {
        panic!("Should give not registering error");
    }

    // The happy path - the user is in a registering state - should succeed
    let program_hash = store_program(
        &api,
        &rpc,
        &program_manager.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    put_register_request_on_chain(
        &api,
        &rpc,
        &alice.clone(),
        alice.to_account_id().into(),
        KeyVisibility::Public,
        BoundedVec(vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }]),
    )
    .await;

    // sends key to alice validator, while filtering out own key
    send_key(
        &api,
        &rpc,
        &alice.to_account_id().into(),
        &mut vec![ALICE_STASH_ADDRESS.clone(), alice.to_account_id().into()],
        user_registration_info.clone(),
        &signer_alice,
    )
    .await
    .unwrap();

    let get_query = UnsafeQuery::new(user_registration_info.key.clone(), vec![]).to_json();

    let client = reqwest::Client::new();
    // check alice has new key
    let response_new_key = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response_new_key.bytes().await.unwrap(), &user_registration_info.value.clone());

    // A keyshare can be overwritten when the user is still in a registering state
    let server_public_key = PublicKey::from(X25519_PUBLIC_KEYS[0]);

    let some_other_share = {
        let share = &KeyShare::<KeyParams>::new_centralized(&mut rand_core::OsRng, 2, None)[0];
        entropy_kvdb::kv_manager::helpers::serialize(&share).unwrap()
    };

    let user_registration_info_overwrite = UserRegistrationInfo {
        key: alice.to_account_id().to_string(),
        value: some_other_share.clone(),
        proactive_refresh: false,
    };

    let signed_message = SignedMessage::new(
        signer_alice.signer(),
        &Bytes(serde_json::to_vec(&user_registration_info_overwrite).unwrap()),
        &server_public_key,
    )
    .unwrap()
    .to_json()
    .unwrap();

    let response_overwrites_key = client
        .post("http://127.0.0.1:3001/user/receive_key")
        .header("Content-Type", "application/json")
        .body(signed_message.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response_overwrites_key.status(), StatusCode::OK);
    assert_eq!(response_overwrites_key.text().await.unwrap(), "");

    // Check that the key has been successfully overwritten
    let get_query = UnsafeQuery::new(user_registration_info.key.clone(), vec![]).to_json();
    let response_new_key = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response_new_key.bytes().await.unwrap(), &some_other_share);

    // Try writing a 'forbidden key' - should fail
    let user_registration_info_forbidden = UserRegistrationInfo {
        key: "MNEMONIC".to_string(),
        value: share.clone(),
        proactive_refresh: false,
    };

    let signed_message = SignedMessage::new(
        signer_alice.signer(),
        &Bytes(serde_json::to_vec(&user_registration_info_forbidden).unwrap()),
        &server_public_key,
    )
    .unwrap()
    .to_json()
    .unwrap();

    let response_overwrites_key = client
        .post("http://127.0.0.1:3001/user/receive_key")
        .header("Content-Type", "application/json")
        .body(signed_message.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response_overwrites_key.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_overwrites_key.text().await.unwrap(), "The given key is forbidden");

    // Try sending a badly formed keyshare - should fail
    let user_registration_info_bad_keyshare = UserRegistrationInfo {
        key: alice.to_account_id().to_string(),
        value: b"This will not deserialize to KeyShare<KeyParams>".to_vec(),
        proactive_refresh: false,
    };

    let signed_message = SignedMessage::new(
        signer_alice.signer(),
        &Bytes(serde_json::to_vec(&user_registration_info_bad_keyshare).unwrap()),
        &server_public_key,
    )
    .unwrap()
    .to_json()
    .unwrap();

    let response_overwrites_key = client
        .post("http://127.0.0.1:3001/user/receive_key")
        .header("Content-Type", "application/json")
        .body(signed_message.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response_overwrites_key.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_overwrites_key.text().await.unwrap(),
        "Input Validation error: Not a valid keyshare"
    );

    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_recover_key() {
    initialize_test_logger().await;
    clean_tests();

    let cxt = test_node_process_testing_state(false).await;
    setup_client().await;
    let (_, bob_kv) =
        create_clients("validator2".to_string(), vec![], vec![], &Some(ValidatorName::Bob)).await;

    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();
    let unsafe_query = UnsafeQuery::new("key".to_string(), vec![10]);
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
    recover_key(&api, &rpc, &bob_kv, &signer_alice, unsafe_query.key.clone()).await.unwrap();

    let value = bob_kv.kv().get(&unsafe_query.key).await.unwrap();
    assert_eq!(value, unsafe_query.value);
    clean_tests();
}

pub async fn put_register_request_on_chain(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    sig_req_keyring: &Sr25519Keyring,
    program_modification_account: subxtAccountId32,
    key_visibility: KeyVisibility,
    program_instance: BoundedVec<ProgramInstance>,
) {
    let sig_req_account =
        PairSigner::<EntropyConfig, sp_core::sr25519::Pair>::new(sig_req_keyring.pair());

    let registering_tx = entropy::tx().registry().register(
        program_modification_account,
        Static(key_visibility),
        program_instance,
    );
    submit_transaction(api, rpc, &sig_req_account, &registering_tx, None).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_sign_tx_user_participates() {
    initialize_test_logger().await;
    clean_tests();

    let one = AccountKeyring::Eve;
    let two = AccountKeyring::Two;

    let (validator_ips, _validator_ids, users_keyshare_option) =
        spawn_testing_validators(Some(EVE_VERIFYING_KEY.to_vec()), true, true).await;
    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();
    let verifying_key = users_keyshare_option
        .clone()
        .unwrap()
        .verifying_key()
        .to_encoded_point(true)
        .as_bytes()
        .to_vec();

    let program_hash = store_program(
        &entropy_api,
        &rpc,
        &two.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    update_programs(
        &entropy_api,
        &rpc,
        verifying_key.clone(),
        &one.pair(),
        OtherBoundedVec(vec![OtherProgramInstance {
            program_pointer: program_hash,
            program_config: vec![],
        }]),
    )
    .await
    .unwrap();

    let validators_info = vec![
        ValidatorInfo {
            ip_address: "127.0.0.1:3001".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone(),
        },
        ValidatorInfo {
            ip_address: "127.0.0.1:3002".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let encoded_transaction_request: String = hex::encode(PREIMAGE_SHOULD_SUCCEED);
    let message_should_succeed_hash = Hasher::keccak(PREIMAGE_SHOULD_SUCCEED);

    let signature_request_account = subxtAccountId32(one.pair().public().0);
    let session_id = SessionId::Sign(SigningSessionInfo {
        signature_verifying_key: verifying_key.clone(),
        message_hash: message_should_succeed_hash,
        request_author: signature_request_account.clone(),
    });

    let mut generic_msg = UserSignatureRequest {
        message: encoded_transaction_request.clone(),
        auxilary_data: Some(vec![Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED))]),
        validators_info: validators_info.clone(),
        timestamp: SystemTime::now(),
        hash: HashingAlgorithm::Keccak,
        signature_verifying_key: verifying_key.clone(),
    };

    let validator_ips_and_keys = vec![
        (validator_ips[0].clone(), X25519_PUBLIC_KEYS[0]),
        (validator_ips[1].clone(), X25519_PUBLIC_KEYS[1]),
    ];
    generic_msg.timestamp = SystemTime::now();

    // Submit transaction requests, and connect and participate in signing
    let (test_user_res, sig_result) = future::join(
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one),
        user_participates_in_signing_protocol(
            &users_keyshare_option.clone().unwrap(),
            validators_info.clone(),
            &one.pair(),
            message_should_succeed_hash,
        ),
    )
    .await;

    let signature_base64 = base64::encode(sig_result.unwrap().to_rsv_bytes());
    assert_eq!(signature_base64.len(), 88);

    verify_signature(test_user_res, message_should_succeed_hash, users_keyshare_option.clone())
        .await;

    generic_msg.timestamp = SystemTime::now();
    generic_msg.signature_verifying_key = DEFAULT_VERIFYING_KEY_NOT_REGISTERED.to_vec();

    // test failing cases
    let test_user_res_not_registered =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), two).await;

    for res in test_user_res_not_registered {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Chain Fetch: Not Registering error: Register Onchain first"
        );
    }

    generic_msg.timestamp = SystemTime::now();
    generic_msg.signature_verifying_key = verifying_key;
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

        let ferdie_pair = AccountKeyring::Ferdie.pair();
        let ferdie_x25519_sk = derive_static_secret(&ferdie_pair);

        // create a SubscribeMessage from a party who is not in the signing commitee
        let subscribe_message_vec =
            bincode::serialize(&SubscribeMessage::new(session_id, &ferdie_pair).unwrap()).unwrap();

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
            bincode::deserialize(&response_message).unwrap();

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

    // Now, test a signature request that should fail
    // The test program is written to fail when `auxilary_data` is `None`
    generic_msg.auxilary_data = None;
    generic_msg.timestamp = SystemTime::now();

    let test_user_failed_programs_res =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;

    for res in test_user_failed_programs_res {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "Runtime error: Runtime(Error::Evaluation(\"This program requires that `auxilary_data` be `Some`.\"))"
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

    clean_tests();
}

/// Test demonstrating registering with private key visibility, where the user participates in DKG
/// and holds a keyshare.
#[tokio::test]
#[serial]
async fn test_register_with_private_key_visibility() {
    initialize_test_logger().await;
    clean_tests();

    let one = AccountKeyring::One;
    let program_modification_account = AccountKeyring::Charlie;
    let program_manager = AccountKeyring::Dave;

    let (validator_ips, _validator_ids, _users_keyshare_option) =
        spawn_testing_validators(None, false, false).await;
    let substrate_context = test_context_stationary().await;
    let api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();

    let program_hash = store_program(
        &api,
        &rpc,
        &program_manager.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    let block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number + 1;

    let one_x25519_sk = derive_static_secret(&one.pair());
    let x25519_public_key = PublicKey::from(&one_x25519_sk).to_bytes();

    put_register_request_on_chain(
        &api,
        &rpc,
        &one,
        program_modification_account.to_account_id().into(),
        KeyVisibility::Private(x25519_public_key),
        BoundedVec(vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }]),
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
        OcwMessageDkg { sig_request_accounts: vec![one.encode()], block_number, validators_info }
    };

    let client = reqwest::Client::new();
    let validators_info: Vec<ValidatorInfo> = validator_ips
        .iter()
        .enumerate()
        .map(|(i, ip)| ValidatorInfo {
            ip_address: ip.to_string(),
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
        user_participates_in_dkg_protocol(validators_info.clone(), &one.pair(), block_number),
    )
    .await;

    let response = new_user_response_result.unwrap();
    assert_eq!(response.text().await.unwrap(), "");

    assert!(keyshare_result.is_ok());
    clean_tests();
}

#[tokio::test]
async fn test_compute_hash() {
    initialize_test_logger().await;
    clean_tests();
    let one = AccountKeyring::Dave;
    let substrate_context = testing_context().await;
    let api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();

    let mut runtime = Runtime::default();
    let program_hash =
        store_program(&api, &rpc, &one.pair(), TEST_PROGRAM_CUSTOM_HASH.to_owned(), vec![], vec![])
            .await
            .unwrap();

    let message_hash = compute_hash(
        &api,
        &rpc,
        &HashingAlgorithm::Custom(0),
        &mut runtime,
        &vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }],
        PREIMAGE_SHOULD_SUCCEED,
    )
    .await
    .unwrap();
    // custom hash program uses blake 3 to hash
    let expected_hash = blake3::hash(PREIMAGE_SHOULD_SUCCEED).as_bytes().to_vec();
    assert_eq!(message_hash.to_vec(), expected_hash);
}

#[tokio::test]
async fn test_check_hash_pointer_out_of_bounds() {
    assert!(check_hash_pointer_out_of_bounds(&HashingAlgorithm::Custom(2), 5).is_ok());
    assert_eq!(
        check_hash_pointer_out_of_bounds(&HashingAlgorithm::Custom(5), 5).unwrap_err().to_string(),
        "Custom hash choice out of bounds".to_string()
    );
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

#[tokio::test]
#[serial]
async fn test_fail_infinite_program() {
    initialize_test_logger().await;
    clean_tests();

    let one = AccountKeyring::Dave;
    let two = AccountKeyring::Two;

    let (validator_ips, _validator_ids, _) =
        spawn_testing_validators(Some(DAVE_VERIFYING_KEY.to_vec()), false, false).await;
    let substrate_context = test_context_stationary().await;
    let entropy_api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();

    let program_hash = store_program(
        &entropy_api,
        &rpc,
        &two.pair(),
        TEST_INFINITE_LOOP_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    update_programs(
        &entropy_api,
        &rpc,
        DAVE_VERIFYING_KEY.to_vec(),
        &one.pair(),
        OtherBoundedVec(vec![OtherProgramInstance {
            program_pointer: program_hash,
            program_config: vec![],
        }]),
    )
    .await
    .unwrap();

    let validators_info = vec![
        ValidatorInfo {
            ip_address: "localhost:3001".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone(),
        },
        ValidatorInfo {
            ip_address: "127.0.0.1:3002".to_string(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone(),
        },
    ];

    let mut generic_msg = UserSignatureRequest {
        message: hex::encode(PREIMAGE_SHOULD_SUCCEED),
        auxilary_data: Some(vec![
            Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED)),
            Some(hex::encode(AUXILARY_DATA_SHOULD_SUCCEED)),
        ]),
        validators_info,
        timestamp: SystemTime::now(),
        hash: HashingAlgorithm::Keccak,
        signature_verifying_key: DAVE_VERIFYING_KEY.to_vec(),
    };

    let validator_ips_and_keys = vec![
        (validator_ips[0].clone(), X25519_PUBLIC_KEYS[0]),
        (validator_ips[1].clone(), X25519_PUBLIC_KEYS[1]),
    ];

    generic_msg.timestamp = SystemTime::now();

    let test_infinite_loop =
        submit_transaction_requests(validator_ips_and_keys.clone(), generic_msg.clone(), one).await;
    for res in test_infinite_loop {
        assert_eq!(res.unwrap().text().await.unwrap(), "Runtime error: OutOfFuel");
    }
}

#[tokio::test]
#[serial]
async fn test_mutiple_confirm_done() {
    initialize_test_logger().await;
    clean_tests();

    let alice = AccountKeyring::Alice;
    let bob = AccountKeyring::Bob;

    let alice_program = AccountKeyring::Charlie;
    let program_manager = AccountKeyring::Dave;

    let cxt = test_context_stationary().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();

    let program_hash = store_program(
        &api,
        &rpc,
        &program_manager.pair(),
        TEST_PROGRAM_WASM_BYTECODE.to_owned(),
        vec![],
        vec![],
    )
    .await
    .unwrap();

    put_register_request_on_chain(
        &api,
        &rpc,
        &alice,
        alice_program.to_account_id().into(),
        KeyVisibility::Public,
        BoundedVec(vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }]),
    )
    .await;

    put_register_request_on_chain(
        &api,
        &rpc,
        &bob,
        alice_program.to_account_id().into(),
        KeyVisibility::Public,
        BoundedVec(vec![ProgramInstance { program_pointer: program_hash, program_config: vec![] }]),
    )
    .await;
    let p_alice = <sr25519::Pair as Pair>::from_string(DEFAULT_MNEMONIC, None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);

    confirm_registered(
        &api,
        &rpc,
        alice.to_account_id().into(),
        0u8,
        &signer_alice,
        DEFAULT_VERIFYING_KEY.to_vec(),
        0u32,
    )
    .await
    .unwrap();
    confirm_registered(
        &api,
        &rpc,
        bob.to_account_id().into(),
        0u8,
        &signer_alice,
        DEFAULT_VERIFYING_KEY.to_vec(),
        1u32,
    )
    .await
    .unwrap();
    check_has_confirmation(&api, &rpc, &alice.pair()).await;
    check_has_confirmation(&api, &rpc, &bob.pair()).await;
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_increment_or_wipe_request_limit() {
    initialize_test_logger().await;
    clean_tests();
    let substrate_context = test_context_stationary().await;
    let api = get_api(&substrate_context.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&substrate_context.node_proc.ws_url).await.unwrap();
    let kv_store = load_kv_store(&None, None).await;

    let request_limit_query = entropy::storage().parameters().request_limit();
    let request_limit = query_chain(&api, &rpc, request_limit_query, None).await.unwrap().unwrap();

    // no error
    assert!(request_limit_check(
        &rpc,
        &kv_store,
        hex::encode(DAVE_VERIFYING_KEY.to_vec()),
        request_limit
    )
    .await
    .is_ok());

    // run up the request check to one less then max (to check integration)
    for _ in 0..request_limit {
        increment_or_wipe_request_limit(
            &rpc,
            &kv_store,
            hex::encode(DAVE_VERIFYING_KEY.to_vec()),
            request_limit,
        )
        .await
        .unwrap();
    }
    // should now fail
    let err_too_many_requests = request_limit_check(
        &rpc,
        &kv_store,
        hex::encode(DAVE_VERIFYING_KEY.to_vec()),
        request_limit,
    )
    .await
    .map_err(|e| e.to_string());
    assert_eq!(err_too_many_requests, Err("Too many requests - wait a block".to_string()));

    clean_tests();
}

pub async fn submit_transaction_requests(
    validator_urls_and_keys: Vec<(String, [u8; 32])>,
    signature_request: UserSignatureRequest,
    keyring: Sr25519Keyring,
) -> Vec<std::result::Result<reqwest::Response, reqwest::Error>> {
    let mock_client = reqwest::Client::new();
    join_all(
        validator_urls_and_keys
            .iter()
            .map(|validator_tuple| async {
                let server_public_key = PublicKey::from(validator_tuple.1);
                let signed_message = SignedMessage::new(
                    &keyring.pair(),
                    &Bytes(serde_json::to_vec(&signature_request.clone()).unwrap()),
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
}
