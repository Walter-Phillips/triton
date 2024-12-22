script;

pub mod interface;
pub mod math;
pub mod utils;

use interface::{MiraAMM, PoolId};
use math::get_amounts_out;
use utils::check_deadline;
use std::{asset::transfer, bytes::Bytes};

configurable {
    AMM_CONTRACT_ID: ContractId = ContractId::zero(),
}

fn main(
    amount_in: u64, // Initial input token amount.
    asset_in: AssetId, // Input asset type.
    amount_out_min: u64, // Minimum output amount expected.
    pools: Vec<PoolId>, // Ordered list of pools to traverse for arbitrage.
    recipient: Identity, // Address to receive final tokens.
    deadline: u32, // Deadline for transaction validity.
) -> Vec<(u64, AssetId)> {
    // Ensure the transaction is within the allowed time window.
    check_deadline(deadline);

    // Compute the output amounts for each pool in the swap path.
    let amounts_out = get_amounts_out(AMM_CONTRACT_ID, amount_in, asset_in, pools);

    // Ensure the final output meets the minimum amount required.
    let last_amount_out = amounts_out.get(amounts_out.len() - 1).unwrap();
    require(
        last_amount_out.0 >= amount_out_min,
        "Insufficient output amount",
    );

    // Transfer the initial input asset to the AMM contract.
    transfer(Identity::ContractId(AMM_CONTRACT_ID), asset_in, amount_in);

    // Obtain the AMM contract's ABI for making swaps.
    let amm = abi(MiraAMM, AMM_CONTRACT_ID.into());

    // Iterate over the pools to execute swaps in sequence.
    let mut i = 0;
    while i < pools.len() {
        let pool_id = pools.get(i).unwrap(); // Current pool in the path.
        let (amount_out, asset_out) = amounts_out.get(i + 1).unwrap(); // Expected output for this pool.

        // Determine the recipient for the output tokens.
        let to = if i == pools.len() - 1 {
            recipient // Final recipient if it's the last pool.
        } else {
            Identity::ContractId(AMM_CONTRACT_ID) // Forward tokens back to AMM for the next swap.
        };

        // Identify which token is being swapped out in the current pool.
        let (amount_0_out, amount_1_out) = if asset_out == pool_id.0 {
            (amount_out, 0) // Swap output on token 0.
        } else {
            (0, amount_out) // Swap output on token 1.
        };

        // Execute the swap for the current pool.
        amm.swap(pool_id, amount_0_out, amount_1_out, to, Bytes::new());

        i += 1; // Proceed to the next pool.
    }

    // Return the computed amounts out for each pool.
    amounts_out
}
