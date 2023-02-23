//! Errors used in User creation

use std::io::Cursor;

use entropy_constraints::Error as ConstraintsError;
use rocket::{
    http::Status,
    response::{Responder, Response},
};
use thiserror::Error;

use crate::chain_api::entropy;

#[derive(Debug, Error)]
pub enum UserErr {
    #[error("Parse error: {0}")]
    Parse(&'static str),
    #[error("Input Validation error: {0}")]
    InputValidation(&'static str),
    #[error("Kv error: {0}")]
    Kv(#[from] kvdb::kv_manager::error::KvError),
    #[error("Substrate error: {0}")]
    Substrate(#[from] subxt::error::DispatchError),
    #[error("Generic Substrate error: {0}")]
    GenericSubstrate(#[from] subxt::error::Error),
    #[error("Serde Json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("ChaCha20 decryption error: {0}")]
    Decryption(String),
    #[error("mnemonic failure: {0:?}")]
    Mnemonic(String),
    #[error("Secret String failure: {0:?}")]
    SecretString(&'static str),
    #[error("Utf8Error: {0:?}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Not Registering error: {0}")]
    NotRegistering(&'static str),
    #[error("Subgroup error: {0}")]
    SubgroupError(&'static str),
    #[error("Invalid Signature: {0}")]
    InvalidSignature(&'static str),
    #[error("Constraints error: {0}")]
    ConstraintsError(#[from] ConstraintsError),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for UserErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        let body = format!("{self}").into_bytes();
        Response::build()
            .sized_body(body.len(), Cursor::new(body))
            .status(Status::InternalServerError)
            .ok()
    }
}
