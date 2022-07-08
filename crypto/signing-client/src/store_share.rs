//! # Store Share
//!
//! ## Overview
//!
//! Allows a user to send shards to nodes and have them store it.
//! User's substrate account acts as key value.
//!
//! ## Routes
//!
//! - /store_keyshare - Post - Takes in a key and value for user
#![allow(unused_imports)]
use crate::Global;
use rocket::{http::Status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::{
	fs::File,
	io::{BufWriter, Write},
};
// use tofnd::kv_manager::{error::KvError, KeyReservation, KvManager};

/// User input, contains key (substrate key) and value (entropy shard)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
	pub key: String,
	pub value: Vec<u8>,
}

/// Accepts user input stores shard under user's substrate key in local KVDB
#[post("/store_keyshare", format = "json", data = "<user_input>")]
pub async fn store_keyshare(
	user_input: Json<User>,
	state: &State<Global>,
) -> Result<Status, std::io::Error> {
	// ToDo: JA verify proof
	// ToDo: validate is owner of key address
	// ToDo: JA make sure signed so other key doesn't override own key

	let cached_state = state.inner();
	let kv_manager = cached_state.kv_manager.clone();

	let reservation = kv_manager.kv().reserve_key(user_input.key.clone()).await.unwrap();
	kv_manager.kv().put(reservation, user_input.value.clone()).await.unwrap();

	Ok(Status::Ok)
}
