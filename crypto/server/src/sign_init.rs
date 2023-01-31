//! Message sent by the Communication Manager to Signing Clients on protocol initiation.
use serde::{Deserialize, Serialize};

pub type MessageDigest = tofn::gg20::sign::MessageDigest;

// CLAIM(TK): The saniziting check required by the tofnd library is only required for a protocol
// execution where this node could hold a multiple secret key shares.
// https://github.com/axelarnetwork/tofnd/blob/cb311ac39e505bdc451d33dcb0228902a80caffe/src/gg20/sign/init.rs#L80
//
/// https://github.com/axelarnetwork/grpc-protobuf/blob/21698133e2f025d706f1dffec19637216d968692/grpc.proto#L120
/// Information passed from the CommunicationManager to all nodes.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SignInit {
    /// Unique id of this signature (may be repeated if this party fails)
    pub sig_uid: String,
    /// identifiers of the participating parties
    // TK: @JA: What to use for this? IP addresses? Substrate addresses? Substrate keys?
    // may overlap with ip_addresses below.
    pub signer_uids: Vec<String>,
    /// The index of the evaluated Shamir Polynomial held by each signer
    // TODO: do we need this? Are these ever not `0..signer_uids.len()`?
    pub signer_idxs: Vec<usize>,
    /// Hash of the message to sign
    pub msg: MessageDigest,
    /// Unique id of this signing party.
    /// If a prior party failed, repeat with a new `party_id`, but the same `sig_uid`
    pub party_uid: String,
    /// User's substrate key. The `key` in the kv-store.
    pub substrate_key: String,
    /// Participating nodes' IP addresses.
    pub ip_addresses: Vec<String>,
}

impl SignInit {
    // TODO(TK): option to make msg Bytes, and have `new` do input validation
    // todo: placeholder for actual logic
    #[allow(dead_code)]
    pub fn new(
        sig_uid: String,
        signer_uids: Vec<String>,
        signer_idxs: Vec<usize>,
        msg: MessageDigest,
        party_uid: String,
        substrate_key: String,
        ip_addresses: Vec<String>,
    ) -> Self {
        Self { sig_uid, signer_uids, signer_idxs, msg, party_uid, substrate_key, ip_addresses }
    }

    // TODO: remove when we have a real implementation
    // Generate temporary data for API testing.
    pub(crate) fn temporary_data(message: entropy_shared::Message) -> Self {
        let digest: MessageDigest = message.sig_request.sig_hash.as_slice().try_into().unwrap();
        let raw_address = &message.account;
        let address_slice: &[u8; 32] =
            &raw_address.clone().try_into().expect("slice with incorrect length");
        let user = sp_core::crypto::AccountId32::new(*address_slice);
        let ip_addresses = message
            .ip_addresses
            .into_iter()
            .map(|ip_address| String::from_utf8(ip_address).unwrap())
            .collect::<Vec<_>>();
        SignInit::new(
            "test".to_string(),
            vec!["test".to_string(), "test1".to_string()],
            vec![0, 1],
            digest,
            "test".to_string(),
            user.to_string(),
            ip_addresses,
        )
    }
}
