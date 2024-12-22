use ethers::utils::format_units;
use fuels::accounts::provider::Provider;
use fuels::accounts::wallet::WalletUnlocked;
use log::{error, info};
use triton::bundle;
use triton::calc::find_optimal_cycles;
use triton::recon::{stream_mira_events_pangea, sync_state};
use triton::types::Event;

#[tokio::main]
async fn main() {
    info!("Starting Triton Arbitrage bot");
    let (tx, rx) = crossbeam_channel::unbounded::<Event>();

    let mut triton = triton::triton::Triton::new();
    println!("triton: {:?}", triton.cycles.len());
    // Spawn a task to stream Mira events
    let wallet = WalletUnlocked::new_from_private_key(
        "0xf2331315499db8ff7868636f12863d514fd232dbbff1510043e78bc248c79e84"
            .parse()
            .unwrap(),
        Some(Provider::connect("mainnet.fuel.network").await.unwrap()),
    );

    sync_state(&mut triton, wallet.clone()).await;
    let event_tx = tx.clone();
    info!("Starting Mira event stream");
    tokio::spawn(async move {
        if let Err(_) = stream_mira_events_pangea(event_tx).await {
            error!("Error in stream_mira_events");
        }
    });

    loop {
        let event = rx.recv().unwrap();
        triton.process_event(event);
        let now = std::time::Instant::now();
        println!("triton: {:?}", triton.cycles.len());
        let cycles = find_optimal_cycles(&mut triton);
        let elapsed = now.elapsed().as_millis();
        println!("Cycle finding took {}ms", elapsed);
        if !cycles.is_empty() {
            println!(
                "Most profitable cycle: {:?} profit as u64: {:?}",
                cycles[0],
                format_units(cycles[0].profit, 6)
            );
            bundle::send_multi_hop(&wallet, cycles[0].clone()).await;
            let elapsed = now.elapsed().as_millis();
            println!("Cycle execution took {}ms", elapsed);
        }
    }
}
