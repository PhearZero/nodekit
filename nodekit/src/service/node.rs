use tokio::sync::mpsc;
use crate::event::{Event, AppEvent};

pub fn spawn_node_loop(
    sender: mpsc::UnboundedSender<Event>,
    client: algod_client::AlgodClient,
    initial_version: Option<algod_client::models::Version>,
) {
    tokio::spawn(async move {
        let mut last_round = None;
        let mut round_times = std::collections::VecDeque::with_capacity(100);
        let mut last_block_time: Option<std::time::Instant> = None;
        let mut current_version = initial_version;
        let mut current_node_status = crate::event::NodeStatus::Stable;

        // Fetch version info once on startup if not already present
        if current_version.is_none() {
            loop {
                match client.get_version().await {
                    Ok(version) => {
                        current_version = Some(version.clone());
                        let channel = version.build.channel.clone();
                        let major = version.build.major;
                        let minor = version.build.minor;
                        let build = version.build.build_number;
                        let _ = sender.send(Event::App(AppEvent::VersionUpdate(Box::new(version))));

                        // Check for updates
                        let sender_clone = sender.clone();
                        tokio::spawn(async move {
                            let update_url = format!("https://api.github.com/repos/algorand/go-algorand/releases/latest");
                            let client = reqwest::Client::builder()
                                .user_agent("nodekit")
                                .build()
                                .unwrap();
                            if let Ok(resp) = client.get(update_url).send().await {
                                if let Ok(release) = resp.json::<serde_json::Value>().await {
                                    if let Some(tag) = release["tag_name"].as_str() {
                                        // tag is usually "vX.Y.Z-stable" or similar
                                        let current = format!("v{}.{}.{}-{}", major, minor, build, channel);
                                        if tag != current {
                                            let _ = sender_clone.send(Event::App(AppEvent::UpdateAvailable(true)));
                                        }
                                    }
                                }
                            }
                        });
                        break;
                    }
                    Err(e) => {
                        let err_str = format!("{:?}", e);
                        if err_str.contains("503") || err_str.contains("Service Unavailable") {
                            // Node is likely starting up or in fast catchup, retry version fetch later
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            continue;
                        }
                        // For other errors, we might want to report it but maybe not immediately exit
                        let _ = sender.send(Event::App(AppEvent::Error(format!("Error fetching version: {:?}", e))));
                        break;
                    }
                }
            }
        }

        loop {
            let status_res = if current_node_status == crate::event::NodeStatus::FastCatchup {
                client.get_status().await.map(|s| {
                    let mut wait_for_block = algod_client::models::WaitForBlock::default();
                    wait_for_block.catchup_time = s.catchup_time;
                    wait_for_block.last_round = s.last_round;
                    wait_for_block.last_version = s.last_version;
                    wait_for_block.next_version = s.next_version;
                    wait_for_block.next_version_round = s.next_version_round;
                    wait_for_block.next_version_supported = s.next_version_supported;
                    wait_for_block.stopped_at_unsupported_round = s.stopped_at_unsupported_round;
                    wait_for_block.time_since_last_round = s.time_since_last_round;
                    wait_for_block.catchpoint = s.catchpoint;
                    wait_for_block.catchpoint_acquired_blocks = s.catchpoint_acquired_blocks;
                    wait_for_block.catchpoint_processed_accounts = s.catchpoint_processed_accounts;
                    wait_for_block.catchpoint_processed_kvs = s.catchpoint_processed_kvs;
                    wait_for_block.catchpoint_total_accounts = s.catchpoint_total_accounts;
                    wait_for_block.catchpoint_total_blocks = s.catchpoint_total_blocks;
                    wait_for_block.catchpoint_total_kvs = s.catchpoint_total_kvs;
                    wait_for_block.catchpoint_verified_accounts = s.catchpoint_verified_accounts;
                    wait_for_block.catchpoint_verified_kvs = s.catchpoint_verified_kvs;
                    wait_for_block
                })
            } else if let Some(round) = last_round {
                client.wait_for_block(round).await
            } else {
                client.get_status().await.map(|s| {
                    let mut wait_for_block = algod_client::models::WaitForBlock::default();
                    wait_for_block.catchup_time = s.catchup_time;
                    wait_for_block.last_round = s.last_round;
                    wait_for_block.last_version = s.last_version;
                    wait_for_block.next_version = s.next_version;
                    wait_for_block.next_version_round = s.next_version_round;
                    wait_for_block.next_version_supported = s.next_version_supported;
                    wait_for_block.stopped_at_unsupported_round = s.stopped_at_unsupported_round;
                    wait_for_block.time_since_last_round = s.time_since_last_round;
                    wait_for_block.catchpoint = s.catchpoint;
                    wait_for_block.catchpoint_acquired_blocks = s.catchpoint_acquired_blocks;
                    wait_for_block.catchpoint_processed_accounts = s.catchpoint_processed_accounts;
                    wait_for_block.catchpoint_processed_kvs = s.catchpoint_processed_kvs;
                    wait_for_block.catchpoint_total_accounts = s.catchpoint_total_accounts;
                    wait_for_block.catchpoint_total_blocks = s.catchpoint_total_blocks;
                    wait_for_block.catchpoint_total_kvs = s.catchpoint_total_kvs;
                    wait_for_block.catchpoint_verified_accounts = s.catchpoint_verified_accounts;
                    wait_for_block.catchpoint_verified_kvs = s.catchpoint_verified_kvs;
                    wait_for_block
                })
            };

            match status_res {
                Ok(status) => {
                    let now = std::time::Instant::now();
                    if let Some(last_time) = last_block_time {
                        let duration = now.duration_since(last_time);
                        round_times.push_back(duration.as_millis() as u64);
                        if round_times.len() > 100 {
                            round_times.pop_front();
                        }
                        let avg = round_times.iter().sum::<u64>() / round_times.len() as u64;
                        if round_times.len() >= 3 {
                            let _ = sender.send(Event::App(AppEvent::AvgRoundTimeUpdate(avg)));
                        }
                    }
                    last_block_time = Some(now);

                    last_round = Some(status.last_round);
                    
                    let node_status = if status.catchpoint.as_ref().map_or(false, |cp| !cp.is_empty()) {
                        crate::event::NodeStatus::FastCatchup
                    } else if status.catchup_time > 0 {
                        crate::event::NodeStatus::Syncing
                    } else {
                        crate::event::NodeStatus::Stable
                    };
                    current_node_status = node_status.clone();

                    let _ = sender.send(Event::App(AppEvent::NodeStatusUpdate(node_status.clone())));
                    
                    // Check if lagging
                    if node_status == crate::event::NodeStatus::Syncing {
                        let sender_clone = sender.clone();
                        let version_clone = current_version.clone();
                        let current_round = status.last_round;
                        tokio::spawn(async move {
                            let network = version_clone.map(|v| v.genesis_id).unwrap_or_else(|| "mainnet-v1.0".to_string());
                            let network_short = if network.contains("testnet") {
                                "testnet"
                            } else if network.contains("betanet") {
                                "betanet"
                            } else if network.contains("fnet") {
                                "fnet"
                            } else {
                                "mainnet"
                            };

                            let url = match network_short {
                                "fnet" => "https://fnet-catchpoints.algorand.green/latest",
                                "betanet" => "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/betanet/latest.catchpoint",
                                "testnet" => "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/testnet/latest.catchpoint",
                                _ => "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/mainnet/latest.catchpoint",
                            };

                            if let Ok(resp) = reqwest::get(url).await {
                                if let Ok(catchpoint) = resp.text().await {
                                    let catchpoint = catchpoint.trim();
                                    if let Some(sharp_idx) = catchpoint.find('#') {
                                        if let Ok(catchpoint_round) = catchpoint[..sharp_idx].parse::<u64>() {
                                            if catchpoint_round > current_round && catchpoint_round - current_round > 30_000 {
                                                let _ = sender_clone.send(Event::App(AppEvent::ShowModal(crate::event::ModalType::Lagging)));
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }

                    let _ = sender.send(Event::App(AppEvent::AlgodUpdate(Box::new(status))));

                    if node_status == crate::event::NodeStatus::FastCatchup {
                        // During Fast Catchup, we can fetch keys but not account information
                        if let Ok(keys) = client.get_participation_keys().await {
                            let _ = sender.send(Event::App(AppEvent::KeysUpdate(keys.clone())));
                            
                            // Create dummy accounts just to show addresses in the list
                            let mut accounts = Vec::new();
                            let mut unique_addresses = std::collections::HashSet::new();
                            for key in &keys {
                                unique_addresses.insert(key.address.clone());
                            }
                            for address in unique_addresses {
                                let mut account = algod_client::models::Account::default();
                                account.address = address;
                                accounts.push(account);
                            }
                            let _ = sender.send(Event::App(AppEvent::AccountsUpdate(accounts)));
                        }

                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }

                    // After a successful status update (new block), fetch keys
                    match client.get_participation_keys().await {
                        Ok(keys) => {
                            let _ = sender.send(Event::App(AppEvent::KeysUpdate(keys.clone())));

                            // Fetch account details for each unique address
                            let mut unique_addresses = std::collections::HashSet::new();
                            for key in &keys {
                                unique_addresses.insert(key.address.clone());
                            }

                            let mut accounts = Vec::new();
                            for address in unique_addresses {
                                match client.account_information(&address, None).await {
                                    Ok(account) => accounts.push(account),
                                    Err(e) => {
                                        let err_str = format!("{:?}", e);
                                        if err_str.contains("503") || err_str.contains("Service Unavailable") || err_str.contains("Unexpected text response") {
                                            // Ignore 503 or non-JSON responses during potential transition to/from fast catchup
                                            continue;
                                        }
                                        let _ = sender.send(Event::App(AppEvent::Error(format!("Error fetching account info for {}: {:?}", address, e))));
                                    }
                                }
                            }
                            if !accounts.is_empty() {
                                let _ = sender.send(Event::App(AppEvent::AccountsUpdate(accounts)));
                            }
                        }
                        Err(e) => {
                            // Handle the case where the node returns null for participation keys (no keys present)
                            // This manifests as a Serde error: "invalid type: null, expected a sequence at line 1 column 4"
                            let err_str = format!("{:?}", e);
                            if err_str.contains("invalid type: null, expected a sequence") {
                                let _ = sender.send(Event::App(AppEvent::KeysUpdate(Vec::new())));
                            } else if err_str.contains("503") || err_str.contains("Service Unavailable") || err_str.contains("Unexpected text response") {
                                // Ignore 503 or non-JSON responses during potential transition
                            } else {
                                let _ = sender.send(Event::App(AppEvent::Error(format!("Error fetching participation keys: {:?}", e))));
                            }
                        }
                    }
                }
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("503") || err_str.contains("Service Unavailable") || err_str.contains("Unexpected text response") {
                        // On 503 or non-JSON status fetch, just wait and retry without showing error modal
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    let _ = sender.send(Event::App(AppEvent::Error(format!("Error fetching status: {:?}", e))));
                    // On other errors, wait a bit before retrying to avoid spamming
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        }
    });
}

pub fn start_fast_catchup(
    sender: mpsc::UnboundedSender<Event>,
    base_url: String,
    api_token: String,
    version_opt: Option<algod_client::models::Version>,
) {
    tokio::spawn(async move {
        let network = version_opt.map(|v| v.genesis_id).unwrap_or_else(|| "mainnet-v1.0".to_string());
        let network_short = if network.contains("testnet") {
            "testnet"
        } else if network.contains("betanet") {
            "betanet"
        } else if network.contains("fnet") {
            "fnet"
        } else {
            "mainnet"
        };

        let url = match network_short {
            "fnet" => "https://fnet-catchpoints.algorand.green/latest",
            "betanet" => "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/betanet/latest.catchpoint",
            "testnet" => "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/testnet/latest.catchpoint",
            _ => "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/mainnet/latest.catchpoint",
        };

        let catchpoint = match reqwest::get(url).await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to fetch catchpoint: HTTP {}", resp.status()))));
                    return;
                }
                match resp.text().await {
                    Ok(text) => text.trim().to_string(),
                    Err(e) => {
                        let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to read catchpoint: {:?}", e))));
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to fetch catchpoint: {:?}", e))));
                return;
            }
        };

        if catchpoint.is_empty() {
            let _ = sender.send(Event::App(AppEvent::Error("Fetched catchpoint is empty".to_string())));
            return;
        }

        let encoded_catchpoint = url::form_urlencoded::byte_serialize(catchpoint.as_bytes()).collect::<String>();
        let base_url = base_url.trim_end_matches('/');
        let start_catchup_url = format!("{}/v2/catchup/{}", base_url, encoded_catchpoint);

        let http_client = reqwest::Client::new();
        match http_client.post(&start_catchup_url)
            .header("X-Algo-API-Token", api_token)
            .send().await 
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let err_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to start catchup: {}", err_text))));
                }
            }
            Err(e) => {
                let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to send start catchup request: {:?}", e))));
            }
        }
    });
}
