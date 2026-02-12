use crate::event::{AppEvent, Event, EventHandler, ModalType, Metrics};
use base64::Engine;
use crate::ui::viewport::ViewportComponent;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Page {
    #[default]
    Accounts,
    Keys,
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Algod URL
    pub url: String,
    /// Algod Token
    pub token: String,
    /// Current page.
    pub current_page: Page,
    /// Active modal.
    pub active_modal: Option<ModalType>,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    /// Algod client
    pub client: algod_client::AlgodClient,
    /// Latest status
    pub status: Option<algod_client::models::WaitForBlock>,
    /// Version info
    pub version: Option<algod_client::models::Version>,
    /// Update available
    pub update_available: bool,
    /// Accounts
    pub accounts: Vec<algod_client::models::Account>,
    /// Participation keys
    pub keys: Vec<algod_client::models::ParticipationKey>,
    /// Selected account address
    pub selected_account: Option<String>,
    /// Selected account index
    pub selected_account_index: usize,
    /// Selected key index
    pub selected_key_index: usize,
    /// Selected key
    pub selected_key: Option<algod_client::models::ParticipationKey>,
    /// Was the selected key just generated?
    pub selected_key_just_generated: bool,
    /// Last error message
    pub last_error: Option<String>,
    /// Key info display mode
    pub key_info_mode: crate::event::KeyInfoMode,
    /// Average round time in milliseconds
    pub avg_round_time: u64,
    /// Node status (Stable, Syncing, FastCatchup)
    pub node_status: crate::event::NodeStatus,
    /// Has the first key fetch been completed?
    pub first_key_fetch_done: bool,
    /// Generation step
    pub generate_step: crate::app::GenerateStep,
    /// Generation address
    pub generate_address: String,
    /// Generation duration
    pub generate_duration: String,
    /// Generation range
    pub generate_range: crate::app::GenerateRange,
    /// Generation error
    pub generate_error: Option<String>,
    /// Has the user dismissed the lagging modal?
    pub lagging_dismissed: bool,
    /// Metrics
    pub metrics: Metrics,
    /// P2P enabled
    pub p2p_enabled: bool,
    /// P2P Hybrid enabled
    pub p2p_hybrid_enabled: bool,
    /// Current shortlink for selected key
    pub current_shortlink: Option<String>,
    /// Incentives disabled
    pub incentives_disabled: bool,
    /// Success message for key info modal
    pub key_info_success_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GenerateStep {
    #[default]
    Address,
    Duration,
    Waiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GenerateRange {
    #[default]
    Day,
    Month,
    Round,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(url: &str, token: &str, data_dir: Option<&str>, incentives_disabled: bool) -> Self {
        let http_client = algokit_http_client::DefaultHttpClient::with_header(
            url,
            "X-Algo-API-Token",
            token,
        ).expect("Failed to create HTTP client");
        let client = algod_client::AlgodClient::new(std::sync::Arc::new(http_client));

        let mut p2p_enabled = false;
        let mut p2p_hybrid_enabled = false;

        if let Some(dir) = data_dir {
            let config_path = std::path::Path::new(dir).join("config.json");
            if let Ok(content) = std::fs::read_to_string(config_path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                    p2p_enabled = config["EnableP2P"].as_bool().unwrap_or(false);
                    p2p_hybrid_enabled = config["EnableP2PHybridMode"].as_bool().unwrap_or(false);
                }
            }
        }

        Self {
            running: true,
            url: url.to_string(),
            token: token.to_string(),
            current_page: Page::default(),
            active_modal: None,
            counter: 0,
            events: EventHandler::new(),
            client,
            status: None,
            version: None,
            update_available: false,
            accounts: Vec::new(),
            keys: Vec::new(),
            selected_account: None,
            selected_account_index: 0,
            selected_key_index: 0,
            selected_key: None,
            selected_key_just_generated: false,
            last_error: None,
            key_info_mode: crate::event::KeyInfoMode::default(),
            avg_round_time: 2900, // Default to 2.9s
            node_status: crate::event::NodeStatus::default(),
            first_key_fetch_done: false,
            generate_step: crate::app::GenerateStep::default(),
            generate_address: String::new(),
            generate_duration: String::new(),
            generate_range: crate::app::GenerateRange::default(),
            generate_error: None,
            lagging_dismissed: false,
            metrics: Metrics::default(),
            p2p_enabled,
            p2p_hybrid_enabled,
            current_shortlink: None,
            incentives_disabled,
            key_info_success_message: None,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let sender = self.events.get_sender();
        let client = self.client.clone();
        let initial_version = self.version.clone();
        let metrics_url = format!("{}/metrics", self.url.trim_end_matches('/'));
        let metrics_token = self.token.clone();

        // Spawn metrics polling loop
        let metrics_sender = sender.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            loop {
                if let Ok(resp) = client.get(&metrics_url)
                    .header("X-Algo-API-Token", &metrics_token)
                    .send()
                    .await {
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
                        let _ = metrics_sender.send(Event::App(AppEvent::MetricsUpdate(metrics)));
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

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

        while self.running {
            terminal.draw(|frame| {
                let viewport = ViewportComponent::new(&self);
                frame.render_widget(&viewport, frame.area());
            })?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Quit => self.quit(),
                    AppEvent::ShowAccounts => self.current_page = Page::Accounts,
                    AppEvent::ShowKeys => self.current_page = Page::Keys,
                    AppEvent::ShowModal(modal) => {
                        if modal == ModalType::Lagging && (self.lagging_dismissed || self.active_modal.is_some()) {
                            continue;
                        }
                        self.active_modal = Some(modal);
                    }
                    AppEvent::Error(err) => {
                        self.last_error = Some(err);
                        self.active_modal = Some(ModalType::Exception);
                    }
                    AppEvent::HideModal => self.active_modal = None,
                    AppEvent::AlgodUpdate(status) => self.status = Some(*status),
                    AppEvent::VersionUpdate(version) => self.version = Some(*version),
                    AppEvent::UpdateAvailable(available) => self.update_available = available,
                    AppEvent::AccountsUpdate(accounts) => {
                        // Detect transitions for selected key
                        if let Some(key) = &self.selected_key {
                            if let Some(old_acct) = self.accounts.iter().find(|a| a.address == key.address) {
                                if let Some(new_acct) = accounts.iter().find(|a| a.address == key.address) {
                                    if old_acct.status == "Offline" && new_acct.status == "Online" {
                                        self.key_info_success_message = Some("Node successfully taken online".to_string());
                                    } else if old_acct.status == "Online" && new_acct.status == "Offline" {
                                        self.key_info_success_message = Some("Node successfully taken offline".to_string());
                                    }
                                }
                            }
                        }
                        self.accounts = accounts;
                    }
                    AppEvent::KeysUpdate(mut keys) => {
                        // Sort keys by expiration time (vote_last_valid)
                        keys.sort_by(|a, b| a.key.vote_last_valid.cmp(&b.key.vote_last_valid));
                        self.keys = keys;

                        // Navigate back to accounts if no keys left on Keys page
                        if self.current_page == Page::Keys {
                            let filtered_keys_count = if let Some(addr) = &self.selected_account {
                                self.keys.iter().filter(|k| &k.address == addr).count()
                            } else {
                                self.keys.len()
                            };
                            if filtered_keys_count == 0 {
                                self.current_page = Page::Accounts;
                            } else if self.selected_key_index >= filtered_keys_count {
                                self.selected_key_index = filtered_keys_count.saturating_sub(1);
                            }
                        }

                        // Show Partkey modal if no keys exist and it's the first fetch
                        if !self.first_key_fetch_done {
                            self.first_key_fetch_done = true;
                            if self.keys.is_empty() {
                                self.active_modal = Some(ModalType::Partkey);
                            }
                        }
                    }
                    AppEvent::SelectKey(key) => {
                        let is_active = self.is_key_active(&key);
                        self.selected_key = Some(key.clone());
                        self.selected_key_just_generated = false;
                        self.current_shortlink = None;
                        self.key_info_success_message = None;

                        if !is_active {
                            let sender = self.events.get_sender();
                            let version = self.version.clone();
                            let key_clone = key.clone();
                            let accounts = self.accounts.clone();
                            tokio::spawn(async move {
                                let network = version.map(|v| v.genesis_id).unwrap_or_else(|| "mainnet-v1.0".to_string());
                            let lora_network = network
                                .replace("-v1.0", "")
                                .replace("-v1", "");
                            let lora_network = if lora_network == "dockernet" || lora_network == "tuinet" {
                                "localnet"
                            } else {
                                &lora_network
                            };

                            let body = serde_json::json!({
                                "account": key_clone.address,
                                "voteKeyB64": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key_clone.key.vote_participation_key),
                                "selectionKeyB64": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key_clone.key.selection_participation_key),
                                "stateProofKeyB64": key_clone.key.state_proof_key.as_ref().map(|k| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(k)).unwrap_or_default(),
                                "voteFirstValid": key_clone.key.vote_first_valid,
                                "voteLastValid": key_clone.key.vote_last_valid,
                                "keyDilution": key_clone.key.vote_key_dilution,
                                "network": lora_network,
                            });

                                let client = reqwest::Client::new();
                                if let Ok(resp) = client.post("http://b.nodekit.run/online")
                                    .json(&body)
                                    .send()
                                    .await 
                                {
                                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                                        if let Some(id) = json["id"].as_str() {
                                            let mut suffix = "";
                                            if let Some(account) = accounts.iter().find(|a| a.address == key_clone.address) {
                                                if account.incentive_eligible.unwrap_or(false) && account.status == "Online" {
                                                    suffix = "i";
                                                }
                                            }
                                            let link = format!("https://b.nodekit.run/{}{}", id, suffix);
                                            let _ = sender.send(Event::App(AppEvent::ShortlinkUpdate(link)));
                                        }
                                    }
                                }
                            });
                        } else {
                            // Fetch offline shortlink
                            let sender = self.events.get_sender();
                            let version = self.version.clone();
                            let key_clone = key.clone();
                            tokio::spawn(async move {
                                let network = version.map(|v| v.genesis_id).unwrap_or_else(|| "mainnet-v1.0".to_string());
                                let lora_network = network
                                    .replace("-v1.0", "")
                                    .replace("-v1", "");
                                let lora_network = if lora_network == "dockernet" || lora_network == "tuinet" {
                                    "localnet"
                                } else {
                                    &lora_network
                                };

                                let body = serde_json::json!({
                                    "account": key_clone.address,
                                    "network": lora_network,
                                });

                                let client = reqwest::Client::new();
                                if let Ok(resp) = client.post("http://b.nodekit.run/offline")
                                    .json(&body)
                                    .send()
                                    .await 
                                {
                                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                                        if let Some(id) = json["id"].as_str() {
                                            let link = format!("https://b.nodekit.run/{}", id);
                                            let _ = sender.send(Event::App(AppEvent::ShortlinkUpdate(link)));
                                        }
                                    }
                                }
                            });
                        }
                    }
                    AppEvent::AvgRoundTimeUpdate(avg) => {
                        self.avg_round_time = avg;
                        self.metrics.round_time = avg;
                    }
                    AppEvent::NodeStatusUpdate(status) => {
                        self.node_status = status;
                        if self.node_status == crate::event::NodeStatus::FastCatchup && self.active_modal != Some(ModalType::Catchup) {
                            self.active_modal = Some(ModalType::Catchup);
                        } else if self.node_status != crate::event::NodeStatus::FastCatchup && self.active_modal == Some(ModalType::Catchup) {
                            self.active_modal = None;
                        }
                    }
                    AppEvent::MetricsUpdate(new_metrics) => {
                        let now = std::time::Instant::now();
                        if let Some(last_ts) = self.metrics.last_ts {
                            let diff = now.duration_since(last_ts).as_secs_f64();
                            if diff > 0.0 {
                                self.metrics.rx = ((new_metrics.rx.saturating_sub(self.metrics.last_rx)) as f64 / diff) as u64;
                                self.metrics.tx = ((new_metrics.tx.saturating_sub(self.metrics.last_tx)) as f64 / diff) as u64;
                                self.metrics.rx_p2p = ((new_metrics.rx_p2p.saturating_sub(self.metrics.last_rx_p2p)) as f64 / diff) as u64;
                                self.metrics.tx_p2p = ((new_metrics.tx_p2p.saturating_sub(self.metrics.last_tx_p2p)) as f64 / diff) as u64;
                                self.metrics.tps = (new_metrics.tps - self.metrics.last_tps) / diff;
                            }
                        }
                        self.metrics.peers_ws = new_metrics.peers_ws;
                        self.metrics.peers_p2p = new_metrics.peers_p2p;
                        self.metrics.last_rx = new_metrics.rx;
                        self.metrics.last_tx = new_metrics.tx;
                        self.metrics.last_rx_p2p = new_metrics.rx_p2p;
                        self.metrics.last_tx_p2p = new_metrics.tx_p2p;
                        self.metrics.last_tps = new_metrics.tps;
                        self.metrics.last_ts = Some(now);
                    }
                    AppEvent::ShortlinkUpdate(link) => {
                        self.current_shortlink = Some(link);
                    }
                    AppEvent::GenerateKeys { address, last_round_delta } => {
                        let sender = self.events.get_sender();
                        let client = self.client.clone();
                        tokio::spawn(async move {
                            // We need to get current status to set first/last correctly if we want to match Go logic exactly
                            // Go: first = lastRound, last = lastRound + duration_rounds
                            let status = match client.get_status().await {
                                Ok(s) => s,
                                Err(e) => {
                                    let _ = sender.send(Event::App(AppEvent::GenerateError(format!("Failed to get status: {:?}", e))));
                                    return;
                                }
                            };

                            let first = status.last_round;
                            let last = first + last_round_delta;

                            match client.generate_participation_keys(&address, None, first, last).await {
                                Ok(_) => {
                                    // Start polling
                                    let timeout = std::time::Duration::from_secs(20 * 60);
                                    let start = std::time::Instant::now();
                                    loop {
                                        if start.elapsed() > timeout {
                                            let _ = sender.send(Event::App(AppEvent::GenerateError("Timeout waiting for key generation".to_string())));
                                            break;
                                        }
                                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                        match client.get_participation_keys().await {
                                            Ok(keys) => {
                                                if let Some(key) = keys.iter().find(|k| k.address == address && k.key.vote_first_valid == first && k.key.vote_last_valid == last) {
                                                    let _ = sender.send(Event::App(AppEvent::GenerateSuccess(key.clone())));
                                                    break;
                                                }
                                            }
                                            Err(_) => {
                                                // Ignore errors during polling, matching Go implementation
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    let err_str = format!("{:?}", e);
                                    if err_str.contains("Unexpected text response") {
                                        // Ignore non-JSON responses during key generation, proceed to poll
                                        let timeout = std::time::Duration::from_secs(20 * 60);
                                        let start = std::time::Instant::now();
                                        loop {
                                            if start.elapsed() > timeout {
                                                let _ = sender.send(Event::App(AppEvent::GenerateError("Timeout waiting for key generation".to_string())));
                                                break;
                                            }
                                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                            match client.get_participation_keys().await {
                                                Ok(keys) => {
                                                    if let Some(key) = keys.iter().find(|k| k.address == address && k.key.vote_first_valid == first && k.key.vote_last_valid == last) {
                                                        let _ = sender.send(Event::App(AppEvent::GenerateSuccess(key.clone())));
                                                        break;
                                                    }
                                                }
                                                Err(_) => {
                                                    // Ignore errors during polling
                                                }
                                            }
                                        }
                                    } else if err_str.to_lowercase().contains("generator already running") {
                                        let _ = sender.send(Event::App(AppEvent::GenerateError("Participation key generator already running. Please wait for the current process to finish.".to_string())));
                                    } else {
                                        let _ = sender.send(Event::App(AppEvent::GenerateError(format!("{}", e))));
                                    }
                                }
                            }
                        });
                    }
                    AppEvent::GenerateSuccess(key) => {
                        self.selected_key = Some(key);
                        self.selected_key_just_generated = true;
                        self.active_modal = Some(ModalType::KeyInfo);
                    }
                    AppEvent::GenerateError(err) => {
                        self.generate_error = Some(err);
                    }
                    AppEvent::StartFastCatchup => {
                        let sender = self.events.get_sender();
                        let version_opt = self.version.clone();
                        let base_url = self.url.clone();
                        let api_token = self.token.clone();

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
                                Ok(resp) => match resp.text().await {
                                    Ok(text) => text.trim().to_string(),
                                    Err(e) => {
                                        let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to read catchpoint: {:?}", e))));
                                        return;
                                    }
                                },
                                Err(e) => {
                                    let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to fetch catchpoint: {:?}", e))));
                                    return;
                                }
                            };

                            if catchpoint.is_empty() {
                                let _ = sender.send(Event::App(AppEvent::Error("Fetched catchpoint is empty".to_string())));
                                return;
                            }

                            // Since we can't easily add it to the library, we'll use reqwest to POST to the endpoint.
                            let encoded_catchpoint = url::form_urlencoded::byte_serialize(catchpoint.as_bytes()).collect::<String>();
                            let base_url = base_url.trim_end_matches('/');
                            let start_catchup_url = format!("{}/v2/catchup/{}", base_url, encoded_catchpoint);

                            let http_client = reqwest::Client::new();
                            match http_client.post(&start_catchup_url)
                                .header("X-Algo-API-Token", api_token)
                                .send().await 
                            {
                                Ok(resp) => {
                                    if resp.status().is_success() {
                                        // Success
                                    } else {
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
                    AppEvent::DeleteKey(id) => {
                        let sender = self.events.get_sender();
                        let client = self.client.clone();
                        let id_clone = id.clone();
                        tokio::spawn(async move {
                            match client.delete_participation_key_by_id(&id).await {
                                Ok(_) => {
                                    let _ = sender.send(Event::App(AppEvent::DeleteSuccess(id_clone)));
                                }
                                Err(e) => {
                                    let err_str = format!("{:?}", e);
                                    if err_str.contains("Unexpected text response: {}") {
                                        let _ = sender.send(Event::App(AppEvent::DeleteSuccess(id_clone)));
                                    } else {
                                        let _ = sender.send(Event::App(AppEvent::Error(format!("Failed to delete key: {:?}", e))));
                                    }
                                }
                            }
                        });
                    }
                    AppEvent::DeleteSuccess(_) => {
                        self.active_modal = None;
                        // Trigger a poll for keys immediately
                        let sender = self.events.get_sender();
                        let client = self.client.clone();
                        tokio::spawn(async move {
                            if let Ok(keys) = client.get_participation_keys().await {
                                let _ = sender.send(Event::App(AppEvent::KeysUpdate(keys)));
                            }
                        });
                    }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.active_modal == Some(ModalType::Partkey) {
            match key_event.code {
                KeyCode::Char('g') => {
                    self.generate_step = GenerateStep::Address;
                    self.generate_address = self.selected_account.clone().unwrap_or_default();
                    self.generate_duration = "1".to_string();
                    self.generate_range = GenerateRange::Month;
                    self.generate_error = None;
                    self.active_modal = Some(ModalType::Generate);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.active_modal = None;
                }
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                _ => {}
            }
            return Ok(());
        }

        if self.active_modal == Some(ModalType::Catchup) {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.events.send(AppEvent::Quit);
                }
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                _ => {}
            }
            return Ok(());
        }

        if self.active_modal == Some(ModalType::Lagging) {
            match key_event.code {
                KeyCode::Char('y') => {
                    self.events.send(AppEvent::StartFastCatchup);
                    self.active_modal = None;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.lagging_dismissed = true;
                    self.active_modal = None;
                }
                KeyCode::Char('q') => {
                    self.events.send(AppEvent::Quit);
                }
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                _ => {}
            }
            return Ok(());
        }

        match key_event.code {
            KeyCode::Char('y') if self.active_modal == Some(ModalType::DeleteConfirm) => {
                if let Some(key) = &self.selected_key {
                    self.active_modal = Some(ModalType::Deleting);
                    self.events.send(AppEvent::DeleteKey(key.id.clone()));
                }
            }
            KeyCode::Char('n') if self.active_modal == Some(ModalType::DeleteConfirm) => {
                self.active_modal = Some(ModalType::KeyInfo);
            }
            KeyCode::Char('d') if self.active_modal == Some(ModalType::KeyInfo) => {
                if self.selected_key.is_some() {
                    self.active_modal = Some(ModalType::DeleteConfirm);
                }
            }
            KeyCode::Char('r') if self.active_modal == Some(ModalType::KeyInfo) => {
                if !self.is_key_active_selected() {
                    self.key_info_mode = crate::event::KeyInfoMode::Online;
                }
            }
            KeyCode::Char('t') if self.active_modal == Some(ModalType::KeyInfo) => {
                if self.is_key_active_selected() {
                    self.key_info_mode = crate::event::KeyInfoMode::Online;
                }
            }
            KeyCode::Char('s') if self.active_modal == Some(ModalType::KeyInfo) => {
                self.key_info_mode = match self.key_info_mode {
                    crate::event::KeyInfoMode::Online => crate::event::KeyInfoMode::QR,
                    crate::event::KeyInfoMode::QR => crate::event::KeyInfoMode::Online,
                    _ => self.key_info_mode.clone(),
                };
            }
            KeyCode::Char('v') | KeyCode::Tab => {
                if self.active_modal == Some(ModalType::KeyInfo) {
                    self.key_info_mode = match self.key_info_mode {
                        crate::event::KeyInfoMode::Text => crate::event::KeyInfoMode::Online,
                        crate::event::KeyInfoMode::Online => crate::event::KeyInfoMode::QR,
                        crate::event::KeyInfoMode::QR => crate::event::KeyInfoMode::Text,
                    };
                }
            }
            KeyCode::Char('g') => {
                // Match Go TUI logic: only open modal when node is stable and round time is available
                if self.node_status == crate::event::NodeStatus::Stable && self.metrics.round_time > 0 {
                    // Ensure an account is selected if we are on the accounts page
                    if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                        let address = self.accounts[self.selected_account_index].address.clone();
                        self.selected_account = Some(address);
                    }

                    self.generate_step = GenerateStep::Address;
                    self.generate_address = self.selected_account.clone().unwrap_or_default();
                    self.generate_duration = "1".to_string();
                    self.generate_range = GenerateRange::Month;
                    self.generate_error = None;
                    self.active_modal = Some(ModalType::Generate);
                } else {
                    self.last_error = Some("Please wait until your node is fully synced".to_string());
                    self.active_modal = Some(ModalType::Exception);
                }
            }
            KeyCode::Esc => {
                if self.active_modal == Some(ModalType::KeyInfo) && self.key_info_mode != crate::event::KeyInfoMode::Text {
                    self.key_info_mode = crate::event::KeyInfoMode::Text;
                } else if self.active_modal.is_some() {
                    self.events.send(AppEvent::HideModal);
                } else if self.current_page == Page::Keys {
                    self.current_page = Page::Accounts;
                } else {
                    self.events.send(AppEvent::Quit);
                }
            }
            KeyCode::Char('q') => {
                if self.active_modal.is_some() && self.active_modal != Some(ModalType::Generate) {
                    self.events.send(AppEvent::HideModal);
                } else if self.active_modal.is_none() {
                    self.events.send(AppEvent::Quit);
                } else if self.active_modal == Some(ModalType::Generate) {
                    self.generate_address.push('q');
                }
            }
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char(c) if self.active_modal == Some(ModalType::Generate) => {
                match self.generate_step {
                    GenerateStep::Address => {
                        self.generate_address.push(c);
                    }
                    GenerateStep::Duration => {
                        if c.is_ascii_digit() {
                            self.generate_duration.push(c);
                        } else if c == 's' {
                            self.generate_range = match self.generate_range {
                                GenerateRange::Day => GenerateRange::Month,
                                GenerateRange::Month => GenerateRange::Round,
                                GenerateRange::Round => GenerateRange::Day,
                            };
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace if self.active_modal == Some(ModalType::Generate) => {
                match self.generate_step {
                    GenerateStep::Address => {
                        self.generate_address.pop();
                    }
                    GenerateStep::Duration => {
                        self.generate_duration.pop();
                    }
                    _ => {}
                }
            }
            KeyCode::Enter if self.active_modal == Some(ModalType::Generate) => {
                match self.generate_step {
                    GenerateStep::Address => {
                        if !self.generate_address.is_empty() {
                            self.generate_step = GenerateStep::Duration;
                        }
                    }
                    GenerateStep::Duration => {
                        if !self.generate_duration.is_empty() {
                            let val = self.generate_duration.parse::<u64>().unwrap_or(0);
                            if val > 0 {
                                let rounds = match self.generate_range {
                                    GenerateRange::Day => (val * 24 * 60 * 60 * 1000) / self.avg_round_time.max(1),
                                    GenerateRange::Month => (val * 30 * 24 * 60 * 60 * 1000) / self.avg_round_time.max(1),
                                    GenerateRange::Round => val,
                                };
                                self.generate_step = GenerateStep::Waiting;
                                self.events.send(AppEvent::GenerateKeys {
                                    address: self.generate_address.clone(),
                                    last_round_delta: rounds,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Right => {
                if self.active_modal == Some(ModalType::Generate) {
                    // Do nothing or maybe move cursor if implemented
                } else if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                    let address = self.accounts[self.selected_account_index].address.clone();
                    self.selected_account = Some(address);
                    self.selected_key_index = 0;
                    self.events.send(AppEvent::ShowKeys);
                } else {
                    self.events.send(AppEvent::Increment);
                }
            }
            KeyCode::Left => {
                if self.active_modal == Some(ModalType::Generate) {
                    // Do nothing
                } else if self.current_page == Page::Keys {
                    self.events.send(AppEvent::ShowAccounts);
                } else {
                    self.events.send(AppEvent::Decrement);
                }
            }
            KeyCode::Char('a') => {
                if self.active_modal == Some(ModalType::Generate) {
                    self.generate_address.push('a');
                } else {
                    self.events.send(AppEvent::ShowAccounts);
                }
            }
            KeyCode::Char('k') => {
                if self.active_modal == Some(ModalType::Generate) {
                    self.generate_address.push('k');
                } else {
                    self.events.send(AppEvent::ShowKeys);
                }
            }
            KeyCode::Up => {
                if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                    self.selected_account_index = self.selected_account_index.saturating_sub(1);
                } else if self.current_page == Page::Keys && !self.keys.is_empty() {
                    self.selected_key_index = self.selected_key_index.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                    self.selected_account_index = (self.selected_account_index + 1).min(self.accounts.len() - 1);
                } else if self.current_page == Page::Keys && !self.keys.is_empty() {
                    let filtered_keys_count = if let Some(addr) = &self.selected_account {
                        self.keys.iter().filter(|k| &k.address == addr).count()
                    } else {
                        self.keys.len()
                    };
                    if filtered_keys_count > 0 {
                        self.selected_key_index = (self.selected_key_index + 1).min(filtered_keys_count - 1);
                    }
                }
            }
            KeyCode::Enter => {
                if self.active_modal.is_some() {
                    // Enter already handled for Generate modal above
                } else if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                    let address = self.accounts[self.selected_account_index].address.clone();
                    self.selected_account = Some(address);
                    self.selected_key_index = 0;
                    self.events.send(AppEvent::ShowKeys);
                } else if self.current_page == Page::Keys && !self.keys.is_empty() {
                    let filtered_keys: Vec<_> = if let Some(addr) = &self.selected_account {
                        self.keys.iter().filter(|k| &k.address == addr).cloned().collect()
                    } else {
                        self.keys.clone()
                    };
                    if !filtered_keys.is_empty() {
                        if self.selected_key_index >= filtered_keys.len() {
                            self.selected_key_index = 0;
                        }
                        let key = filtered_keys[self.selected_key_index].clone();
                        self.events.send(AppEvent::SelectKey(key));
                        self.events.send(AppEvent::ShowModal(ModalType::KeyInfo));
                    }
                }
            }
            KeyCode::Char('h') => {
                if self.active_modal == Some(ModalType::Generate) {
                    self.generate_address.push('h');
                } else {
                    self.events.send(AppEvent::ShowModal(ModalType::Hybrid));
                }
            }
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    pub fn is_key_active(&self, key: &algod_client::models::ParticipationKey) -> bool {
        if let Some(addr) = &self.selected_account {
            if let Some(account) = self.accounts.iter().find(|a| &a.address == addr) {
                if let Some(part) = &account.participation {
                    return key.key.vote_first_valid == part.vote_first_valid &&
                           key.key.vote_last_valid == part.vote_last_valid &&
                           key.key.vote_participation_key == part.vote_participation_key;
                }
            }
        }
        false
    }

    fn is_key_active_selected(&self) -> bool {
        if let Some(key) = &self.selected_key {
            self.is_key_active(key)
        } else {
            false
        }
    }
}
