use base64::Engine;
use crate::event::{AppEvent, Event, spawn};
use tokio::sync::mpsc;

pub fn fetch_online_shortlink(
    sender: mpsc::UnboundedSender<Event>,
    version: Option<algod_client::models::Version>,
    key: algod_client::models::ParticipationKey,
    account_incentive_eligible: bool,
    account_status: String,
) {
    spawn(async move {
        let network = version.map(|v| v.genesis_id).unwrap_or_else(|| "mainnet-v1.0".to_string());
        let lora_network = network.replace("-v1.0", "").replace("-v1", "");
        let lora_network = if lora_network == "dockernet" || lora_network == "tuinet" {
            "localnet"
        } else {
            &lora_network
        };

        let body = serde_json::json!({
            "account": key.address,
            "voteKeyB64": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key.key.vote_participation_key),
            "selectionKeyB64": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key.key.selection_participation_key),
            "stateProofKeyB64": key.key.state_proof_key.as_ref().map(|k| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(k)).unwrap_or_default(),
            "voteFirstValid": key.key.vote_first_valid,
            "voteLastValid": key.key.vote_last_valid,
            "keyDilution": key.key.vote_key_dilution,
            "network": lora_network,
        });

        let client = reqwest::Client::new();
        let res = client.post("http://b.nodekit.run/online")
            .json(&body)
            .send()
            .await;
        if let Ok(resp) = res {
            let json_res = resp.json::<serde_json::Value>().await;
            if let Ok(json) = json_res {
                if let Some(id) = json["id"].as_str() {
                    let mut suffix = "";
                    if account_incentive_eligible && account_status == "Online" {
                        suffix = "i";
                    }
                    let link = format!("https://b.nodekit.run/{}{}", id, suffix);
                    let _ = sender.send(Event::App(AppEvent::ShortlinkUpdate(link)));
                }
            }
        }
    });
}

pub fn fetch_offline_shortlink(
    sender: mpsc::UnboundedSender<Event>,
    version: Option<algod_client::models::Version>,
    address: String,
) {
    spawn(async move {
        let network = version.map(|v| v.genesis_id).unwrap_or_else(|| "mainnet-v1.0".to_string());
        let lora_network = network.replace("-v1.0", "").replace("-v1", "");
        let lora_network = if lora_network == "dockernet" || lora_network == "tuinet" {
            "localnet"
        } else {
            &lora_network
        };

        let body = serde_json::json!({
            "account": address,
            "network": lora_network,
        });

        let client = reqwest::Client::new();
        let res = client.post("http://b.nodekit.run/offline")
            .json(&body)
            .send()
            .await;
        if let Ok(resp) = res {
            let json_res = resp.json::<serde_json::Value>().await;
            if let Ok(json) = json_res {
                if let Some(id) = json["id"].as_str() {
                    let link = format!("https://b.nodekit.run/{}", id);
                    let _ = sender.send(Event::App(AppEvent::ShortlinkUpdate(link)));
                }
            }
        }
    });
}
