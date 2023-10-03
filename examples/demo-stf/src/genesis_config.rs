use std::convert::AsRef;
use std::path::Path;

use anyhow::Context as AnyhowContext;
#[cfg(feature = "experimental")]
use reth_primitives::Bytes;
use serde::de::DeserializeOwned;
use sov_accounts::AccountConfig;
use sov_bank::BankConfig;
use sov_chain_state::ChainStateConfig;
use sov_cli::wallet_state::PrivateKeyAndAddress;
#[cfg(feature = "experimental")]
use sov_evm::{AccountData, EvmConfig, SpecId};
pub use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::Context;
use sov_nft_module::NonFungibleTokenConfig;
use sov_rollup_interface::da::DaSpec;
pub use sov_state::config::Config as StorageConfig;
use sov_value_setter::ValueSetterConfig;

/// Creates config for a rollup with some default settings, the config is used in demos and tests.
use crate::runtime::GenesisConfig;

pub const LOCKED_AMOUNT: u64 = 50;
pub const DEMO_TOKEN_NAME: &str = "sov-demo-token";

/// Configure our rollup with a centralized sequencer using the SEQUENCER_DA_ADDRESS
/// address constant. Since the centralize sequencer's address is consensus critical,
/// it has to be hardcoded as a constant, rather than read from the config at runtime.
///
/// If you want to customize the rollup to accept transactions from your own celestia
/// address, simply change the value of the SEQUENCER_DA_ADDRESS to your own address.
/// For example:
/// ```rust,no_run
/// const SEQUENCER_DA_ADDRESS: &str = "celestia1qp09ysygcx6npted5yc0au6k9lner05yvs9208";
/// ```
pub fn get_genesis_config<C: Context, Da: DaSpec>(
    sequencer_da_address: Da::Address,
    #[cfg(feature = "experimental")] evm_genesis_addresses: Vec<reth_primitives::Address>,
) -> GenesisConfig<C, Da> {
    let token_deployer: PrivateKeyAndAddress<C> = read_private_key();

    create_genesis_config(
        token_deployer.address.clone(),
        sequencer_da_address,
        #[cfg(feature = "experimental")]
        evm_genesis_addresses,
    )
    .expect("Unable to read genesis configuration")
}

fn create_genesis_config<C: Context, Da: DaSpec>(
    sequencer_address: C::Address,
    sequencer_da_address: Da::Address,
    #[cfg(feature = "experimental")] evm_genesis_addresses: Vec<reth_primitives::Address>,
) -> anyhow::Result<GenesisConfig<C, Da>> {
    // This path will be injected as a parameter: #872
    let bank_genesis_path = "../test-data/genesis/bank.json";
    let bank_config: BankConfig<C> = read_json_file(bank_genesis_path)?;
    // This will be read from a file: #872
    let token_address = sov_bank::get_genesis_token_address::<C>(
        &bank_config.tokens[0].token_name,
        bank_config.tokens[0].salt,
    );

    // This will be read from a file: #872
    let sequencer_registry_config = sov_sequencer_registry::SequencerConfig {
        seq_rollup_address: sequencer_address,
        seq_da_address: sequencer_da_address,
        coins_to_lock: sov_bank::Coins {
            amount: LOCKED_AMOUNT,
            token_address,
        },
        is_preferred_sequencer: true,
    };

    // This path will be injected as a parameter: #872
    let value_setter_genesis_path = "../test-data/genesis/value_setter.json";
    let value_setter_config: ValueSetterConfig<C> = read_json_file(value_setter_genesis_path)?;

    let accounts_genesis_path = "../test-data/genesis/accounts.json";
    let accounts_config: AccountConfig<C> = read_json_file(accounts_genesis_path)?;

    let nft_config: NonFungibleTokenConfig = NonFungibleTokenConfig {};

    let chain_state_path = "../test-data/genesis/chain_state.json";
    let chain_state_config: ChainStateConfig = read_json_file(chain_state_path)?;

    Ok(GenesisConfig::new(
        bank_config,
        sequencer_registry_config,
        (),
        chain_state_config,
        value_setter_config,
        accounts_config,
        #[cfg(feature = "experimental")]
        get_evm_config(evm_genesis_addresses),
        nft_config,
    ))
}

fn read_json_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> anyhow::Result<T> {
    let path_str = path.as_ref().display();

    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read genesis from {}", path_str))?;
    let config: T = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse genesis from {}", path_str))?;

    Ok(config)
}

// TODO: #840
#[cfg(feature = "experimental")]
fn get_evm_config(genesis_addresses: Vec<reth_primitives::Address>) -> EvmConfig {
    let data = genesis_addresses
        .into_iter()
        .map(|address| AccountData {
            address,
            balance: AccountData::balance(u64::MAX),
            code_hash: AccountData::empty_code(),
            code: Bytes::default(),
            nonce: 0,
        })
        .collect();

    EvmConfig {
        data,
        chain_id: 1,
        limit_contract_code_size: None,
        spec: vec![(0, SpecId::LATEST)].into_iter().collect(),
        block_timestamp_delta: 1u64,
        ..Default::default()
    }
}

pub fn read_private_key<C: Context>() -> PrivateKeyAndAddress<C> {
    // TODO fix the hardcoded path: #872
    let token_deployer_data =
        std::fs::read_to_string("../test-data/keys/token_deployer_private_key.json")
            .expect("Unable to read file to string");

    let token_deployer: PrivateKeyAndAddress<C> = serde_json::from_str(&token_deployer_data)
        .unwrap_or_else(|_| {
            panic!(
                "Unable to convert data {} to PrivateKeyAndAddress",
                &token_deployer_data
            )
        });

    assert!(
        token_deployer.is_matching_to_default(),
        "Inconsistent key data"
    );

    token_deployer
}
