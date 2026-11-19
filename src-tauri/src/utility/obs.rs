use std::{path::PathBuf, time::Duration};

use futures_util::StreamExt;
use obws::{Client, events::Event};
use tokio::{sync::{mpsc, watch}, time::sleep};

use crate::structs::{ClipSettings, ConnectionState};

async fn listenOnce(tx: &mpsc::UnboundedSender<PathBuf>, state: &watch::Sender<ConnectionState>, settings: &ClipSettings) -> anyhow::Result<()> {
    let client = Client::connect("localhost", settings.port.unwrap_or(4455), settings.password.clone()).await?;
    let events = client.events()?;

    state.send_replace(ConnectionState::Connected);
    tokio::pin!(events);

    while let Some(event) = events.next().await {
        match event {
            Event::ReplayBufferSaved { path } => {
                let _ = tx.send(path);
            },
            Event::ServerStopped => break,
            _ => {},
        }
    }

    Ok(())
}

pub async fn obsListen(tx: mpsc::UnboundedSender<PathBuf>, state: watch::Sender<ConnectionState>, settings: ClipSettings) {
    let mut backoff = Duration::from_secs(5);

    loop {
        match listenOnce(&tx, &state, &settings).await {
            Ok(()) => backoff = Duration::from_secs(5),
            Err(e) => eprintln!("OBS WEBSOCKET ERR: {e:#}"),
        }

        state.send_replace(ConnectionState::Disconnected);
        sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(20));
    }
}
