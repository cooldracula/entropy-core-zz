use axum::http::StatusCode;
use kvdb::clean_tests;
use serial_test::serial;

use super::api::UnsafeQuery;
use crate::helpers::tests::setup_client;

#[tokio::test]
#[serial]
async fn test_unsafe_get_endpoint() {
    setup_client().await;
    let client = reqwest::Client::new();

    let get_query = UnsafeQuery::new("MNEMONIC".to_string(), "foo".to_string()).to_json();

    // Test that the get endpoint works
    let response = client
        .post("http://localhost:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let response_mnemonic = response.text().await.unwrap();
    assert!(!response_mnemonic.is_empty());

    // Update the mnemonic, testing the put endpoint works
    let put_response = client
        .post("http://localhost:3001/unsafe/put")
        .header("Content-Type", "application/json")
        .body(get_query.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(put_response.status(), StatusCode::OK);

    // Check the updated mnemonic is the new value
    let get_response = client
        .post("http://localhost:3001/unsafe/get")
        .header("Content-Type", "application/json")
        .body(get_query)
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);
    let updated_response_mnemonic = get_response.text().await.unwrap();
    assert_eq!(updated_response_mnemonic, "foo".to_string());

    clean_tests();
}
