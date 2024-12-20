use crate::{
    tokens::get_pools,
    types::{BurnEventWithTx, Event, MintEventWithTx, Pool, SwapEventWithTx},
};
use ethers::types::U256;
use fuels::types::AssetId;
use mira_v1::interface::PoolId;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Debug, Clone, Copy)]
pub struct IndexedPair {
    pub index: usize,
    pub pair: PoolId,
}
#[derive(Debug, Clone)]
pub struct Cycle {
    pub cycle: Vec<IndexedPair>,
}
#[derive(Debug)]
pub struct Triton {
    // // Pangea Client
    // pub pangea_client: pangea_client::Client<WsProvider>,
    // Mapping from index to PoolId
    pub index_mapping: HashMap<usize, PoolId>,
    // Reverse mapping of PoolId top Index
    pub pool_id_mapping: HashMap<PoolId, usize>,
    // Mapping of index to Pool
    pub pools: HashMap<usize, RefCell<Pool>>,
    // Viable cycles found on startup
    pub cycles: Vec<Cycle>,
}

impl Default for Triton {
    fn default() -> Self {
        Self::new()
    }
}

impl Triton {
    pub fn new() -> Triton {
        let pairs = get_pools();
        let mut index_mapping = HashMap::new();
        let mut pool_id_mapping = HashMap::new();
        let mut pools = HashMap::new();
        let mut indexed_pairs = Vec::new();
        let mut index = 0;

        for pair in pairs {
            let is_stable = pair.fee_rate < U256::from(300);
            let pool_id = (pair.from, pair.to, is_stable);
            index_mapping.insert(index, pool_id);
            pool_id_mapping.insert(pool_id, index);
            pools.insert(index, RefCell::new(pair));

            let indexed_pair = IndexedPair {
                index: *pool_id_mapping.get(&pool_id).unwrap(),
                pair: pool_id,
            };

            indexed_pairs.push(indexed_pair);
            index += 1;
        }

        // Now that we have indexed pairs and pools, we can find cycles
        let mut cycles = Vec::new();
        let mut seen = HashSet::new();

        // Going through ETH atm change to USDC later
        let usdc_asset_id =
            AssetId::from_str("0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07")
                .unwrap();
        // We call find_cycles to find triangular arbitrage cycles (USDC to USDC, for example)
        cycles = Triton::find_cycles(
            &indexed_pairs,
            usdc_asset_id, // Starting token
            usdc_asset_id, // Target token
            5,             // Maximum hops (for triangular arbitrage)
            &Vec::new(),
            &mut cycles,
            &mut seen,
        );

        Triton {
            index_mapping,
            pool_id_mapping,
            pools,
            cycles,
        }
    }
    pub fn find_cycles(
        pairs: &[IndexedPair],
        token_in: AssetId,
        token_out: AssetId,
        max_hops: i32,
        current_pairs: &Vec<IndexedPair>,
        cycles: &mut Vec<Cycle>,
        seen: &mut HashSet<usize>,
    ) -> Vec<Cycle> {
        let mut cycles_copy: Vec<Cycle> = cycles.clone();

        for pair in pairs {
            // Skip if already visited
            if seen.contains(&pair.index) {
                continue;
            }

            // Determine the next token based on the current pool
            let temp_out = if token_in == pair.pair.0 {
                pair.pair.1
            } else if token_in == pair.pair.1 {
                pair.pair.0
            } else {
                continue;
            };

            // Mark this pair as visited
            let mut new_seen = seen.clone();
            new_seen.insert(pair.index);

            // If we've reached the target token, store the cycle
            if temp_out == token_out {
                let mut new_cycle = current_pairs.clone();
                new_cycle.push(*pair);
                cycles_copy.push(Cycle { cycle: new_cycle });
            } else if max_hops > 1 {
                let mut new_pairs = current_pairs.clone();
                new_pairs.push(*pair);
                cycles_copy = Self::find_cycles(
                    pairs,
                    temp_out,
                    token_out,
                    max_hops - 1,
                    &new_pairs,
                    &mut cycles_copy,
                    &mut new_seen,
                );
            }
        }

        cycles_copy
    }

    pub fn check_if_we_have_pool(
        pool_id: &PoolId,
        pool_id_mapping: &HashMap<PoolId, usize>,
    ) -> bool {
        pool_id_mapping.contains_key(pool_id)
    }

    fn handle_event_if_pool_exists<F>(&self, pool_id: (AssetId, AssetId, bool), event_handler: F)
    where
        F: FnOnce(),
    {
        if Triton::check_if_we_have_pool(&pool_id, &self.pool_id_mapping) {
            event_handler();
        }
    }
    pub fn process_event(&self, event: Event) {
        match event {
            Event::MiraSwap(event) => {
                let pool_id = (
                    AssetId::from_str(&event.pool_id.0.bits).expect("no asset id"),
                    AssetId::from_str(&event.pool_id.1.bits).expect("no asset id"),
                    event.pool_id.2,
                );
                let handler = || self.handle_swap(&event);
                self.handle_event_if_pool_exists(pool_id, handler);
            }
            Event::MiraMint(event) => {
                let pool_id = (
                    AssetId::from_str(&event.pool_id.0.bits).expect("no asset id"),
                    AssetId::from_str(&event.pool_id.1.bits).expect("no asset id"),
                    event.pool_id.2,
                );
                let handler = || self.handle_mint(&event);
                self.handle_event_if_pool_exists(pool_id, handler);
            }
            Event::MiraBurn(event) => {
                let pool_id = (
                    AssetId::from_str(&event.pool_id.0.bits).expect("no asset id"),
                    AssetId::from_str(&event.pool_id.1.bits).expect("no asset id"),
                    event.pool_id.2,
                );
                let handler = || self.handle_burn(&event);
                self.handle_event_if_pool_exists(pool_id, handler);
            }
        }
    }

