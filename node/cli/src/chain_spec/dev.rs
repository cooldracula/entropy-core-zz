// Copyright (C) 2023 Entropy Cryptography Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::chain_spec::get_account_id_from_seed;
use crate::endowed_accounts::endowed_accounts_dev;

use entropy_runtime::{
    constants::currency::*, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
    BalancesConfig, CouncilConfig, DemocracyConfig, ElectionsConfig, GrandpaConfig, ImOnlineConfig,
    IndicesConfig, MaxNominations, ParametersConfig, RegistryConfig, RuntimeGenesisConfig,
    SessionConfig, StakerStatus, StakingConfig, StakingExtensionConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeConfig,
};
use entropy_runtime::{AccountId, Balance};
use entropy_shared::{
    DAVE_VERIFYING_KEY, EVE_VERIFYING_KEY, FERDIE_VERIFYING_KEY,
    INITIAL_MAX_INSTRUCTIONS_PER_PROGRAM,
};
use grandpa_primitives::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::ChainType;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::sr25519;
use sp_runtime::{BoundedVec, Perbill};

/// The configuration used for development.
///
/// Since Entropy requires at least two signing groups to work properly we spin up this network with
/// two validators, Alice and Bob.
pub fn development_config() -> crate::chain_spec::ChainSpec {
    crate::chain_spec::ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        || {
            development_genesis_config(
                vec![
                    crate::chain_spec::authority_keys_from_seed("Alice"),
                    crate::chain_spec::authority_keys_from_seed("Bob"),
                ],
                vec![],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec!["127.0.0.1:3001", "127.0.0.1:3002"],
            )
        },
        vec![],
        None,
        None,
        None,
        None,
        Default::default(),
    )
}

/// The configuration used for a local development network spun up with the `docker-compose` setup
/// provided in this repository.
///
/// Since Entropy requires at least two signing groups to work properly we spin up this network with
/// two validators, Alice and Bob.
pub fn devnet_local_config() -> crate::chain_spec::ChainSpec {
    crate::chain_spec::ChainSpec::from_genesis(
        "Devnet Local",
        "devnet_local",
        ChainType::Development,
        || {
            development_genesis_config(
                vec![
                    crate::chain_spec::authority_keys_from_seed("Alice"),
                    crate::chain_spec::authority_keys_from_seed("Bob"),
                ],
                vec![],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec!["alice-tss-server:3001", "bob-tss-server:3002"],
            )
        },
        vec![],
        None,
        None,
        None,
        None,
        Default::default(),
    )
}

pub fn development_genesis_config(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    initial_nominators: Vec<AccountId>,
    root_key: AccountId,
    threshold_server_endpoints: Vec<&str>,
) -> RuntimeGenesisConfig {
    let mut endowed_accounts = endowed_accounts_dev();
    // endow all authorities and nominators.
    initial_authorities.iter().map(|x| &x.0).chain(initial_nominators.iter()).for_each(|x| {
        if !endowed_accounts.contains(x) {
            endowed_accounts.push(x.clone())
        }
    });

    // stakers: all validators and nominators.
    let mut rng = rand::thread_rng();
    let stakers = initial_authorities
        .iter()
        .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
        .chain(initial_nominators.iter().map(|x| {
            use rand::{seq::SliceRandom, Rng};
            let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
            let count = rng.gen::<usize>() % limit;
            let nominations = initial_authorities
                .as_slice()
                .choose_multiple(&mut rng, count)
                .map(|choice| choice.0.clone())
                .collect::<Vec<_>>();
            (x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
        }))
        .collect::<Vec<_>>();

    let num_endowed_accounts = endowed_accounts.len();

    const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = ENDOWMENT / 1000;

    RuntimeGenesisConfig {
        system: SystemConfig { code: wasm_binary_unwrap().to_vec(), ..Default::default() },
        balances: BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
        },
        indices: IndicesConfig { indices: vec![] },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        crate::chain_spec::session_keys(
                            x.2.clone(),
                            x.3.clone(),
                            x.4.clone(),
                            x.5.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        },
        staking: StakingConfig {
            validator_count: initial_authorities.len() as u32,
            minimum_validator_count: 0,
            invulnerables: vec![],
            slash_reward_fraction: Perbill::from_percent(10),
            stakers,
            ..Default::default()
        },
        staking_extension: StakingExtensionConfig {
            threshold_servers: vec![
                (
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    (
                        crate::chain_spec::tss_account_id::ALICE.clone(),
                        crate::chain_spec::tss_x25519_public_key::ALICE,
                        threshold_server_endpoints[0].as_bytes().to_vec(),
                    ),
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    (
                        crate::chain_spec::tss_account_id::BOB.clone(),
                        crate::chain_spec::tss_x25519_public_key::BOB,
                        threshold_server_endpoints[1].as_bytes().to_vec(),
                    ),
                ),
            ],
            signing_groups: vec![
                (0, vec![get_account_id_from_seed::<sr25519::Public>("Alice//stash")]),
                (1, vec![get_account_id_from_seed::<sr25519::Public>("Bob//stash")]),
            ],
            proactive_refresh_data: (vec![], vec![]),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, STASH))
                .collect(),
        },
        council: CouncilConfig::default(),
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        },
        sudo: SudoConfig { key: Some(root_key) },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(entropy_runtime::BABE_GENESIS_EPOCH_CONFIG),
            ..Default::default()
        },
        im_online: ImOnlineConfig { keys: vec![] },
        authority_discovery: AuthorityDiscoveryConfig { keys: vec![], ..Default::default() },
        grandpa: GrandpaConfig { authorities: vec![], ..Default::default() },
        technical_membership: Default::default(),
        treasury: Default::default(),
        registry: RegistryConfig {
            registered_accounts: vec![
                (
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    0,
                    None,
                    BoundedVec::try_from(DAVE_VERIFYING_KEY.to_vec()).unwrap(),
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    1,
                    Some(crate::chain_spec::tss_x25519_public_key::EVE),
                    BoundedVec::try_from(EVE_VERIFYING_KEY.to_vec()).unwrap(),
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    2,
                    None,
                    BoundedVec::try_from(FERDIE_VERIFYING_KEY.to_vec()).unwrap(),
                ),
            ],
        },
        parameters: ParametersConfig {
            request_limit: 20,
            max_instructions_per_programs: INITIAL_MAX_INSTRUCTIONS_PER_PROGRAM,
            ..Default::default()
        },
        vesting: Default::default(),
        transaction_storage: Default::default(),
        transaction_payment: Default::default(),
        nomination_pools: Default::default(),
    }
}
