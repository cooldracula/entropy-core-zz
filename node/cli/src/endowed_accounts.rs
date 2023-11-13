use hex_literal::hex;
pub use node_primitives::AccountId;
use sp_core::sr25519;

use crate::chain_spec::get_account_id_from_seed;

pub fn endowed_accounts_dev() -> Vec<AccountId> {
    vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        get_account_id_from_seed::<sr25519::Public>("One"),
        get_account_id_from_seed::<sr25519::Public>("Two"),
        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
        get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
        get_account_id_from_seed::<sr25519::Public>("One//stash"),
        get_account_id_from_seed::<sr25519::Public>("Two//stash"),
        hex!["e0543c102def9f6ef0e8b8ffa31aa259167a9391566929fd718a1ccdaabdb876"].into(),
        hex!["2a8200850770290c7ea3b50a8ff64c6761c882ff8393dc95fccb5d1475eff17f"].into(),
        hex!["14d223daeec68671f07298c66c9458980a48bb89fb8a85d5df31131acad8d611"].into(),
    ]
}

pub fn endowed_accounts_devnet() -> Vec<AccountId> {
    vec![
        // random placeholder
        // hex!["a617f1a88de5efbaefaafdf4d02818e00b6bb45c673c2dedca447b62dad2a26d"].into(),
    ]
}
