// use crate::calc::NetPositiveCycle;
// use fuels::types::transaction::TxPolicies;
// use mira_v1::mira_amm::MiraAmmReadOnly;

// pub async fn send_multi_hop(mira: MiraAmmReadOnly, profitable_cycle: NetPositiveCycle){
//     let policies = TxPolicies::default();
//     let tx = mira.preview_swap_exact_input(profitable_cycle.optimal_in.as_u64(),profitable_cycle.cycle_ids[0].0, profitable_cycle.profit.as_u64(),profitable_cycle.cycle_ids,999999999,Some(policies)).await.unwrap();
//     println!("tx: {:#?}", tx);
// }
