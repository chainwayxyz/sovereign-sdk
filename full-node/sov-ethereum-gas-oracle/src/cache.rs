use std::hash::Hash;
use std::sync::{Mutex, RwLock};

use reth_primitives::H256;
use reth_rpc_types::{Block, Rich, RichBlock};
use schnellru::{ByLength, Limiter, LruMap};
use sov_evm::EthResult;
use sov_modules_api::WorkingSet;

// Create BlockCache
pub struct BlockCache<C: sov_modules_api::Context> {
    cache: Mutex<LruMap<H256, Rich<Block>, ByLength>>,
    provider: sov_evm::Evm<C>,
}

impl<C: sov_modules_api::Context> BlockCache<C> {
    pub fn new(max_size: u32, provider: sov_evm::Evm<C>) -> Self {
        Self {
            cache: Mutex::new(LruMap::new(ByLength::new(max_size))),
            provider,
        }
    }

    /// Gets block from cache or from provider
    pub fn get_block(
        &self,
        block_hash: H256,
        working_set: &mut WorkingSet<C>,
    ) -> EthResult<Option<Rich<Block>>> {
        // Check if block is in cache
        let mut cache = self.cache.lock().unwrap();
        if let Some(block) = cache.get(&block_hash) {
            return Ok(Some(block.clone()));
        }

        // Get block from provider
        let block = self
            .provider
            .get_block_by_hash(block_hash.into(), Some(true), working_set)
            .unwrap()
            .unwrap();

        // Add block to cache
        cache.insert(block_hash, block.clone());

        Ok(Some(block))
    }
}