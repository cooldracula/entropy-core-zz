//! The Node requests the client to take part in a signature generation.

use crate::Global;
use common::OCWMessage;
use constraints::whitelist::is_on_whitelist;
use parity_scale_codec::{Decode, Encode};
use rocket::{http::Status, State};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};
use sp_keyring::AccountKeyring;
use std::str;
use std::thread;
use subxt::{
	sp_runtime::AccountId32, ClientBuilder, Config, DefaultConfig, PairSigner,
	PolkadotExtrinsicParams,
};
use tofnd::kv_manager::KvManager;
// load entropy metadata so that subxt knows what types can be handled by the entropy network
#[subxt::subxt(runtime_metadata_path = "../protocol/src/entropy_metadata.scale")]
pub mod entropy {}

/// Response of the signing node
#[derive(Debug, Encode)]
struct SignRes {
	pub demo: u8,
}

pub type EntropyRuntime =
	entropy::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>;

//ToDo: receive keyshare and store locally
#[post("/sign", format = "application/x-parity-scale-codec", data = "<encoded_data>")]
pub async fn provide_share(encoded_data: Vec<u8>, state: &State<Global>) -> Status {
	println!("encoded_data {:?}", encoded_data);

	let data = OCWMessage::decode(&mut encoded_data.as_ref());
	let data = match data {
		Ok(x) => x,
		Err(err) => panic!("failed to decode input {}", err),
	};
	println!("data: {:?}", &data);

	let cached_state = state.inner();
	let endpoint = cached_state.endpoint.clone();
	let mnemonic = cached_state.mnemonic.clone();
	let kv_manager = cached_state.kv_manager.clone();

	let api = get_api(&endpoint).await.unwrap();
	let block_number = get_block_number(&api).await.unwrap();

	let block_author = get_block_author(&api).await.unwrap();
	let author_endpoint = get_author_endpoint(&api, &block_author).await.unwrap();
	let string_author_endpoint = convert_endpoint(&author_endpoint);
	let bool_block_author = is_block_author(&api, &block_author).await.unwrap();

	for message in data {
		let raw_address = &message.account;
		let address_slice: &[u8; 32] =
			&raw_address.clone().try_into().expect("slice with incorrect length");
		let user = AccountId32::new(*address_slice);

		//TODO: JA may have to be moved to after message is decoded
		let address_whitelist = get_whitelist(&api, &user).await.unwrap();
		let is_address_whitelisted = is_on_whitelist(address_whitelist, &vec![]);

		let does_have_key = does_have_key(&kv_manager, user.to_string()).await;
		if does_have_key && !bool_block_author {
			let _result = send_ip_address(&author_endpoint).await;
		}
	}

	// TODO: JA This thread needs to happen after all signing processes are completed and contain locations in vec of any failures (which need to be stored locally in DB temporarily)
	let handle = thread::spawn(move || async move {
		let api_2 = get_api(&endpoint).await.unwrap();
		let block_author = get_block_author(&api_2).await.unwrap();
		if is_block_author(&api_2, &block_author).await.unwrap() {
			let result = acknowledge_responsibility(&api_2, &mnemonic, block_number).await;
			println!("result of acknowledge responsibility: {:?}", result)
		} else {
			println!("result of no acknowledgmen");
		}
	});
	// TODO: JA Thread blocks the return, not sure if needed a problem, keep an eye out for this downstream
	handle.join().unwrap().await;

	Status::Ok
}

pub async fn get_api(url: &str) -> Result<EntropyRuntime, subxt::Error<entropy::DispatchError>> {
	let api = ClientBuilder::new()
		.set_url(url)
		.build()
		.await?
		.to_runtime_api::<EntropyRuntime>();
	Ok(api)
}

pub async fn is_block_author(
	api: &EntropyRuntime,
	block_author: &AccountId32,
) -> Result<bool, subxt::Error<entropy::DispatchError>> {
	let all_validator_keys = api.storage().session().queued_keys(None).await?;

	let author_keys = all_validator_keys.iter().find(|&key| &key.0 == block_author);
	let key = author_keys.unwrap().1.babe.encode();
	let result = api.client.rpc().has_key(key.into(), "babe".to_string()).await?;
	Ok(result)
}

pub async fn get_block_author(
	api: &EntropyRuntime,
) -> Result<AccountId32, subxt::Error<entropy::DispatchError>> {
	let block_number = get_block_number(api).await?;
	let author = api.storage().propagation().block_author(&block_number, None).await?.unwrap();
	Ok(author)
}

pub async fn get_block_number(
	api: &EntropyRuntime,
) -> Result<u32, subxt::Error<entropy::DispatchError>> {
	let block_number = api.storage().system().number(None).await?;
	Ok(block_number)
}

pub async fn get_author_endpoint(
	api: &EntropyRuntime,
	block_author: &AccountId32,
) -> Result<Vec<u8>, subxt::Error<entropy::DispatchError>> {
	let author_endpoint = api
		.storage()
		.staking_extension()
		.endpoint_register(block_author, None)
		.await?
		.unwrap();
	Ok(author_endpoint)
}

pub fn convert_endpoint(author_endpoint: &Vec<u8>) -> Result<&str, std::str::Utf8Error> {
	Ok(str::from_utf8(author_endpoint).unwrap())
}

pub async fn acknowledge_responsibility(
	api: &EntropyRuntime,
	mnemonic: &String,
	block_number: u32,
) -> Result<(), subxt::Error<entropy::DispatchError>> {
	let pair: Sr25519Pair = Pair::from_string(mnemonic, None).unwrap();
	let signer = PairSigner::new(pair);
	// TODO: JA unhardcode failures and block number should be of the target block
	let result = api
		.tx()
		.relayer()
		.confirm_done(block_number.saturating_sub(2), vec![])
		.sign_and_submit_then_watch_default(&signer)
		.await?
		.wait_for_in_block()
		.await?
		.wait_for_success()
		.await?;

	if let Some(event) = result.find_first::<entropy::relayer::events::ConfirmedDone>()? {
		println!("confirmed done block number: {:?}", event.1);
	} else {
		println!("Failed to confirm done event: {:?}", block_number);
	}
	Ok(())
}

pub async fn get_whitelist(
	api: &EntropyRuntime,
	user: &AccountId32,
) -> Result<Vec<Vec<u8>>, subxt::Error<entropy::DispatchError>> {
	let whitelist = api.storage().constraints().address_whitelist(user, None).await?;

	Ok(whitelist)
}

pub async fn does_have_key(kv: &KvManager, user: String) -> bool {
	kv.kv().exists(&user).await.unwrap()
}

pub async fn send_ip_address(author_endpoint: &Vec<u8>) -> String {
	let my_ip = local_ip::get().unwrap().to_string();
	let mut route = "/get_ip/".to_owned();
	route.push_str(&my_ip);
	let mut ip = str::from_utf8(author_endpoint).unwrap().to_string();
	ip.push_str(&route);
	reqwest::get(ip).await.unwrap().text().await.unwrap()
}
