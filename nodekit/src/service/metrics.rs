use tokio::sync::mpsc;
use crate::event::{Event, AppEvent, Metrics, spawn, sleep};

pub fn spawn_metrics_loop(
    sender: mpsc::UnboundedSender<Event>,
    url: String,
    token: String,
) {
    let metrics_url = format!("{}/metrics", url.trim_end_matches('/'));
    spawn(async move {
        let client = reqwest::Client::new();
        loop {
            let res = client.get(&metrics_url)
                .header("X-Algo-API-Token", &token)
                .send()
                .await;
            if let Ok(resp) = res {
                if let Ok(text) = resp.text().await {
                    let mut metrics = Metrics::default();
                    let mut tx_count = 0;
                    for line in text.lines() {
                        if line.starts_with("algod_network_incoming_peers") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.peers_ws += val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("algod_network_outgoing_peers") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.peers_ws += val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("libp2p_rcmgr_connections{dir=\"inbound\",scope=\"system\"}") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.peers_p2p += val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("libp2p_rcmgr_connections{dir=\"outbound\",scope=\"system\"}") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.peers_p2p += val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("algod_network_received_bytes_total") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.rx = val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("algod_network_sent_bytes_total") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.tx = val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("algod_network_p2p_received_bytes_total") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.rx_p2p = val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("algod_network_p2p_sent_bytes_total") {
                            if let Some(val) = line.split_whitespace().last() {
                                metrics.tx_p2p = val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        } else if line.starts_with("algod_ledger_transactions_total") {
                            if let Some(val) = line.split_whitespace().last() {
                                tx_count = val.parse::<f64>().unwrap_or(0.0) as u64;
                            }
                        }
                    }
                    metrics.tps = tx_count as f64; // Temporarily store total tx count to calculate TPS later
                    let _ = sender.send(Event::App(AppEvent::MetricsUpdate(metrics)));
                }
            } else {
                let _ = sender.send(Event::App(AppEvent::NodeStatusUpdate(crate::event::NodeStatus::Disconnected)));
            }
            sleep(std::time::Duration::from_secs(2)).await;
        }
    });
}
