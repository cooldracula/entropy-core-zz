use std::{net::TcpListener, time::SystemTime};

use bip39::{Language, Mnemonic};
use entropy_shared::MIN_BALANCE;
use kvdb::clean_tests;
use serial_test::serial;
use sp_core::{sr25519, Pair};
use sp_keyring::AccountKeyring;
use subxt::{ext::sp_core::Bytes, tx::PairSigner};
use testing_utils::{
    constants::{ALICE_STASH_ADDRESS, RANDOM_ACCOUNT},
    substrate_context::{
        test_context_stationary, test_node_process_testing_state, testing_context,
    },
};
use x25519_dalek::PublicKey;

use super::api::{
    check_balance_for_fees, get_all_keys, get_and_store_values, get_random_server_info,
    tell_chain_syncing_is_done, Keys,
};
use crate::{
    chain_api::{get_api, EntropyConfig},
    helpers::{
        launch::{
            DEFAULT_ALICE_MNEMONIC, DEFAULT_BOB_MNEMONIC, DEFAULT_CHARLIE_MNEMONIC,
            DEFAULT_MNEMONIC, FORBIDDEN_KEYS,
        },
        substrate::get_subgroup,
        tests::create_clients,
    },
    validation::{
        derive_static_secret, mnemonic_to_pair, new_mnemonic, SignedMessage, TIME_BUFFER,
    },
};

#[tokio::test]
async fn test_get_all_keys() {
    clean_tests();
    let cxt = testing_context().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();

    let mut result = get_all_keys(&api, 3).await.unwrap();
    let mut result_2 = get_all_keys(&api, 5).await.unwrap();
    let mut result_3 = get_all_keys(&api, 1).await.unwrap();
    let mut result_4 = get_all_keys(&api, 1000).await.unwrap();

    let mut expected_results = vec![
        "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL",
        "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy",
        "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw",
    ];
    result.sort();
    expected_results.sort();
    result_2.sort();
    result_3.sort();
    result_4.sort();

    assert_eq!(result, expected_results);
    assert_eq!(result_2, expected_results);
    assert_eq!(result_3, expected_results);
    assert_eq!(result_4, expected_results);
    clean_tests();
}

#[tokio::test]
#[should_panic]
async fn test_get_all_keys_fail() {
    clean_tests();
    let cxt = testing_context().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let _ = get_all_keys(&api, 0).await.unwrap();
    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_sync_kvdb() {
    clean_tests();
    let _ctx = test_context_stationary().await;
    let addrs = vec![
        "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL".to_string(),
        "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy".to_string(),
        "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw".to_string(),
    ];

    let b_usr_sk =
        mnemonic_to_pair(&Mnemonic::from_phrase(DEFAULT_BOB_MNEMONIC, Language::English).unwrap())
            .unwrap();
    let b_usr_ss = derive_static_secret(&b_usr_sk);
    let recip = PublicKey::from(&b_usr_ss);
    let values = vec![vec![10], vec![11], vec![12]];

    let port = 3001;
    let (bob_axum, _) = create_clients("bob".to_string(), values, addrs.clone(), false, true).await;
    let listener_bob = TcpListener::bind(format!("0.0.0.0:{port}")).unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener_bob).unwrap().serve(bob_axum).await.unwrap();
    });
    let client = reqwest::Client::new();
    let mut keys = Keys { keys: addrs, timestamp: SystemTime::now() };
    let enc_keys =
        SignedMessage::new(&b_usr_sk, &Bytes(serde_json::to_vec(&keys).unwrap()), &recip).unwrap();
    let formatted_url = format!("http://127.0.0.1:{port}/validator/sync_kvdb");
    let result = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&enc_keys).unwrap())
        .send()
        .await
        .unwrap();

    // Validates that keys signed/encrypted to the correct key
    // return no error (status code 200).
    assert_eq!(result.status(), 200);

    let a_usr_sk = mnemonic_to_pair(
        &Mnemonic::from_phrase(DEFAULT_ALICE_MNEMONIC, Language::English).unwrap(),
    )
    .unwrap();
    let a_usr_ss = derive_static_secret(&a_usr_sk);
    let sender = PublicKey::from(&a_usr_ss);

    let enc_keys_failed_decrypt =
        SignedMessage::new(&b_usr_sk, &Bytes(serde_json::to_vec(&keys).unwrap()), &sender).unwrap();
    let formatted_url = format!("http://127.0.0.1:{port}/validator/sync_kvdb");
    let result_2 = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&enc_keys_failed_decrypt).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(result_2.status(), 500);
    assert_eq!(
        result_2.text().await.unwrap(),
        "Validation Error: ChaCha20 decryption error: aead::Error"
    );

    let enc_keys =
        SignedMessage::new(&a_usr_sk, &Bytes(serde_json::to_vec(&keys).unwrap()), &recip).unwrap();
    let formatted_url = format!("http://127.0.0.1:{port}/validator/sync_kvdb");
    let result_3 = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&enc_keys).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(result_3.status(), 500);
    assert_eq!(result_3.text().await.unwrap(), "Validator not in subgroup");

    // check random key fails not in subgroup
    let random_usr_sk = mnemonic_to_pair(&new_mnemonic()).unwrap();

    let enc_keys =
        SignedMessage::new(&random_usr_sk, &Bytes(serde_json::to_vec(&keys).unwrap()), &recip)
            .unwrap();
    let formatted_url = format!("http://127.0.0.1:{port}/validator/sync_kvdb");
    let result_3 = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&enc_keys).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(result_3.status(), 500);
    // fails on lookup for stash key
    assert_eq!(result_3.text().await.unwrap(), "Option Unwrap error: Stash Fetch Error");

    keys.keys = vec![FORBIDDEN_KEYS[0].to_string()];
    let enc_forbidden =
        SignedMessage::new(&b_usr_sk, &Bytes(serde_json::to_vec(&keys).unwrap()), &recip).unwrap();
    let result_4 = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&enc_forbidden).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(result_4.status(), 500);
    assert_eq!(result_4.text().await.unwrap(), "Forbidden Key");

    keys.timestamp = keys.timestamp.checked_sub(TIME_BUFFER).unwrap();
    let enc_stale =
        SignedMessage::new(&b_usr_sk, &Bytes(serde_json::to_vec(&keys).unwrap()), &recip).unwrap();
    let result_5 = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&enc_stale).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(result_5.status(), 500);
    assert_eq!(result_5.text().await.unwrap(), "Validation Error: Message is too old");

    let sig: [u8; 64] = [0; 64];
    let slice: [u8; 32] = [0; 32];
    let nonce: [u8; 12] = [0; 12];

    let user_input_bad = SignedMessage::new_test(
        Bytes(serde_json::to_vec(&keys.clone()).unwrap()),
        sr25519::Signature::from_raw(sig),
        AccountKeyring::Eve.pair().public().into(),
        slice,
        slice,
        nonce,
    );

    let failed_sign = client
        .post(formatted_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&user_input_bad).unwrap())
        .send()
        .await
        .unwrap();

    assert_eq!(failed_sign.status(), 500);
    assert_eq!(failed_sign.text().await.unwrap(), "Invalid Signature: Invalid signature.");

    clean_tests();
}

