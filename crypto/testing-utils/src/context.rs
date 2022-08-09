use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use subxt::{Client, DefaultConfig, PairSigner, SubstrateExtrinsicParams};

use super::node_proc::TestNodeProcess;

/// substrate node should be installed
fn get_path() -> String {
  format!("{}/target/release/entropy", project_root::get_project_root().unwrap().to_string_lossy())
}

pub type NodeRuntimeSignedExtra = SubstrateExtrinsicParams<DefaultConfig>;

pub async fn test_node_process_with(key: AccountKeyring) -> TestNodeProcess<DefaultConfig> {
  let path = get_path();

  let proc = TestNodeProcess::<DefaultConfig>::build(path.as_str())
    .with_authority(key)
    .scan_for_open_ports()
    .spawn::<DefaultConfig>()
    .await;
  proc.unwrap()
}

pub async fn test_node(key: AccountKeyring) -> TestNodeProcess<DefaultConfig> {
  let path = get_path();

  let proc = TestNodeProcess::<DefaultConfig>::build(path.as_str())
    .with_authority(key)
    .spawn::<DefaultConfig>()
    .await;
  proc.unwrap()
}

pub async fn test_node_process() -> TestNodeProcess<DefaultConfig> {
  test_node_process_with(AccountKeyring::Alice).await
}

pub async fn test_node_process_stationary() -> TestNodeProcess<DefaultConfig> {
  test_node(AccountKeyring::Alice).await
}

#[subxt::subxt(runtime_metadata_path = "../server/entropy_metadata.scale")]
pub mod entropy {}

pub struct TestContext {
  pub node_proc: TestNodeProcess<DefaultConfig>,
  pub api:       entropy::RuntimeApi<DefaultConfig, SubstrateExtrinsicParams<DefaultConfig>>,
}

impl TestContext {
  pub fn client(&self) -> &Client<DefaultConfig> { &self.api.client }
}

pub async fn test_context() -> TestContext {
  env_logger::try_init().ok();
  let node_proc: TestNodeProcess<DefaultConfig> = test_node_process().await;
  let api = node_proc.client().clone().to_runtime_api();
  TestContext { node_proc, api }
}

pub async fn test_context_stationary() -> TestContext {
  env_logger::try_init().ok();
  let node_proc: TestNodeProcess<DefaultConfig> = test_node_process_stationary().await;
  let api = node_proc.client().clone().to_runtime_api();
  TestContext { node_proc, api }
}

pub fn pair_signer(pair: Pair) -> PairSigner<DefaultConfig, Pair> { PairSigner::new(pair) }
