use kvdb::kv_manager::KvManager;
use rocket::{http::Status, response::stream::EventStream, serde::json::Json, Shutdown, State};
use tracing::instrument;

use crate::{
  sign_init::SignInit,
  signing_client::{
    new_party::{Channels, Gg20Service},
    subscribe::{subscribe_all, Listener, Receiver},
    SignerState, SigningErr, SubscribeErr, SubscribeMessage,
  },
};

const SUBSCRIBE_TIMEOUT_SECONDS: u64 = 2;

/// https://github.com/axelarnetwork/tofnd/blob/117a35b808663ceebfdd6e6582a3f0a037151198/src/gg20/sign/mod.rs#L39
/// Endpoint for Communication Manager to initiate a new signing party.
/// We have to do some extra work setting up subscriber streams.
///
/// Communication Manager calls this endpoint for each node in the new Signing Party.
/// The node creates a `ProtocolManager` to run the protocol, and a SubscriberManager to manage
/// subscribed nodes. This method should run the protocol, returning the result.
#[instrument(skip(kv_manager))]
#[post("/new_party", format = "json", data = "<info>")]
pub async fn new_party(
  info: Json<SignInit>,
  state: &State<SignerState>,
  kv_manager: &State<KvManager>,
) -> Result<Status, SigningErr> {
  let info = info.into_inner();
  info!("new_party: {info:?}");
  let gg20_service = Gg20Service::new(state, kv_manager);

  // set up context for signing protocol execution
  let sign_context = gg20_service.get_sign_context(info).await?;

  // subscribe to all other participating parties. Listener waits for other subscribers.
  let (rx_ready, listener) = Listener::new();
  state.listeners.lock().unwrap().insert(sign_context.sign_init.party_uid.to_string(), listener);
  let channels = {
    let stream_in = subscribe_all(&sign_context).await?;
    let broadcast_out = rx_ready.await??;
    Channels(stream_in, broadcast_out)
  };

  let result = gg20_service.execute_sign(&sign_context, channels).await?;
  gg20_service.handle_result(&result, &sign_context);
  Ok(Status::Ok)
}

/// Other nodes in the party call this method to subscribe to this node's broadcasts.
/// The SigningProtocol begins when all nodes in the party have called this method on this node.
#[instrument]
#[post("/subscribe", data = "<msg>")]
pub async fn subscribe(
  msg: Json<SubscribeMessage>,
  end: Shutdown,
  state: &State<SignerState>,
) -> Result<EventStream![], SubscribeErr> {
  let msg = msg.into_inner();
  msg.validate_registration()?;
  info!("got subscribe, with message: {msg:?}");

  if !state.contains_listener(&msg.party_id) {
    // CM hasn't yet informed this node of the party. Wait for a timeout and procede (or fail below)
    tokio::time::sleep(std::time::Duration::from_secs(SUBSCRIBE_TIMEOUT_SECONDS)).await;
  };

  let rx = {
    let mut listeners = state.listeners.lock().unwrap();
    let listener = listeners.get_mut(&msg.party_id).ok_or(SubscribeErr::NoListener("no"))?;
    let rx_outcome = listener.subscribe(&msg)?;

    // If this is the last subscriber, remove the listener from state
    match rx_outcome {
      Receiver::Receiver(rx) => rx,
      Receiver::FinalReceiver(rx) => {
        // all subscribed, wake up the waiting listener in new_party
        let listener = listeners.remove(&msg.party_id).unwrap();
        let (tx, broadcaster) = listener.into_broadcaster();
        let _ = tx.send(Ok(broadcaster));
        rx
      },
    }
  };

  Ok(Listener::create_event_stream(rx, end))
}
