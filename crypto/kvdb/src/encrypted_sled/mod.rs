//! Wrap a layer of encryption around [sled]. We use [chacha20poly1305] to encrypt/decrypt values.
//! Specifically, use [chacha20poly1305::XChaCha20Poly1305] because the nonces are generated
//! randomly. To create an new [Db], an key to use as entropy for the stream cipher needs to be
//! provided.

mod constants;
mod kv;
mod password;
mod record;
mod result;

// match the API of sled
pub use kv::EncryptedDb as Db;
pub use password::{Password, PasswordMethod, PasswordSalt};
pub use result::{EncryptedDbError as Error, EncryptedDbResult as Result};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub use tests::get_test_password;