    pub fn handle_swap(&self, event: &SwapEventWithTx) {
        let pool_id = (
            AssetId::from_str(&event.pool_id.0.bits).expect("no asset id"),
            AssetId::from_str(&event.pool_id.1.bits).expect("no asset id"),
            event.pool_id.2,
        );
        let index = self.pool_id_mapping.get(&pool_id).expect("Pool not found");
        let pool = self.pools.get(index).expect("Pool not found");

        log::debug!(
            "Before swap - Pool {:?} state: reserve_0={}, reserve_1={}",
            event.pool_id,
            pool.borrow().reserve_0,
            pool.borrow().reserve_1
        );

        {
            // Create a block to scope the immutable borrow
            let current_reserve_0 = pool.borrow().reserve_0;
            let new_reserve_0 = current_reserve_0
                .checked_add(event.asset_0_in.into())
                .and_then(|x| x.checked_sub(event.asset_0_out.into()))
                .expect("Can't add or subtract");

            let current_reserve_1 = pool.borrow().reserve_1;
            let new_reserve_1 = current_reserve_1
                .checked_add(event.asset_1_in.into())
                .and_then(|x| x.checked_sub(event.asset_1_out.into()))
                .unwrap();

            // Now perform the mutable borrow
            let mut pool_mut = pool.borrow_mut();
            pool_mut.reserve_0 = new_reserve_0;
            pool_mut.reserve_1 = new_reserve_1;
        }
        log::debug!(
            "After swap - Pool {:?} state: reserve_0={}, reserve_1={}\nSwap details: in_0={}, in_1={}, out_0={}, out_1={}",
            event.pool_id,
            pool.borrow().reserve_0,
            pool.borrow().reserve_1,
            event.asset_0_in,
            event.asset_1_in,
            event.asset_0_out,
            event.asset_1_out
        );
    }
    pub fn handle_mint(&self, event: &MintEventWithTx) {
        let pool_id = (
            AssetId::from_str(&event.pool_id.0.bits).expect("no asset id"),
            AssetId::from_str(&event.pool_id.1.bits).expect("no asset id"),
            event.pool_id.2,
        );
        let index = self.pool_id_mapping.get(&pool_id).expect("Pool not found");
        let pool = self.pools.get(index).expect("Pool not found");

        log::debug!(
            "Before mint - Pool {:?} state: reserve_0={}, reserve_1={}",
            event.pool_id,
            pool.borrow().reserve_0,
            pool.borrow().reserve_1
        );

        {
            let current_reserve_0 = pool.borrow().reserve_0;
            let new_reserve_0 = current_reserve_0
                .checked_add(event.asset_0_in.into())
                .expect("Can't add");

            let current_reserve_1 = pool.borrow().reserve_1;
            let new_reserve_1 = current_reserve_1
                .checked_add(event.asset_1_in.into())
                .expect("Can't add");

            let mut pool_mut = pool.borrow_mut();
            pool_mut.reserve_0 = new_reserve_0;
            pool_mut.reserve_1 = new_reserve_1;
        }
        log::debug!(
        "After mint - Pool {:?} state: reserve_0={}, reserve_1={}\nMint details: in_0={}, in_1={}, liquidity={}",
        event.pool_id,
        pool.borrow().reserve_0,
        pool.borrow().reserve_1,
        event.asset_0_in,
        event.asset_1_in,
        event.liquidity.amount
    );
    }
    pub fn handle_burn(&self, event: &BurnEventWithTx) {
        let pool_id = (
            AssetId::from_str(&event.pool_id.0.bits).expect("no asset id"),
            AssetId::from_str(&event.pool_id.1.bits).expect("no asset id"),
            event.pool_id.2,
        );
        let index = self.pool_id_mapping.get(&pool_id).expect("Pool not found");
        let pool = self.pools.get(index).expect("Pool not found");

        log::debug!(
            "Before burn - Pool {:?} state: reserve_0={}, reserve_1={}",
            event.pool_id,
            pool.borrow().reserve_0,
            pool.borrow().reserve_1
        );

        {
            let current_reserve_0 = pool.borrow().reserve_0;
            let new_reserve_0 = current_reserve_0
                .checked_sub(event.asset_0_out.into())
                .expect("Can't subtract");

            let current_reserve_1 = pool.borrow().reserve_1;
            let new_reserve_1 = current_reserve_1
                .checked_sub(event.asset_1_out.into())
                .expect("Can't subtract");

            let mut pool_mut = pool.borrow_mut();
            pool_mut.reserve_0 = new_reserve_0;
            pool_mut.reserve_1 = new_reserve_1;
        }
        log::debug!(
                "After burn - Pool {:?} state: reserve_0={}, reserve_1={}\nBurn details: out_0={}, out_1={}, liquidity={}",
                event.pool_id,
                pool.borrow().reserve_0,
                pool.borrow().reserve_1,
                event.asset_0_out,
                event.asset_1_out,
                event.liquidity.amount
            );
    }
}
