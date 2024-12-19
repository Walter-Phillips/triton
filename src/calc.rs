use std::{cell::RefCell, cmp::Ordering, str::FromStr};
use fuels::types::{AssetId, U256};
use serde::Deserialize;
// use alloy_primitives::I256;
use ethers::types::I256;
use crate::types::Pool;

#[derive(Debug, Deserialize)]
pub struct NetPositiveCycle {
    pub profit: I256,
    pub optimal_in: U256,
    pub swap_amounts: Vec<U256>,
    pub cycle_ids: Vec<(AssetId, AssetId, bool)>,
}

impl Ord for NetPositiveCycle {
    fn cmp(&self, other: &Self) -> Ordering {
        other.profit.cmp(&self.profit)
    }
}

impl PartialOrd for NetPositiveCycle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NetPositiveCycle {}

// Ordering based on profit
impl PartialEq for NetPositiveCycle {
    fn eq(&self, other: &Self) -> bool {
        self.profit == other.profit
    }
}

pub fn find_optimal_cycles(triton: &mut crate::triton::Triton) -> Vec<NetPositiveCycle> {
    let eth_asset_id =
        AssetId::from_str("0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07")
            .unwrap();

    let mut net_profit_cycles = Vec::new();
    for cycle in &triton.cycles {
        let pairs = cycle
            .cycle
            .iter()
            .filter_map(|pair| triton.pools.get(&pair.index))
            .collect::<Vec<&RefCell<Pool>>>();
        let pairs_clone = pairs.clone();
        log::debug!("getting profit");
        let profit_function =
            move |amount_in: U256| -> I256 { get_profit(eth_asset_id, amount_in, &pairs_clone) };

        log::debug!("maximizing profit");
        let optimal = maximize_profit(
            U256::one(),
            U256::from_dec_str("10000000000000000000000").unwrap(),
            U256::from_dec_str("10").unwrap(),
            profit_function,
        );

        log::debug!("getting profit with amount");
        let (profit, swap_amounts) = get_profit_with_amount(eth_asset_id, optimal, &pairs);
        let mut cycle_internal = Vec::new();
        for pair in pairs {
            let is_stable = pair.borrow().fee_rate < U256::from(300);
            cycle_internal.push((pair.borrow().from, pair.borrow().to, is_stable));
        }
        println!("profit: {}", profit);
        if profit > I256::one() {
            let net_positive_cycle = NetPositiveCycle {
                profit,
                optimal_in: optimal,
                cycle_ids: cycle_internal,
                swap_amounts,
            };
            net_profit_cycles.push(net_positive_cycle);
        }
    }
    println!("net_profit_cycles: {:#?}", net_profit_cycles);
    net_profit_cycles.sort();
    net_profit_cycles.into_iter().take(5).collect()
}

// Quadratic search
fn maximize_profit(
    mut domain_min: U256,
    mut domain_max: U256,
    lowest_delta: U256,
    f: impl Fn(U256) -> I256,
) -> U256 {
    loop {
        if domain_max > domain_min {
            if (domain_max - domain_min) > lowest_delta {
                let mid = (domain_min + domain_max) / 2;

                let lower_mid = (mid + domain_min) / 2;
                let upper_mid = (mid + domain_max) / 2;

                let f_output_lower = f(lower_mid);
                let f_output_upper = f(upper_mid);

                if f_output_lower > f_output_upper {
                    domain_max = mid;
                } else {
                    domain_min = mid;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    log::debug!("domain_min: {}, domain_max: {}", domain_min, domain_max);
    (domain_max + domain_min) / 2
}

pub fn get_profit_with_amount(
    token_in: AssetId,
    amount_in: U256,
    pairs: &Vec<&RefCell<Pool>>,
) -> (I256, Vec<U256>) {
    let mut amount_out: U256 = amount_in;
    let mut token_in = token_in;
    let mut amounts = Vec::with_capacity(pairs.len() + 1);
    amounts.push(amount_in);
    for pair in pairs {
        let pair = pair.borrow();
        let fees;
        let (reserve0, reserve1) = if pair.to == token_in {
            fees = pair.fee_rate;
            (pair.reserve_0, pair.reserve_1)
        } else {
            fees = pair.fee_rate;
            (pair.reserve_1, pair.reserve_0)
        };
        amount_out = get_amount_out(amount_out, reserve0, reserve1, fees, U256::from(0));
        amounts.push(amount_out);
        token_in = if pair.to == token_in {
            pair.from
        } else {
            pair.to
        };
    }

    let binding = amount_out.to_string();
    let amount_out_as_str = binding.as_str();
    let binding = amount_in.to_string();
    let amount_in_as_str = binding.as_str();

    (    I256::from_str(amount_out_as_str).unwrap() - I256::from_str(amount_in_as_str).unwrap(), amounts)
}

pub fn get_profit(token_in: AssetId, amount_in: U256, pairs: &Vec<&RefCell<Pool>>) -> I256 {
    let mut amount_out: U256 = amount_in;
    let mut token_in = token_in;

    for pair in pairs {
        let pair = pair.borrow();
        let fees;
        let (reserve0, reserve1) = if pair.to == token_in {
            fees = pair.fee_rate;
            (pair.reserve_0, pair.reserve_1)
        } else {
            fees = pair.fee_rate;
            (pair.reserve_1, pair.reserve_0)
        };

        log::debug!(
            "amount_out: {}, reserve0: {}, reserve1: {}, fees: {}",
            amount_out, reserve0, reserve1, fees
        );

        amount_out = get_amount_out_with_saturation(amount_out, reserve0, reserve1, fees);
        token_in = if pair.to == token_in {
            pair.from
        } else {
            pair.to
        };
    }

    log::debug!("Final amount_out: {}, amount_in: {}", amount_out, amount_in);
    let binding = amount_out.to_string();
    let amount_out_as_str = binding.as_str();
    let binding = amount_in.to_string();
    let amount_in_as_str = binding.as_str();
    I256::from_str(amount_out_as_str).unwrap() - I256::from_str(amount_in_as_str).unwrap()
}

pub fn get_amount_out(
    a_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fees: U256,
    router_fee: U256,
) -> U256 {
    if a_in == U256::zero() {
        return U256::zero();
    }
    let a_in_with_fee = a_in.saturating_mul(router_fee);
    let a_out = a_in_with_fee.saturating_mul(reserve_out)
        / U256::from(10000)
            .saturating_mul(reserve_in)
            .saturating_add(a_in_with_fee);

    a_out - a_out.saturating_mul(fees) / U256::from(10000)
}
fn get_amount_out_with_saturation(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_rate: U256,
) -> U256 {
    if amount_in == U256::zero() || reserve_in == U256::zero() || reserve_out == U256::zero() {
        return U256::zero();
    }

    let fee_denominator = U256::from(1000);

    // Use saturating arithmetic
    let amount_fee = amount_in.saturating_mul(fee_rate) / fee_denominator;
    let effective_amount_in = amount_in.saturating_sub(amount_fee);

    let numerator = effective_amount_in.saturating_mul(reserve_out);
    let denominator = reserve_in.saturating_add(effective_amount_in);

    if denominator == U256::zero() {
        return U256::zero();
    }

    numerator / denominator
}
