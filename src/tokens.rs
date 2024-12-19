use crate::types::Pool;
use fuels::types::{AssetId, U256};

use std::str::FromStr;

pub fn get_pools() -> Vec<Pool> {
    let usdc_asset_id =
        AssetId::from_str("0x286c479da40dc953bddc3bb4c453b608bba2e0ac483b077bd475174115395e6b")
            .unwrap();
    let usdt_asset_id =
        AssetId::from_str("0xa0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812e")
            .unwrap();
    let ezeth_asset_id =
        AssetId::from_str("0x91b3559edb2619cde8ffb2aa7b3c3be97efd794ea46700db7092abeee62281b0")
            .unwrap();
    let eth_asset_id =
        AssetId::from_str("0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07")
            .unwrap();
    let pzeth_asset_id =
        AssetId::from_str("0x1493d4ec82124de8f9b625682de69dcccda79e882b89a55a8c737b12de67bd68")
            .unwrap();
    let weth_asset_id =
        AssetId::from_str("0xa38a5a8beeb08d95744bc7f58528073f4052b254def59eba20c99c202b5acaa3")
            .unwrap();
    let weeth_asset_id =
        AssetId::from_str("0x239ed6e12b7ce4089ee245244e3bf906999a6429c2a9a445a1e1faf56914a4ab")
            .unwrap();
    let usdf_asset_id =
        AssetId::from_str("0x33a6d90877f12c7954cca6d65587c25e9214c7bed2231c188981c7114c1bdb78")
            .unwrap();
    vec![
        Pool {
            pool_name: "WETH/ETH",
            from: weth_asset_id,
            to: eth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(50),
        },
        Pool {
            pool_name: "USDC/USDT",
            from: usdc_asset_id,
            to: usdt_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(50),
        },
        Pool {
            pool_name: "ezETH/ETH",
            from: ezeth_asset_id,
            to: eth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(50),
        },
        Pool {
            pool_name: "pzETH/ETH",
            from: pzeth_asset_id,
            to: eth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(50),
        },
        Pool {
            pool_name: "weETH/ETH",
            from: weeth_asset_id,
            to: eth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(50),
        },
        Pool {
            pool_name: "USDC/USDF",
            from: usdc_asset_id,
            to: usdf_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(50),
        },
        Pool {
            pool_name: "USDC/ETH",
            from: usdc_asset_id,
            to: eth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(300),
        },
        Pool {
            pool_name: "USDT/ETH",
            from: usdt_asset_id,
            to: eth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(300),
        },
        Pool {
            pool_name: "USDC/ezETH",
            from: usdc_asset_id,
            to: ezeth_asset_id,
            reserve_0: U256::from(0),
            reserve_1: U256::from(0),
            fee_rate: U256::from(300),
        },
    ]
}
