use std::ops::Range;

use reth_primitives::{Address, Header, SealedHeader, TransactionSigned, H256};
use revm::primitives::{BlobExcessGasAndPrice, EVMError};

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub(crate) struct BlockEnv {
    pub(crate) number: u64,
    pub(crate) coinbase: Address,
    pub(crate) timestamp: u64,
    /// Prevrandao is used after Paris (aka TheMerge) instead of the difficulty value.
    pub(crate) prevrandao: H256,
    /// basefee is added in EIP1559 London upgrade
    pub(crate) basefee: u64,
    pub(crate) gas_limit: u64,
    /// blob_excess_gas_and_price is added in EIP-4844
    pub(crate) blob_excess_gas_and_price: Option<BlobExcessGasAndPrice>,
}

impl Default for BlockEnv {
    fn default() -> Self {
        Self {
            number: Default::default(),
            coinbase: Default::default(),
            timestamp: Default::default(),
            prevrandao: Default::default(),
            basefee: Default::default(),
            gas_limit: reth_primitives::constants::ETHEREUM_BLOCK_GAS_LIMIT,
            blob_excess_gas_and_price: None,
        }
    }
}

impl BlockEnv {
    /// Takes `blob_excess_gas` saves it inside env
    /// and calculates `blob_fee` with [`BlobGasAndFee`].
    pub fn set_blob_excess_gas_and_price(&mut self, excess_blob_gas: u64) {
        self.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(excess_blob_gas));
    }
    /// See [EIP-4844] and [`crate::calc_blob_gasprice`].
    ///
    /// Returns `None` if `Cancun` is not enabled. This is enforced in [`Env::validate_block_env`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    #[inline]
    pub fn get_blob_gasprice(&self) -> Option<u128> {
        self.blob_excess_gas_and_price
            .as_ref()
            .map(|a| a.blob_gasprice)
    }

    /// Return `blob_excess_gas` header field. See [EIP-4844].
    ///
    /// Returns `None` if `Cancun` is not enabled. This is enforced in [`Env::validate_block_env`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    #[inline]
    pub fn get_blob_excess_gas(&self) -> Option<u64> {
        self.blob_excess_gas_and_price
            .as_ref()
            .map(|a| a.excess_blob_gas)
    }
}

// BlockEnv from SealedBlock
impl From<&SealedBlock> for BlockEnv {
    fn from(block: &SealedBlock) -> Self {
        Self {
            number: block.header.number,
            coinbase: block.header.beneficiary,
            timestamp: block.header.timestamp,
            prevrandao: block.header.mix_hash,
            basefee: block.header.base_fee_per_gas.unwrap_or_default(),
            gas_limit: block.header.gas_limit,
            blob_excess_gas_and_price: Some(BlobExcessGasAndPrice::new(
                block.header.excess_blob_gas.unwrap_or_default(),
            )),
        }
    }
}

/// Rlp encoded evm transaction.
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct RlpEvmTransaction {
    /// Rlp data.
    pub rlp: Vec<u8>,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TransactionSignedAndRecovered {
    /// Signer of the transaction
    pub(crate) signer: Address,
    /// Signed transaction
    pub(crate) signed_transaction: TransactionSigned,
    /// Block the transaction was added to
    pub(crate) block_number: u64,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Block {
    /// Block header.
    pub(crate) header: Header,

    /// Transactions in this block.
    pub(crate) transactions: Range<u64>,
}

impl Block {
    pub(crate) fn seal(self) -> SealedBlock {
        SealedBlock {
            header: self.header.seal_slow(),
            transactions: self.transactions,
        }
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SealedBlock {
    /// Block header.
    pub(crate) header: SealedHeader,

    /// Transactions in this block.
    pub(crate) transactions: Range<u64>,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Receipt {
    pub(crate) receipt: reth_primitives::Receipt,
    pub(crate) gas_used: u64,
    pub(crate) log_index_start: u64,
    pub(crate) error: Option<EVMError<u8>>,
}
