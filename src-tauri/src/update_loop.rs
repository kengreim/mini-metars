use crate::state::AppState;
use crate::vatis;
use cached::Cached;
use cached::TimedCache;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, warn};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};

pub async fn vatsim_datafeed_loop(app_handle: AppHandle) {
    let Some(state) = app_handle.try_state::<Arc<AppState>>() else {
        error!("Could not retrieve state to initialize VATSIM datafeed update loop");
        return;
    };

    let Ok(client) = state.get_vatsim_client().await else {
        const E: &str = "VATSIM API client not initialized";
        error!("Error fetching VATSIM data: {E}");
        return;
    };

    debug!("Starting VATSIM datafeed update loop");
    loop {
        let mut sleep_duration = Duration::from_secs(15);
        match client.get_v3_data().await {
            Ok(data) => {
                let is_duplicate = state
                    .latest_vatsim_data
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map_or_else(
                        || false,
                        |old_data| old_data.general.update == data.general.update,
                    );

                if is_duplicate {
                    debug!(
                        "Fetched duplicate VATSIM datafeed: {}",
                        &data.general.update
                    );
                    sleep_duration = Duration::from_secs(1);
                } else {
                    debug!("Fetched new VATSIM datafeed: {}", &data.general.update);
                    *state.latest_vatsim_data.lock().unwrap() = Some(data);
                }
            }
            Err(e) => {
                error!("Error fetching VATSIM data: {e}");
            }
        }

        tokio::time::sleep(sleep_duration).await;
    }
}

pub async fn vatis_websocket_loop(app_handle: AppHandle) {
    const WS_URL: &str = "ws://127.0.0.1:49082/";
    const PING_INTERVAL_SECONDS: u64 = 30;
    const WS_TRY_CONNECT_INTERVAL_SECONDS: u64 = 30;
    const WS_TRY_RECONNECT_INTERVAL_SECONDS: u64 = 1;
    const CACHE_TTL_SECONDS: u64 = 60 * 3;

    let Some(state) = app_handle.try_state::<Arc<AppState>>() else {
        error!("Could not retrieve state to initialize vATIS weboscket update loop");
        return;
    };

    *state.vatis_data.lock().unwrap() = Some(TimedCache::with_lifespan(CACHE_TTL_SECONDS));

    debug!("Starting vATIS websocket update loop");
    loop {
        let mut sleep_duration = Duration::from_secs(WS_TRY_CONNECT_INTERVAL_SECONDS);
        if let Ok((ws_stream, _)) = connect_async(WS_URL).await {
            debug!("Successfully connected to vATIS websocket: {WS_URL}");
            sleep_duration = Duration::from_secs(WS_TRY_RECONNECT_INTERVAL_SECONDS);
            let (mut write, mut read) = ws_stream.split();
            let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECONDS));

            loop {
                tokio::select! {
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(msg))) => {
                                match serde_json::from_str::<vatis::AtisUpdateMessage>(msg.as_str()) {
                                    Ok(update) => {
                                        debug!("Received vATIS update message for station {:?} with letter {:?}", update.value.station, update.value.atis_letter);
                                        if let Some(station) = update.value.station.as_ref() {
                                            match *state.vatis_data.lock().unwrap() {
                                                Some(ref mut map) => {
                                                    map.cache_set(station.to_string(), update);
                                                }
                                                _ => {
                                                    warn!("vATIS update hashmap not initialized")
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        warn!("Error deserializing vATIS update message: {e}")
                                    }
                                }
                            },
                            Some(Ok(Message::Ping(bytes))) => {
                                debug!("Received ping message from vATIS websocket");
                                if let Err(e) = write.send(Message::Pong(bytes)).await {
                                    warn!("Error sending pong message to vATIS websocket: {e}");
                                } else {
                                    debug!("Responded with pong message to vATIS websocket");
                                }
                            },
                            Some(Ok(m)) => {
                                debug_message(&m);
                            },
                            Some(Err(e)) => {
                                warn!("Error receiving message from vATIS websocket: {e}");
                                break;
                            },
                            None => {
                                debug!("vATIS websocket connection closed. Trying to reconnect");
                                break;
                            },
                        }
                    }
                    _ = interval.tick() => {
                        if let Err(e) = write.send(Message::Ping(Bytes::default())).await {
                            warn!("Error sending ping message to vATIS websocket: {e}");
                        } else {
                            debug!("Sent ping message to vATIS websocket");
                        }
                    }
                }
            }
        } else {
            error!("Could not initialize vATIS websocket connection");
        }

        tokio::time::sleep(sleep_duration).await;
    }
}

fn debug_message(msg: &Message) {
    let str = match msg {
        Message::Close(_) => "close",
        Message::Pong(_) => "pong",
        Message::Binary(_) => "binary",
        Message::Frame(_) => "frame",
        Message::Text(_) => "text",
        Message::Ping(_) => "ping ",
    };
    debug!("Received {str} message from vATIS websocket");
}
