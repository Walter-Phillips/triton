use std::str::FromStr;

use ethers::types::U256;
use fuels::types::{Address, AssetId, ContractId, Identity};
use serde::de::{self, Deserializer};
use serde::Deserialize;
#[derive(Debug)]
pub enum Event {
    MiraSwap(SwapEventWithTx),
    MiraMint(MintEventWithTx),
    MiraBurn(BurnEventWithTx),
}
#[derive(Debug, Clone)]
pub enum Action {}
#[derive(Debug, Clone)]
pub struct Config {}

#[derive(Debug)]
pub struct Pool {
    pub pool_name: &'static str,
    pub from: AssetId,
    pub to: AssetId,
    pub reserve_0: U256,
    pub reserve_1: U256,
    pub fee_rate: U256,
}

#[derive(Debug, Deserialize)]

#[allow(dead_code)]
pub struct PangeaLogData {
    chain: u64,
    pub block_number: String,     // Hexadecimal, represented as a String
    block_hash: String,           // Hexadecimal, represented as a String
    transaction_index: String,    // Hexadecimal, represented as a String
    pub transaction_hash: String, // Hexadecimal, represented as a String
    log_index: String,            // Hexadecimal, represented as a String
    pub id: String,               // Hexadecimal, represented as a String
    ra: String,                   // Hexadecimal, represented as a String
    pub rb: String,               // Hexadecimal, represented as a String
    pc: String,                   // Hexadecimal, represented as a String
    is: String,                   // Hexadecimal, represented as a String
    ptr: String,                  // Hexadecimal, represented as a String
    len: String,                  // Hexadecimal, represented as a String
    digest: String,               // Hexadecimal, represented as a String
    pub data: String,             // Hexadecimal, represented as a String
    pub event_name: String,
    pub decoded: String,
}

pub type PoolId = (AssetIdInternal, AssetIdInternal, bool);
pub struct CreatePoolEvent {
    pub pool_id: PoolId,
    pub decimals_0: u8,
    pub decimals_1: u8,
}
#[derive(Debug, Deserialize)]
pub struct AssetIdInternal {
    pub bits: String, // Matches {"bits": "..."} in JSON
}

#[derive(Debug)]
pub struct Asset {
    pub id: AssetIdInternal,
    pub amount: u64,
}
#[derive(Debug)]
pub struct MintEvent {
    pub pool_id: PoolId,
    pub recipient: Identity,
    pub liquidity: Asset,
    pub asset_0_in: u64,
    pub asset_1_in: u64,
}

#[derive(Debug)]
pub struct BurnEvent {
    pub pool_id: PoolId,
    pub recipient: Identity,
    pub liquidity: Asset,
    pub asset_0_out: u64,
    pub asset_1_out: u64,
}

#[derive(Debug)]
pub struct SwapEvent {
    pub pool_id: PoolId,
    pub recipient: Identity,
    pub asset_0_in: u64,
    pub asset_1_in: u64,
    pub asset_0_out: u64,
    pub asset_1_out: u64,
}

#[derive(Debug)]
pub struct SwapEventWithTx {
    pub tx_id: String,
    pub pool_id: PoolId,
    pub recipient: Identity,
    pub asset_0_in: u64,
    pub asset_1_in: u64,
    pub asset_0_out: u64,
    pub asset_1_out: u64,
}

#[derive(Debug)]
pub struct MintEventWithTx {
    pub tx_id: String,
    pub pool_id: PoolId,
    pub recipient: Identity,
    pub liquidity: Asset,
    pub asset_0_in: u64,
    pub asset_1_in: u64,
}

#[derive(Debug)]
pub struct BurnEventWithTx {
    pub tx_id: String,
    pub pool_id: PoolId,
    pub recipient: Identity,
    pub liquidity: Asset,
    pub asset_0_out: u64,
    pub asset_1_out: u64,
}

impl<'de> Deserialize<'de> for SwapEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSwapEvent {
            asset_0_in: u64,
            asset_0_out: u64,
            asset_1_in: u64,
            asset_1_out: u64,
            pool_id: Vec<serde_json::Value>,
            recipient: serde_json::Value,
        }

        let raw = RawSwapEvent::deserialize(deserializer)?;

        // Parse pool_id
        if raw.pool_id.len() != 3 {
            return Err(de::Error::custom("pool_id must contain exactly 3 items"));
        }

        let asset_0 = match &raw.pool_id[0] {
            serde_json::Value::Object(obj) if obj.contains_key("bits") => AssetIdInternal {
                bits: obj["bits"].as_str().unwrap().to_string(),
            },
            _ => return Err(de::Error::custom("Invalid format for pool_id[0]")),
        };

        let asset_1 = match &raw.pool_id[1] {
            serde_json::Value::Object(obj) if obj.contains_key("bits") => AssetIdInternal {
                bits: obj["bits"].as_str().unwrap().to_string(),
            },
            _ => return Err(de::Error::custom("Invalid format for pool_id[1]")),
        };

        let flag = match raw.pool_id[2] {
            serde_json::Value::Bool(flag) => flag,
            _ => return Err(de::Error::custom("Invalid format for pool_id[2]")),
        };

        // Parse recipient
        let recipient = if let Some(address) = raw.recipient.get("Address") {
            if let Some(bits) = address.get("bits").and_then(|b| b.as_str()) {
                Identity::Address(Address::from_str(bits).map_err(de::Error::custom)?)
            } else {
                return Err(de::Error::custom("Invalid Address format"));
            }
        } else if let Some(contract) = raw.recipient.get("ContractId") {
            if let Some(bits) = contract.get("bits").and_then(|b| b.as_str()) {
                Identity::ContractId(ContractId::from_str(bits).map_err(de::Error::custom)?)
            } else {
                return Err(de::Error::custom("Invalid ContractId format"));
            }
        } else {
            return Err(de::Error::custom("Invalid recipient structure"));
        };

        Ok(SwapEvent {
            pool_id: (asset_0, asset_1, flag),
            recipient,
            asset_0_in: raw.asset_0_in,
            asset_1_in: raw.asset_1_in,
            asset_0_out: raw.asset_0_out,
            asset_1_out: raw.asset_1_out,
        })
    }
}

