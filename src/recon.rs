use std::{collections::HashSet, str::FromStr};

use crate::{
    constants::{CONTRACT_ID, MIRA_BURN_EVENT_ID, MIRA_MINT_EVENT_ID, MIRA_SWAP_EVENT_ID},
    tokens::get_pools,
    triton,
    types::{
        BurnEvent, BurnEventWithTx, Event, MintEvent, MintEventWithTx, PangeaLogData, SwapEvent,
        SwapEventWithTx,
    },
};
use chrono::Local;
use crossbeam_channel::Sender;
use ethers::types::U256;
use fuels::{
    accounts::{impersonated_account::ImpersonatedAccount, wallet::WalletUnlocked},
    programs::{
        calls::{CallHandler, Execution},
        responses::CallResponse,
    },
    types::transaction::TxPolicies,
};
use futures::StreamExt;
use log::debug;
use mira_v1::interface::PoolMetadata;
use pangea_client::{
    core::types::ChainId, provider::FuelProvider, query::Bound, requests::fuel::GetFuelLogsRequest,
    ClientBuilder, Format, WsProvider,
};

pub async fn stream_mira_events_pangea(tx: Sender<Event>) -> Result<(), ()> {
    dotenvy::dotenv_override().ok();
    let client = ClientBuilder::default()
        .build::<WsProvider>()
        .await
        .unwrap();

    // stream realtime
    let request = GetFuelLogsRequest {
        to_block: Bound::Subscribe,
        id__in: HashSet::from([
            "0x2e40f2b244b98ed6b8204b3de0156c6961f98525c8162f80162fcf53eebd90e7"
                .parse()
                .unwrap(),
        ]),
        chains: HashSet::from([ChainId::FUEL]),
        ..Default::default()
    };

    let stream = client
        .get_fuel_logs_decoded_by_format(request, Format::JsonStream, false)
        .await
        .unwrap();

    futures::pin_mut!(stream);

    while let Some(Ok(data)) = stream.next().await {
        let data: PangeaLogData = serde_json::from_slice(&data).unwrap();

        let rb_value = u64::from_str_radix(&data.rb[2..], 16).expect("Invalid hexadecimal string");
        match rb_value {
            MIRA_SWAP_EVENT_ID => {
                let event: SwapEvent =
                    serde_json::from_slice(data.decoded.as_bytes()).expect("Failed to decode");
                let event_with_tx = SwapEventWithTx {
                    tx_id: data.transaction_hash,
                    pool_id: event.pool_id,
                    recipient: event.recipient,
                    asset_0_in: event.asset_0_in,
                    asset_1_in: event.asset_1_in,
                    asset_0_out: event.asset_0_out,
                    asset_1_out: event.asset_1_out,
                };
                debug!("Swap {:?} {:?}", data.block_number, Local::now());
                let _ = tx.send(Event::MiraSwap(event_with_tx));
            }
            MIRA_MINT_EVENT_ID => {
                let event: MintEvent =
                    serde_json::from_slice(data.decoded.as_bytes()).expect("Failed to decode");
                let event_with_tx = MintEventWithTx {
                    tx_id: data.transaction_hash,
                    pool_id: event.pool_id,
                    liquidity: event.liquidity,
                    recipient: event.recipient,
                    asset_0_in: event.asset_0_in,
                    asset_1_in: event.asset_1_in,
                };
                debug!("Mint {:?} {:?}", event_with_tx, Local::now());
                let _ = tx.send(Event::MiraMint(event_with_tx));
            }
            MIRA_BURN_EVENT_ID => {
                let event: BurnEvent =
                    serde_json::from_slice(data.decoded.as_bytes()).expect("Failed to decode");
                let event_with_tx = BurnEventWithTx {
                    tx_id: data.transaction_hash,
                    pool_id: event.pool_id,
                    liquidity: event.liquidity,
                    recipient: event.recipient,
                    asset_0_out: event.asset_0_out,
                    asset_1_out: event.asset_1_out,
                };
                debug!("Burn {:?} {:?}", event_with_tx, Local::now());
                let _ = tx.send(Event::MiraBurn(event_with_tx));
            }
            _ => {
                debug!(
                    "Not Relevant {:?} {:?}",
                    data.transaction_hash,
                    Local::now()
                )
            }
        }
    }
    Ok(())
}

pub async fn sync_state(triton: &mut triton::Triton, wallet: WalletUnlocked) {
    // Get contract instance
    let address = wallet.address();
    let provider = wallet.provider();
    let contract_id = fuels::types::ContractId::from_str(CONTRACT_ID).unwrap();
    let simulation_account: ImpersonatedAccount =
        ImpersonatedAccount::new(address.clone(), provider.cloned());
    let mira_contract = mira_v1::interface::MiraAmmContract::new(contract_id, simulation_account);

    // Get contract methods
    let contract_methods = mira_contract.methods();

    // Create multicall handlers
    let mut fee_multi_call_handler = CallHandler::new_multi_call(wallet.clone());
    let mut metadata_multi_call_handler = CallHandler::new_multi_call(wallet.clone());

    // Get pool IDs and add them to pool manager
    let pools = get_pools();

    debug!("pools: {:#?}", pools.len());

    for pool in pools.iter() {
        let fee_call_handler = contract_methods
            .fees()
            .with_tx_policies(TxPolicies::default());
        // Add pool_metadata call to multicall handler
        let is_stable = pool.fee_rate < U256::from(300);
        let metadata_call_handler = contract_methods
            .pool_metadata((pool.from, pool.to, is_stable))
            .with_tx_policies(TxPolicies::default());

        fee_multi_call_handler = fee_multi_call_handler.add_call(fee_call_handler);
        metadata_multi_call_handler = metadata_multi_call_handler.add_call(metadata_call_handler);
    }

    // Execute multicall
    let metadata_results: CallResponse<(
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
        Option<PoolMetadata>,
    )> = metadata_multi_call_handler
        .simulate(Execution::StateReadOnly)
        .await
        .unwrap();

    // Execute multicall
    let fee_results: CallResponse<(
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
        (u64, u64, u64, u64),
    )> = fee_multi_call_handler
        .simulate(Execution::StateReadOnly)
        .await
        .unwrap();

    // Convert tuple response to vector
    let metadata_vec = vec![
        metadata_results.value.0,
        metadata_results.value.1,
        metadata_results.value.2,
        metadata_results.value.3,
        metadata_results.value.4,
        metadata_results.value.5,
        metadata_results.value.6,
        metadata_results.value.7,
        metadata_results.value.8,
        metadata_results.value.9,
        metadata_results.value.10,
    ];

    let fee_vec = vec![
        fee_results.value.0,
        fee_results.value.1,
        fee_results.value.2,
        fee_results.value.3,
        fee_results.value.4,
        fee_results.value.5,
        fee_results.value.6,
        fee_results.value.7,
        fee_results.value.8,
        fee_results.value.9,
        fee_results.value.10,
    ];

    debug!("metadata_vec: {:#?}", metadata_vec);
    debug!("fee_vec: {:?}", fee_vec);

    // Process results and update pool states
    for ((i, metadata_opt), _) in metadata_vec
        .into_iter()
        .enumerate()
        .zip(fee_vec.into_iter())
    {
        if let Some(metadata) = metadata_opt {
            if let Some(pool) = triton.pools.get_mut(&i) {
                pool.borrow_mut().reserve_0 = U256::from(metadata.reserve_0);
                pool.borrow_mut().reserve_1 = U256::from(metadata.reserve_1);
            }
        }
    }
    debug!("{:#?}", triton.pools);
}
