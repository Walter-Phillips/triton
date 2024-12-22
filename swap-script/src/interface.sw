library;

use std::{bytes::Bytes, string::String};

/// (asset_0, asset_1, is_stable)
pub type PoolId = (AssetId, AssetId, bool);

pub struct Asset {
    pub id: AssetId,
    pub amount: u64,
}

impl Asset {
    pub fn new(id: AssetId, amount: u64) -> Self {
        Self { id, amount }
    }
}

pub struct PoolInfo {
    pub id: PoolId,
    pub reserve_0: u64,
    pub reserve_1: u64,
    pub decimals_0: u8,
    pub decimals_1: u8,
}

impl PoolInfo {
    pub fn new(id: PoolId, decimals_0: u8, decimals_1: u8) -> Self {
        Self {
            id,
            reserve_0: 0,
            reserve_1: 0,
            decimals_0,
            decimals_1,
        }
    }

    pub fn copy_with_reserves(self, reserve_0: u64, reserve_1: u64) -> PoolInfo {
        Self {
            id: self.id,
            reserve_0,
            reserve_1,
            decimals_0: self.decimals_0,
            decimals_1: self.decimals_1,
        }
    }
}

pub struct PoolMetadata {
    pub reserve_0: u64,
    pub reserve_1: u64,
    pub liquidity: Asset,
    pub decimals_0: u8,
    pub decimals_1: u8,
}

impl PoolMetadata {
    pub fn from_pool_and_liquidity(pool: PoolInfo, liquidity: Asset) -> Self {
        Self {
            reserve_0: pool.reserve_0,
            reserve_1: pool.reserve_1,
            liquidity,
            decimals_0: pool.decimals_0,
            decimals_1: pool.decimals_1,
        }
    }
}

abi IBaseCallee {
    #[storage(read, write)]
    fn hook(sender: Identity, amount_0: u64, amount_1: u64, data: Bytes);
}

abi MiraAMM {
    #[storage(read, write)]
    fn create_pool(
        token_0_contract_id: ContractId,
        token_0_sub_id: b256,
        token_1_contract_id: ContractId,
        token_1_sub_id: b256,
        is_stable: bool,
    ) -> PoolId;

    #[storage(read)]
    fn pool_metadata(pool_id: PoolId) -> Option<PoolMetadata>;

    #[storage(read)]
    fn fees() -> (u64, u64, u64, u64);

    #[storage(write)]
    fn set_protocol_fees(volatile_fee: u64, stable_fee: u64);

    #[storage(write)]
    fn set_hook(contract_id: Option<ContractId>);

    #[storage(read)]
    fn hook() -> Option<ContractId>;

    #[storage(read, write)]
    fn mint(pool_id: PoolId, to: Identity) -> Asset;

    #[payable]
    #[storage(read, write)]
    fn burn(pool_id: PoolId, to: Identity) -> (u64, u64);

    #[payable]
    #[storage(read, write)]
    fn swap(
        pool_id: PoolId,
        amount_0_out: u64,
        amount_1_out: u64,
        to: Identity,
        data: Bytes,
    );

    #[storage(read, write)]
    fn transfer_ownership(new_owner: Identity);
}