#[tokio::test]
#[serial]
async fn test_get_and_store_values() {
    clean_tests();
    let cxt = test_node_process_testing_state().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let p_alice = <sr25519::Pair as Pair>::from_string(DEFAULT_MNEMONIC, None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);
    let my_subgroup = get_subgroup(&api, &signer_alice).await.unwrap().0.unwrap();
    let server_info =
        get_random_server_info(&api, my_subgroup, signer_alice.account_id().clone()).await.unwrap();
    let recip_key = x25519_dalek::PublicKey::from(server_info.x25519_public_key);
    let keys = vec![
        "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL".to_string(),
        "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy".to_string(),
        "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw".to_string(),
    ];
    let port_0 = 3002;
    let port_1 = 3003;
    let values = vec![vec![10], vec![11], vec![12]];
    // Construct a client to use for dispatching requests.
    let (alice_axum, _) =
        create_clients("alice".to_string(), values.clone(), keys.clone(), true, false).await;

    let (bob_axum, bob_kv) = create_clients("bob".to_string(), vec![], vec![], false, true).await;
    let listener_alice = TcpListener::bind(format!("0.0.0.0:{port_0}")).unwrap();
    let listener_bob = TcpListener::bind(format!("0.0.0.0:{port_1}")).unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener_alice).unwrap().serve(alice_axum).await.unwrap();
    });
    tokio::spawn(async move {
        axum::Server::from_tcp(listener_bob).unwrap().serve(bob_axum).await.unwrap();
    });
    let p_charlie = <sr25519::Pair as Pair>::from_string(DEFAULT_CHARLIE_MNEMONIC, None).unwrap();
    let signer_charlie = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_charlie);
    let _result = get_and_store_values(
        keys.clone(),
        &bob_kv,
        "127.0.0.1:3002".to_string(),
        9,
        false,
        &recip_key,
        &signer_charlie,
    )
    .await;
    for (i, key) in keys.iter().enumerate() {
        println!("!! -> -> RECEIVED KEY at IDX {i} of value {key:?}");
        let val = bob_kv.kv().get(key).await;
        assert!(val.is_ok());
        assert_eq!(val.unwrap(), values[i]);
    }
    clean_tests();
}

#[tokio::test]
#[should_panic = "index out of bounds: the len is 1 but the index is 1"]
async fn test_get_random_server_info() {
    clean_tests();
    let cxt = testing_context().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let p_alice = <sr25519::Pair as Pair>::from_string(DEFAULT_MNEMONIC, None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);
    let (my_subgroup, validator_address) = get_subgroup(&api, &signer_alice).await.unwrap();

    let result =
        get_random_server_info(&api, my_subgroup.unwrap(), signer_alice.account_id().clone())
            .await
            .unwrap();
    assert_eq!("127.0.0.1:3001".as_bytes().to_vec(), result.endpoint);
    // panics here because no other validators in subgroup
    get_random_server_info(&api, my_subgroup.unwrap(), validator_address).await.unwrap();
    clean_tests();
}

#[tokio::test]
#[should_panic = "Account does not exist, add balance"]
async fn test_check_balance_for_fees() {
    clean_tests();
    let cxt = testing_context().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();

    let result = check_balance_for_fees(&api, &ALICE_STASH_ADDRESS, MIN_BALANCE).await.unwrap();

    assert!(result);

    let result_2 = check_balance_for_fees(&api, &ALICE_STASH_ADDRESS, 10000000000000000000000u128)
        .await
        .unwrap();
    assert!(!result_2);

    let _ = check_balance_for_fees(&api, &RANDOM_ACCOUNT, MIN_BALANCE).await.unwrap();
    clean_tests();
}

#[tokio::test]
async fn test_tell_chain_syncing_is_done() {
    clean_tests();
    let cxt = testing_context().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let p_alice = <sr25519::Pair as Pair>::from_string("//Alice", None).unwrap();
    let signer_alice = PairSigner::<EntropyConfig, sr25519::Pair>::new(p_alice);

    // expect this to fail in the proper way
    let result = tell_chain_syncing_is_done(&api, &signer_alice).await;
    assert!(result.is_err());
    clean_tests();
}
