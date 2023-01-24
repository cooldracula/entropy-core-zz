// use frame_support::*;
pub use acl::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize as DeserializeDerive, Serialize as SerializeDerive};
pub use sp_core::{H160, H256};
use sp_std::{fmt::Debug, vec::Vec};

/// Supported architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum Arch {
    Evm,
    Btc,
    Generic,
}

/// Trait that defines types for the architecture for a transaction.
pub trait Architecture: Serialize + for<'de> Deserialize<'de> {
    /// Account type for that chain(SS58, H160, etc)
    type Address: Eq + Serialize + for<'de> Deserialize<'de>;
    type TransactionRequest: HasSender<Self>
        + HasReceiver<Self>
        + Serialize
        + for<'de> Deserialize<'de>;
}

/// Trait for getting the the sender of a transaction.
pub trait HasSender<A: Architecture + ?Sized> {
    fn sender(&self) -> Option<A::Address>;
}

/// Trait for getting the the receiver of a transaction.
pub trait HasReceiver<A: Architecture + ?Sized> {
    fn receiver(&self) -> Option<A::Address>;
}

/// Trait for getting the Arch of a transaction.
pub trait HasArch {
    fn arch() -> Arch;
}

/// Represents a user's constraints
#[derive(Default, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct Constraints {
    pub evm_acl: Option<Acl<H160>>,
    pub btc_acl: Option<Acl<H256>>,
}

/// Basic transaction that has a sender and receiver with single accounts.
/// TODO remove this and compose Constraints using trait bounds (eg. HasSender + HasReceiver, etc)
#[derive(Default, Debug, Clone, PartialEq, SerializeDerive, DeserializeDerive)]
pub struct BasicTransaction<A: Architecture> {
    pub from: Option<A::Address>,
    pub to: Option<A::Address>,
}

/// This includes common types and functions related to using ACL functionality.
mod acl {
    use super::*;

    /// An access control list (Allow/Deny lists).
    #[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo, MaxEncodedLen)]
    pub struct Acl<Address> {
        pub addresses: Vec<Address>,
        pub kind: AclKind,
        pub allow_null_recipient: bool,
    }

    /// Represents either an allow or deny list.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum AclKind {
        Allow,
        Deny,
    }

    /// Creates an empty ACL that always evaluates to false.
    impl<A: Default> Default for Acl<A> {
        fn default() -> Self {
            let addresses = Vec::<A>::default();
            Self { addresses, kind: AclKind::Allow, allow_null_recipient: false }
        }
    }
}