impl<'de> Deserialize<'de> for MintEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMintEvent {
            asset_0_in: u64,
            asset_1_in: u64,
            liquidity: Asset, // Now directly deserialize liquidity as Asset
            pool_id: Vec<serde_json::Value>,
            recipient: serde_json::Value,
        }

        let raw = RawMintEvent::deserialize(deserializer)?;

        // Parse pool_id
        if raw.pool_id.len() != 3 {
            return Err(de::Error::custom("pool_id must contain exactly 3 items"));
        }

        let asset_0 = match &raw.pool_id[0] {
            serde_json::Value::Object(obj) if obj.contains_key("bits") => AssetIdInternal {
                bits: obj["bits"].as_str().unwrap().to_string(),
            },
            _ => return Err(de::Error::custom("Invalid format for pool_id[0]")),
        };

        let asset_1 = match &raw.pool_id[1] {
            serde_json::Value::Object(obj) if obj.contains_key("bits") => AssetIdInternal {
                bits: obj["bits"].as_str().unwrap().to_string(),
            },
            _ => return Err(de::Error::custom("Invalid format for pool_id[1]")),
        };

        let flag = match raw.pool_id[2] {
            serde_json::Value::Bool(flag) => flag,
            _ => return Err(de::Error::custom("Invalid format for pool_id[2]")),
        };

        // Parse recipient
        let recipient = if let Some(address) = raw.recipient.get("Address") {
            if let Some(bits) = address.get("bits").and_then(|b| b.as_str()) {
                Identity::Address(Address::from_str(bits).map_err(de::Error::custom)?)
            } else {
                return Err(de::Error::custom("Invalid Address format"));
            }
        } else if let Some(contract) = raw.recipient.get("ContractId") {
            if let Some(bits) = contract.get("bits").and_then(|b| b.as_str()) {
                Identity::ContractId(ContractId::from_str(bits).map_err(de::Error::custom)?)
            } else {
                return Err(de::Error::custom("Invalid ContractId format"));
            }
        } else {
            return Err(de::Error::custom("Invalid recipient structure"));
        };

        // `liquidity` is now directly an Asset, so it's already handled in the RawMintEvent
        let liquidity = raw.liquidity;

        Ok(MintEvent {
            pool_id: (asset_0, asset_1, flag),
            recipient,
            liquidity,
            asset_0_in: raw.asset_0_in,
            asset_1_in: raw.asset_1_in,
        })
    }
}

// Custom Deserialize implementation for BurnEvent
impl<'de> Deserialize<'de> for BurnEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawBurnEvent {
            asset_0_out: u64,
            asset_1_out: u64,
            liquidity: Asset, // Now directly deserialize liquidity as Asset
            pool_id: Vec<serde_json::Value>,
            recipient: serde_json::Value,
        }

        let raw = RawBurnEvent::deserialize(deserializer)?;

        // Parse pool_id
        if raw.pool_id.len() != 3 {
            return Err(de::Error::custom("pool_id must contain exactly 3 items"));
        }

        let asset_0 = match &raw.pool_id[0] {
            serde_json::Value::Object(obj) if obj.contains_key("bits") => AssetIdInternal {
                bits: obj["bits"].as_str().unwrap().to_string(),
            },
            _ => return Err(de::Error::custom("Invalid format for pool_id[0]")),
        };

        let asset_1 = match &raw.pool_id[1] {
            serde_json::Value::Object(obj) if obj.contains_key("bits") => AssetIdInternal {
                bits: obj["bits"].as_str().unwrap().to_string(),
            },
            _ => return Err(de::Error::custom("Invalid format for pool_id[1]")),
        };

        let flag = match raw.pool_id[2] {
            serde_json::Value::Bool(flag) => flag,
            _ => return Err(de::Error::custom("Invalid format for pool_id[2]")),
        };

        // Parse recipient
        let recipient = if let Some(address) = raw.recipient.get("Address") {
            if let Some(bits) = address.get("bits").and_then(|b| b.as_str()) {
                Identity::Address(Address::from_str(bits).map_err(de::Error::custom)?)
            } else {
                return Err(de::Error::custom("Invalid Address format"));
            }
        } else if let Some(contract) = raw.recipient.get("ContractId") {
            if let Some(bits) = contract.get("bits").and_then(|b| b.as_str()) {
                Identity::ContractId(ContractId::from_str(bits).map_err(de::Error::custom)?)
            } else {
                return Err(de::Error::custom("Invalid ContractId format"));
            }
        } else {
            return Err(de::Error::custom("Invalid recipient structure"));
        };

        // `liquidity` is now directly an Asset, so it's already handled in the RawBurnEvent
        let liquidity = raw.liquidity;

        Ok(BurnEvent {
            pool_id: (asset_0, asset_1, flag),
            recipient,
            liquidity,
            asset_0_out: raw.asset_0_out,
            asset_1_out: raw.asset_1_out,
        })
    }
}

// Custom Deserialize implementation for Asset
impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawAsset {
            id: AssetIdInternal,
            amount: u64,
        }

        let raw = RawAsset::deserialize(deserializer)?;

        Ok(Asset {
            id: raw.id,
            amount: raw.amount,
        })
    }
}
