use fuels::accounts::provider::Provider;
use fuels::accounts::wallet::WalletUnlocked;
// use chrono::Utc;
use log::{error, info};
use trigon::recon::{stream_mira_events_pangea, sync_state};
use trigon::types::Event;

#[tokio::main]
async fn main() {
    info!("Starting Triton Arbitrage bot");
    let (tx, rx) = crossbeam_channel::unbounded::<Event>();

    let mut triton = trigon::triton::Triton::new();
    println!("triton: {:?}", triton.cycles.len());
    // Spawn a task to stream Mira events
    let wallet = WalletUnlocked::new_from_private_key(
        "0xf2331315499db8ff7868636f12863d514fd232dbbff1510043e78bc248c79e84"
            .parse()
            .unwrap(),
        Some(Provider::connect("mainnet.fuel.network").await.unwrap()),
    );

    sync_state(&mut triton, wallet).await;
    let event_tx = tx.clone();
    info!("Starting Mira event stream");
    tokio::spawn(async move {
        if let Err(e) = stream_mira_events_pangea(event_tx).await {
            error!("Error in stream_mira_events");
        }
    });

    loop {
        let event = rx.recv().unwrap();
        triton.process_event(event);
    }
}
