use std::str::FromStr;

// use crate::calc::NetPositiveCycle;
// use fuels::types::transaction::TxPolicies;
use mira_v1::interface::MiraAmmContract;

use ethers::types::{I256, U256};
use fuels::{
    accounts::{wallet::WalletUnlocked, Account},
    macros::abigen,
    prelude::Result,
    programs::calls::Execution,
    types::{
        input::Input, output::Output, transaction::TxPolicies,
        transaction_builders::VariableOutputPolicy, AssetId, ContractId,
    },
};
use mira_v1::interface::PoolId;

use crate::{calc::NetPositiveCycle, constants::CONTRACT_ID};

pub async fn send_multi_hop(wallet: &WalletUnlocked, profitable_cycle: NetPositiveCycle) {
    let policies = TxPolicies::default();
    let amount_in: u64 =
        scale_and_convert_to_u64_from_u256(profitable_cycle.optimal_in, 1000000).unwrap();
    println!("optimal amount_in: {:#?}", amount_in);
    println!("token_in: {:#?}", profitable_cycle.cycle_ids[0].0);
    let tx = preview_swap_exact_input(
        wallet,
        amount_in,
        profitable_cycle.cycle_ids[0].0,
        profitable_cycle.profit.as_u64(),
        profitable_cycle.cycle_ids,
        999999999,
        Some(policies),
    )
    .await
    .unwrap();
    println!("tx: {:#?}", tx);
}
fn scale_and_convert_to_u64_from_u256(value: U256, scale: u64) -> Option<u64> {
    // Convert scale to U256
    let scaling_factor = U256::from(scale);

    // Perform scaling (integer division)
    let scaled_value = value / scaling_factor;

    // Check if the scaled value fits within u64
    if scaled_value <= U256::from(u64::MAX) { 
        Some(scaled_value.low_u64()) // Extract lower 64 bits
    } else {
        None // Indicate out of bounds
    }
}
pub fn scale_and_convert_to_u64(value: I256, scale: i64) -> i64 {
    // Ensure the scaling factor is positive
    let scaling_factor = I256::from(scale);

    // Perform scaling (integer division)
    let scaled_value = value / scaling_factor;

    scaled_value.low_i64()
}
abigen!(Script(
    name = "SwapScript",
    abi = "/Users/walterphillips/BlockchainDev/triton/swap-script/out/debug/swap-script-abi.json"
));

pub async fn get_transaction_inputs_outputs(
    wallet: &WalletUnlocked,
    assets: &Vec<(AssetId, u64)>,
) -> (Vec<Input>, Vec<Output>) {
    let mut inputs: Vec<Input> = vec![];
    let mut outputs: Vec<Output> = Vec::with_capacity(assets.len());

    for (asset, amount) in assets {
        let asset_inputs = wallet
            .get_asset_inputs_for_amount(*asset, *amount, None)
            .await
            .unwrap();
        inputs.extend(asset_inputs);
        outputs.push(Output::Change {
            asset_id: *asset,
            amount: 0,
            to: wallet.address().into(),
        });
    }
    (inputs, outputs)
}

pub async fn preview_swap_exact_input(
    wallet: &WalletUnlocked,
    amount_in: u64,
    asset_in: AssetId,
    amount_out_min: u64,
    pools: Vec<PoolId>,
    deadline: u32,
    tx_policies: Option<TxPolicies>,
) -> Result<Vec<(u64, AssetId)>> {
    let amm_contract: MiraAmmContract<WalletUnlocked> =
        MiraAmmContract::new(ContractId::from_str(CONTRACT_ID).unwrap(), wallet.clone());
    let swap_exact_input_script = SwapScript::new(
        wallet.clone(),
        "/Users/walterphillips/BlockchainDev/triton/swap-script/out/debug/swap-script.bin",
    )
    .with_configurables(
        SwapScriptConfigurables::default()
            .with_AMM_CONTRACT_ID(amm_contract.contract_id().into())
            .unwrap(),
    );
    let (inputs, outputs) =
        get_transaction_inputs_outputs(wallet, &vec![(asset_in, amount_in)]).await;
    let assets = swap_exact_input_script
        .main(
            amount_in,
            asset_in,
            amount_out_min,
            pools,
            wallet.address().into(),
            deadline,
        )
        .with_tx_policies(tx_policies.unwrap_or_default())
        .with_contracts(&[&amm_contract])
        .with_inputs(inputs)
        .with_outputs(outputs)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .simulate(Execution::Realistic)
        .await
        .unwrap()
        .value;
    println!("assets: {:#?}", assets);
    Ok(assets)
}
