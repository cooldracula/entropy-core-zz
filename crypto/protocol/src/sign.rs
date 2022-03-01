use anyhow::{anyhow, Context, Result};
use curv::{arithmetic::Converter, BigInt};
use futures::{SinkExt, StreamExt, TryStreamExt};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::sign::{
	OfflineStage, SignManual,
};
use round_based::{async_runtime::AsyncProtocol, Msg};
use std::path::PathBuf;
use structopt::StructOpt;

use crate::gg20_sm_client::join_computation;

#[derive(Debug, StructOpt, Clone)]
pub struct SignCli {
	pub address: surf::Url,
	pub room: String,
	/// Index of the party
	pub index: u16,
	pub parties: Vec<u16>,
	pub data_to_sign: String,
}

pub async fn sign(args: SignCli) -> Result<()> {
	println!("User starts gg20-signing...");
	let local_share = PathBuf::from(format!("local-share{}.json", args.index));
	println!("0.1");
	println!("args {:?}", args);

	let local_share = tokio::fs::read(local_share)
		.await
		.context(format!("cannot read local share at index {}", args.index))?;
	// println!("local_share {:?}", local_share);
	println!("0.2");

	let local_share = serde_json::from_slice(&local_share).context("parse local share")?;
	let number_of_parties = args.parties.len();

	println!("0.4");
	let (i, incoming, outgoing) =
		join_computation(args.address.clone(), &format!("{}-offline", args.room))
			.await
			.context("join offline computation")?;
	println!("0.5");

	let incoming = incoming.fuse();
	tokio::pin!(incoming);
	tokio::pin!(outgoing);

	println!("1");
	let signing = OfflineStage::new(i, args.parties, local_share)?;
	let completed_offline_stage = AsyncProtocol::new(signing, incoming, outgoing)
		.run()
		.await
		// TODO: tk alice can't send messages to herself in round_based dep
		.map_err(|e| anyhow!("protocol execution terminated with error: {}", e))?;

	let (_i, incoming, outgoing) = join_computation(args.address, &format!("{}-online", args.room))
		.await
		.context("join online computation")?;

	tokio::pin!(incoming);
	tokio::pin!(outgoing);
	println!("2");

	let (signing, partial_signature) =
		SignManual::new(BigInt::from_bytes(args.data_to_sign.as_bytes()), completed_offline_stage)?;

	outgoing
		.send(Msg { sender: i, receiver: None, body: partial_signature })
		.await?;

	let partial_signatures: Vec<_> = incoming
		.take(number_of_parties - 1)
		.map_ok(|msg| msg.body)
		.try_collect()
		.await?;
	let signature = signing.complete(&partial_signatures).context("online stage failed")?;
	let signature = serde_json::to_string(&signature).context("serialize signature")?;
	println!("3");

	println!("signature {}", signature);

	Ok(())
}
