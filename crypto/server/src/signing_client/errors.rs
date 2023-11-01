//! Errors for everyone ✅
use std::string::FromUtf8Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use entropy_protocol::errors::ProtocolExecutionErr;
use kvdb::kv_manager::error::InnerKvError;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

use super::ProtocolMessage;

/// Errors for protocol execution
#[derive(Debug, Error)]
pub enum ProtocolErr {
    #[error("Protocol Execution Error {0}")]
    ProtocolExecution(#[from] ProtocolExecutionErr),
    #[error("Kv error: {0}")]
    Kv(#[from] kvdb::kv_manager::error::KvError),
    #[error("Inner Kv error: {0}")]
    InnerKv(#[from] InnerKvError),
    #[error("Codec decoding error: {0}")]
    CodecError(#[from] parity_scale_codec::Error),
    #[error("Conversion Error: {0}")]
    TryFrom(#[from] std::array::TryFromSliceError),
    #[error("Decoding Error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),
    #[error("Deserialization Error: {0}")]
    Deserialization(String),
    #[error("Oneshot timeout error: {0}")]
    OneshotTimeout(#[from] RecvError),
    #[error("Subscribe API error: {0}")]
    Subscribe(#[from] SubscribeErr),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Utf8Error: {0:?}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("reqwest event error: {0}")]
    ReqwestEvent(#[from] reqwest_eventsource::Error),
    #[error("Broadcast error: {0}")]
    Broadcast(#[from] Box<tokio::sync::broadcast::error::SendError<ProtocolMessage>>),
    #[error("anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Generic Substrate error: {0}")]
    GenericSubstrate(#[from] subxt::error::Error),
    #[error("Serde Json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Session Error: {0}")]
    SessionError(String),
    #[error("String Conversion Error: {0}")]
    StringConversion(#[from] FromUtf8Error),
    #[error("User Error: {0}")]
    UserError(String),
    #[error("Validation Error: {0}")]
    ValidationErr(#[from] crate::validation::errors::ValidationErr),
    #[error("Subscribe message rejected: {0}")]
    BadSubscribeMessage(String),
    #[error("From Hex Error: {0}")]
    FromHex(#[from] hex::FromHexError),
    #[error("Vec<u8> Conversion Error: {0}")]
    Conversion(&'static str),
    #[error("Could not open ws connection: {0}")]
    ConnectionError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Timed out waiting for remote party")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Encrypted connection error {0}")]
    EncryptedConnection(String),
    #[error("Program error: {0}")]
    ProgramError(#[from] crate::user::errors::ProgramError),
    #[error("Invalid length for converting address")]
    AddressConversionError(String),
    #[error("Socket Address Parse Error: {0}")]
    SocketAddParseError(#[from] std::io::Error),
    #[error("Kv Fatal error")]
    KvSerialize(String),
    #[error("Validator Error: {0}")]
    ValidatorErr(String),
    #[error("Subgroup error: {0}")]
    SubgroupError(&'static str),
    #[error("Account unable to be deserialized: {0}")]
    StringError(&'static str),
    #[error("Option Unwrap error: {0}")]
    OptionUnwrapError(String),
    #[error("Proactive Refresh not called for")]
    NoProactiveRefresh,
}

impl IntoResponse for ProtocolErr {
    fn into_response(self) -> Response {
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Errors for the `subscribe` API
#[derive(Debug, Error)]
pub enum SubscribeErr {
    #[error("no listener: {0}")]
    NoListener(&'static str),
    #[error("invalid party ID: {0}")]
    InvalidPartyId(String),
    #[error("Lock Error: {0}")]
    LockError(String),
    #[error("Invalid Signature: {0}")]
    InvalidSignature(&'static str),
    #[error("Validation error: {0}")]
    Decryption(String),
    #[error("Serde Json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("User Error: {0}")]
    UserError(String),
}

impl IntoResponse for SubscribeErr {
    fn into_response(self) -> Response {
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
