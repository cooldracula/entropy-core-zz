use super::api::validate_proactive_refresh;
use crate::{
    chain_api::{get_api, get_rpc},
    helpers::{
        launch::LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH,
        tests::{run_to_block, setup_client, spawn_testing_validators},
    },
    r#unsafe::api::UnsafeQuery,
};
use entropy_shared::OcwMessageProactiveRefresh;
use futures::future::join_all;
use kvdb::clean_tests;
use parity_scale_codec::Encode;
use serial_test::serial;
use sp_core::crypto::Ss58Codec;
use sp_keyring::AccountKeyring;
use testing_utils::{
    constants::{TSS_ACCOUNTS, X25519_PUBLIC_KEYS},
    substrate_context::{test_context_stationary, test_node_process_testing_state},
};

#[tokio::test]
#[serial]
async fn test_proactive_refresh() {
    clean_tests();
    let one = AccountKeyring::Eve;
    let _cxt = test_node_process_testing_state().await;

    let signing_address = one.clone().to_account_id().to_ss58check();
    let (validator_ips, _validator_ids, _users_keyshare_option) =
        spawn_testing_validators(Some(signing_address.clone()), true).await;

    let client = reqwest::Client::new();
    let get_query = UnsafeQuery::new(one.to_account_id().to_string(), "".to_string()).to_json();

    // check get key before proactive refresh
    let response = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    let value = response.text().await.unwrap();

    let validators_info = vec![
        entropy_shared::ValidatorInfo {
            ip_address: "127.0.0.1:3001".as_bytes().to_vec(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[0].clone().encode(),
        },
        entropy_shared::ValidatorInfo {
            ip_address: "127.0.0.1:3002".as_bytes().to_vec(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone().encode(),
        },
    ];

    let mut ocw_message = OcwMessageProactiveRefresh { validators_info };

    let test_fail_incorrect_data =
        submit_transaction_requests(validator_ips.clone(), ocw_message.clone()).await;

    for res in test_fail_incorrect_data {
        assert_eq!(res.unwrap().text().await.unwrap(), "Proactive Refresh data incorrect");
    }
    ocw_message.validators_info[0].x25519_public_key = X25519_PUBLIC_KEYS[0];
    let test_user_res =
        submit_transaction_requests(validator_ips.clone(), ocw_message.clone()).await;

    for res in test_user_res {
        assert_eq!(res.unwrap().text().await.unwrap(), "");
    }
    // check get key before proactive refresh
    let response_3 = client
        .post("http://127.0.0.1:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    let value_after = response_3.text().await.unwrap();

    assert_ne!(value, value_after);
    let alice = AccountKeyring::Alice;
    ocw_message.validators_info[0].tss_account = alice.encode();
    ocw_message.validators_info[1].tss_account = alice.encode();

    let test_user_res_not_in_group =
        submit_transaction_requests(validator_ips.clone(), ocw_message.clone()).await;
    for res in test_user_res_not_in_group {
        assert_eq!(
            res.unwrap().text().await.unwrap(),
            "User Error: Invalid Signer: Invalid Signer in Signing group"
        );
    }

    clean_tests();
}

pub async fn submit_transaction_requests(
    validator_urls: Vec<String>,
    ocw_message: OcwMessageProactiveRefresh,
) -> Vec<std::result::Result<reqwest::Response, reqwest::Error>> {
    let mock_client = reqwest::Client::new();
    join_all(
        validator_urls
            .iter()
            .map(|ip| async {
                let url = format!("http://{}/signer/proactive_refresh", ip.clone());
                mock_client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .body(ocw_message.clone().encode())
                    .send()
                    .await
            })
            .collect::<Vec<_>>(),
    )
    .await
}

#[tokio::test]
#[serial]
async fn test_proactive_refresh_validation_fail() {
    clean_tests();
    let cxt = test_context_stationary().await;
    let api = get_api(&cxt.node_proc.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.node_proc.ws_url).await.unwrap();
    let kv = setup_client().await;
    let validators_info = vec![
        entropy_shared::ValidatorInfo {
            ip_address: "127.0.0.1:3001".as_bytes().to_vec(),
            x25519_public_key: X25519_PUBLIC_KEYS[0],
            tss_account: TSS_ACCOUNTS[0].clone().encode(),
        },
        entropy_shared::ValidatorInfo {
            ip_address: "127.0.0.1:3002".as_bytes().to_vec(),
            x25519_public_key: X25519_PUBLIC_KEYS[1],
            tss_account: TSS_ACCOUNTS[1].clone().encode(),
        },
    ];

    let block_number = rpc.chain_get_header(None).await.unwrap().unwrap().number + 1;
    let ocw_message = OcwMessageProactiveRefresh { validators_info };
    run_to_block(&rpc, block_number).await;

    // manipulates kvdb to get to repeated data error
    kv.kv().delete(LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH).await.unwrap();
    let reservation =
        kv.kv().reserve_key(LATEST_BLOCK_NUMBER_PROACTIVE_REFRESH.to_string()).await.unwrap();
    kv.kv().put(reservation, (block_number + 5).to_be_bytes().to_vec()).await.unwrap();

    let err_stale_data =
        validate_proactive_refresh(&api, &rpc, &kv, &ocw_message).await.map_err(|e| e.to_string());
    assert_eq!(err_stale_data, Err("Data is repeated".to_string()));
    clean_tests();
}
