use thiserror::Error;

#[cfg(test)]
mod tests;

pub mod architectures;
pub mod constraints;
pub mod tx;
pub mod utils;

pub use architectures::*;
pub use constraints::*;
pub use tx::*;
pub use utils::*;

/// Errors related to parsing and evaulating constraints.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Transaction request could not be parsed
    #[error("Invalid transaction request: {0}")]
    InvalidTransactionRequest(String),
    /// Transaction request did not meet constraint requirements.
    #[error("Constraint Evaluation error: {0}")]
    EvaluationError(String),
}
