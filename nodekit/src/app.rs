use crate::event::{AppEvent, Event, EventHandler, ModalType, Metrics, BackendKeyEvent, BackendKeyCode, AlgodStatus, AlgodVersion, AlgodAccount, AlgodParticipationKey, spawn, Instant, sleep};
use crate::ui::viewport::ViewportComponent;
use ratatui::prelude::Backend;
use ratatui::{
    Terminal,
};
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

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
    pub status: Option<AlgodStatus>,
    /// Version info
    pub version: Option<AlgodVersion>,
    /// Update available
    pub update_available: bool,
    /// Accounts
    pub accounts: Vec<AlgodAccount>,
    /// Participation keys
    pub keys: Vec<AlgodParticipationKey>,
    /// Selected account address
    pub selected_account: Option<String>,
    /// Selected account index
    pub selected_account_index: usize,
    /// Selected key index
    pub selected_key_index: usize,
    /// Selected key
    pub selected_key: Option<AlgodParticipationKey>,
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
    /// Is loading?
    pub is_loading: bool,
    /// Loading start time
    pub loading_start_time: Option<Instant>,
    /// Loading message
    pub loading_message: Option<String>,
    /// WiFi connected
    pub wifi_connected: bool,
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

        #[allow(unused_mut)]
        let mut p2p_enabled = false;
        #[allow(unused_mut)]
        let mut p2p_hybrid_enabled = false;

        if let Some(_dir) = data_dir {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let config_path = std::path::Path::new(_dir).join("config.json");
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        p2p_enabled = config["EnableP2P"].as_bool().unwrap_or(false);
                        p2p_hybrid_enabled = config["EnableP2PHybridMode"].as_bool().unwrap_or(false);
                    }
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
            is_loading: true,
            loading_start_time: Some(Instant::now()),
            loading_message: None,
            wifi_connected: false,
        }
    }

    /// Run the application's main loop.
    pub async fn run<B: Backend>(self, terminal: Terminal<B>) -> color_eyre::Result<()> where <B as Backend>::Error: std::error::Error + Send + Sync + 'static {
    #[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
    {
        use futures::FutureExt;
        log::info!("Starting App::run embedded loop...");
        // This is a simplified loop for embedded
        let mut terminal = terminal;
        let mut app = self;
        let mut last_loading = app.is_loading;
        
        loop {
            #[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
            unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

            if last_loading && !app.is_loading {
                let _ = terminal.clear();
            }
            last_loading = app.is_loading;

            terminal.draw(|frame: &mut ratatui::Frame| {
                let viewport = ViewportComponent::new(&app);
                frame.render_widget(&viewport, frame.area());
            })?;
            
            app.tick();
            
            while let Some(event) = app.events.next().now_or_never() {
                if let Ok(event) = event {
                    match event {
                        Event::Tick => app.tick(),
                        Event::Key(key_event) => {
                            let _ = app.handle_key_events(key_event);
                        }
                        Event::App(app_event) => {
                            let _ = app.handle_app_event(app_event).await;
                        }
                        Event::LeftButton => {
                            let _ = app.handle_key_events(BackendKeyEvent {
                                code: BackendKeyCode::Left,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            });
                        }
                        Event::RightButton => {
                            let _ = app.handle_key_events(BackendKeyEvent {
                                code: BackendKeyCode::Right,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            });
                        }
                        Event::SelectButton => {
                            let _ = app.handle_key_events(BackendKeyEvent {
                                code: BackendKeyCode::Enter,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            });
                        }
                        Event::Touch(x, y) => {
                            let _ = app.handle_touch_event(x, y);
                        }
                    }
                }
            }

            if !app.running {
                break;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        return Ok(());
    }

        #[cfg(target_arch = "wasm32")]
        {
            use ratzilla::WebRenderer;
            let sender = self.events.get_sender();
            terminal.on_key_event(move |evt: ratzilla::event::KeyEvent| {
                let _ = sender.send(Event::Key(evt.into()));
            });

            let app = Rc::new(RefCell::new(self));
            let app_clone = app.clone();

            terminal.draw_web(move |frame: &mut ratatui::Frame| {
                let mut app = app_clone.borrow_mut();
                let viewport = ViewportComponent::new(&app);
                frame.render_widget(&viewport, frame.area());

                let mut events_processed = 0;
                while let Ok(event) = app.events.try_next() {
                    events_processed += 1;
                    match event {
                        Event::Tick => app.tick(),
                        Event::Key(key_event) => {
                            let _ = app.handle_key_events(key_event);
                        }
                        Event::App(app_event) => {
                            let app_inner = app_clone.clone();
                            spawn(async move {
                                app_inner.borrow_mut().handle_app_event(app_event).await;
                            });
                        }
                        Event::LeftButton | Event::RightButton | Event::SelectButton | Event::Touch(_, _) => {}
                    }
                    if events_processed > 20 {
                        break;
                    }
                }
            });
            Ok(())
        }

        #[cfg(not(any(target_arch = "wasm32", target_arch = "xtensa", target_arch = "riscv32")))]
        {
            let mut terminal = terminal;
            let mut app = self;
            while app.running {
                terminal.draw(|frame| {
                    let viewport = ViewportComponent::new(&app);
                    frame.render_widget(&viewport, frame.area());
                })?;
                match app.events.next().await? {
                    Event::Tick => app.tick(),
                    Event::Key(key_event) => {
                        app.handle_key_events(key_event)?
                    }
                    Event::App(app_event) => app.handle_app_event(app_event).await,
                    Event::LeftButton => {
                        app.handle_key_events(BackendKeyEvent {
                            code: BackendKeyCode::Left,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        })?;
                    }
                    Event::RightButton => {
                        app.handle_key_events(BackendKeyEvent {
                            code: BackendKeyCode::Right,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        })?;
                    }
                    Event::SelectButton => {
                        app.handle_key_events(BackendKeyEvent {
                            code: BackendKeyCode::Enter,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        })?;
                    }
                    Event::Touch(x, y) => {
                        app.handle_touch_event(x, y);
                    }
                }
            }
            Ok(())
        }
    }

    #[cfg(feature = "simulator")]
    pub async fn run_simulator(
        mut self,
        mut terminal: ratatui::Terminal<mousefood::EmbeddedBackend<'_, embedded_graphics_simulator::SimulatorDisplay<embedded_graphics::pixelcolor::Rgb888>, embedded_graphics::pixelcolor::Rgb888>>,
        mut window: embedded_graphics_simulator::Window,
    ) -> color_eyre::Result<()> {
        use embedded_graphics_simulator::SimulatorEvent;
        'running: loop {
            terminal.draw(|frame: &mut ratatui::Frame| {
                let viewport = ViewportComponent::new(&self);
                frame.render_widget(&viewport, frame.area());
            })?;

            window.update(terminal.backend().display());

            for event in window.events() {
                match event {
                    SimulatorEvent::Quit => break 'running,
                    SimulatorEvent::KeyDown { keycode, .. } => {
                        use embedded_graphics_simulator::sdl2::Keycode;
                        match keycode {
                            Keycode::Left => {
                                let _ = self.events.get_sender().send(Event::LeftButton);
                            }
                            Keycode::Right => {
                                let _ = self.events.get_sender().send(Event::RightButton);
                            }
                            Keycode::Return | Keycode::Space => {
                                let _ = self.events.get_sender().send(Event::SelectButton);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            while let Ok(event) = self.events.try_next() {
                match event {
                    Event::Tick => self.tick(),
                    Event::Key(key_event) => {
                        self.handle_key_events(key_event)?;
                    }
                    Event::App(app_event) => self.handle_app_event(app_event).await,
                    Event::LeftButton => {
                        self.handle_key_events(BackendKeyEvent {
                            code: BackendKeyCode::Left,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        })?;
                    }
                    Event::RightButton => {
                        self.handle_key_events(BackendKeyEvent {
                            code: BackendKeyCode::Right,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        })?;
                    }
                    Event::SelectButton => {
                        self.handle_key_events(BackendKeyEvent {
                            code: BackendKeyCode::Enter,
                            ctrl: false,
                            alt: false,
                            shift: false,
                        })?;
                    }
                    Event::Touch(x, y) => {
                        self.handle_touch_event(x, y);
                    }
                }
            }

            if !self.running {
                break 'running;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }

    pub fn handle_touch_event(&mut self, x: u16, y: u16) {
        // Basic mapping of touch to key events for the Advance 5 display (800x480)
        // If we are in Keys page, and touch is in the top left, go back
        if self.current_page == Page::Keys && y < 100 && x < 200 {
            let _ = self.handle_key_events(BackendKeyEvent {
                code: BackendKeyCode::Esc,
                ctrl: false,
                alt: false,
                shift: false,
            });
        } else if y < 100 {
            // Top bar touch: navigate between pages
            if x < 400 {
                self.events.send(AppEvent::ShowAccounts);
            } else {
                self.events.send(AppEvent::ShowKeys);
            }
        } else if y > 380 {
            // Bottom bar touch: maybe quit or other action?
        } else {
            // Main area: scroll or select
            if x < 200 {
                let _ = self.handle_key_events(BackendKeyEvent {
                    code: BackendKeyCode::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                });
            } else if x > 600 {
                let _ = self.handle_key_events(BackendKeyEvent {
                    code: BackendKeyCode::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                });
            } else {
                let _ = self.handle_key_events(BackendKeyEvent {
                    code: BackendKeyCode::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                });
            }
        }
    }

    pub async fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            AppEvent::Increment => self.increment_counter(),
            AppEvent::Decrement => self.decrement_counter(),
            AppEvent::Quit => self.quit(),
            AppEvent::ShowAccounts => self.current_page = Page::Accounts,
            AppEvent::ShowKeys => self.current_page = Page::Keys,
            AppEvent::ShowModal(modal) => {
                if modal == ModalType::Lagging && (self.lagging_dismissed || self.active_modal.is_some()) {
                    return;
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
                    let account_incentive_eligible = self.accounts.iter()
                        .find(|a| a.address == key.address)
                        .and_then(|a| a.incentive_eligible)
                        .unwrap_or(false);
                    let account_status = self.accounts.iter()
                        .find(|a| a.address == key.address)
                        .map(|a| a.status.clone())
                        .unwrap_or_else(|| "Offline".to_string());

                    crate::service::shortlink::fetch_online_shortlink(
                        self.events.get_sender(),
                        self.version.clone(),
                        key,
                        account_incentive_eligible,
                        account_status,
                    );
                } else {
                    crate::service::shortlink::fetch_offline_shortlink(
                        self.events.get_sender(),
                        self.version.clone(),
                        key.address,
                    );
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
                let now = Instant::now();
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
                spawn(async move {
                    // We need to get current status to set first/last correctly if we want to match Go logic exactly
                    // Go: first = lastRound, last = lastRound + duration_rounds
                    let status_res = client.get_status().await;
                    let status = match status_res {
                        Ok(s) => s,
                        Err(e) => {
                            let _ = sender.send(Event::App(AppEvent::GenerateError(format!("Failed to get status: {:?}", e))));
                            return;
                        }
                    };

                    let first = status.last_round;
                    let last = first + last_round_delta;

                    let gen_res = client.generate_participation_keys(&address, None, first, last).await;
                    match gen_res {
                        Ok(_) => {
                            Self::poll_for_generated_key(sender, client, address, first, last).await;
                        }
                        Err(e) => {
                            let err_str = format!("{:?}", e);
                            if err_str.contains("Unexpected text response") {
                                // Ignore non-JSON responses during key generation, proceed to poll
                                Self::poll_for_generated_key(sender, client, address, first, last).await;
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
                crate::service::node::start_fast_catchup(
                    self.events.get_sender(),
                    self.url.clone(),
                    self.token.clone(),
                    self.version.clone(),
                );
            }
            AppEvent::DeleteKey(id) => {
                let sender = self.events.get_sender();
                let client = self.client.clone();
                let id_clone = id.clone();
                spawn(async move {
                    let res = client.delete_participation_key_by_id(&id).await;
                    match res {
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
                spawn(async move {
                    let keys_res = client.get_participation_keys().await;
                    if let Ok(keys) = keys_res {
                        let _ = sender.send(Event::App(AppEvent::KeysUpdate(keys)));
                    }
                });
            }
            AppEvent::LoadingMessage(msg) => {
                self.loading_message = Some(msg);
            }
            AppEvent::WifiConnected => {
                self.wifi_connected = true;
            }
        }
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: BackendKeyEvent) -> color_eyre::Result<()> {
        if self.handle_modal_key_events(key_event)? {
            return Ok(());
        }

        self.handle_global_key_events(key_event)
    }

    fn handle_modal_key_events(&mut self, key_event: BackendKeyEvent) -> color_eyre::Result<bool> {
        if let Some(modal) = &self.active_modal {
            match modal {
                ModalType::Partkey => {
                    match key_event.code {
                        BackendKeyCode::Char('g') => {
                            self.generate_step = GenerateStep::Address;
                            self.generate_address = self.selected_account.clone().unwrap_or_default();
                            self.generate_duration = "1".to_string();
                            self.generate_range = GenerateRange::Month;
                            self.generate_error = None;
                            self.active_modal = Some(ModalType::Generate);
                        }
                        BackendKeyCode::Esc | BackendKeyCode::Char('q') => {
                            self.active_modal = None;
                        }
                        BackendKeyCode::Char('c' | 'C') if key_event.ctrl => {
                            self.events.send(AppEvent::Quit)
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                ModalType::Catchup => {
                    match key_event.code {
                        BackendKeyCode::Esc | BackendKeyCode::Char('q') => {
                            self.events.send(AppEvent::Quit);
                        }
                        BackendKeyCode::Char('c' | 'C') if key_event.ctrl => {
                            self.events.send(AppEvent::Quit)
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                ModalType::Lagging => {
                    match key_event.code {
                        BackendKeyCode::Char('y') => {
                            self.events.send(AppEvent::StartFastCatchup);
                            self.active_modal = None;
                        }
                        BackendKeyCode::Char('n') | BackendKeyCode::Esc => {
                            self.lagging_dismissed = true;
                            self.active_modal = None;
                        }
                        BackendKeyCode::Char('q') => {
                            self.events.send(AppEvent::Quit);
                        }
                        BackendKeyCode::Char('c' | 'C') if key_event.ctrl => {
                            self.events.send(AppEvent::Quit)
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                ModalType::DeleteConfirm => {
                    match key_event.code {
                        BackendKeyCode::Char('y') => {
                            if let Some(key) = &self.selected_key {
                                self.active_modal = Some(ModalType::Deleting);
                                self.events.send(AppEvent::DeleteKey(key.id.clone()));
                            }
                        }
                        BackendKeyCode::Char('n') | BackendKeyCode::Esc => {
                            self.active_modal = Some(ModalType::KeyInfo);
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                ModalType::KeyInfo => {
                    match key_event.code {
                        BackendKeyCode::Char('d') => {
                            if self.selected_key.is_some() {
                                self.active_modal = Some(ModalType::DeleteConfirm);
                            }
                        }
                        BackendKeyCode::Char('r') => {
                            if !self.is_key_active_selected() {
                                self.key_info_mode = crate::event::KeyInfoMode::Online;
                            }
                        }
                        BackendKeyCode::Char('t') => {
                            if self.is_key_active_selected() {
                                self.key_info_mode = crate::event::KeyInfoMode::Online;
                            }
                        }
                        BackendKeyCode::Char('s') => {
                            self.key_info_mode = match self.key_info_mode {
                                crate::event::KeyInfoMode::Online => crate::event::KeyInfoMode::QR,
                                crate::event::KeyInfoMode::QR => crate::event::KeyInfoMode::Online,
                                _ => self.key_info_mode.clone(),
                            };
                        }
                        BackendKeyCode::Char('v') | BackendKeyCode::Tab => {
                            self.key_info_mode = match self.key_info_mode {
                                crate::event::KeyInfoMode::Text => crate::event::KeyInfoMode::Online,
                                crate::event::KeyInfoMode::Online => crate::event::KeyInfoMode::QR,
                                crate::event::KeyInfoMode::QR => crate::event::KeyInfoMode::Text,
                            };
                        }
                        BackendKeyCode::Esc => {
                            if self.key_info_mode != crate::event::KeyInfoMode::Text {
                                self.key_info_mode = crate::event::KeyInfoMode::Text;
                            } else {
                                self.active_modal = None;
                            }
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                ModalType::Generate => {
                    match key_event.code {
                        BackendKeyCode::Char(c) => {
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
                        BackendKeyCode::Backspace => {
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
                        BackendKeyCode::Enter => {
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
                        BackendKeyCode::Esc => {
                            self.active_modal = None;
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                _ => {
                    if key_event.code == BackendKeyCode::Esc || key_event.code == BackendKeyCode::Char('q') {
                        self.active_modal = None;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    fn handle_global_key_events(&mut self, key_event: BackendKeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            BackendKeyCode::Char('g') => {
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
            BackendKeyCode::Esc => {
                if self.current_page == Page::Keys {
                    self.current_page = Page::Accounts;
                } else {
                    self.events.send(AppEvent::Quit);
                }
            }
            BackendKeyCode::Char('q') => {
                self.events.send(AppEvent::Quit);
            }
            BackendKeyCode::Char('c' | 'C') if key_event.ctrl => {
                self.events.send(AppEvent::Quit)
            }
            BackendKeyCode::Right => {
                if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                    let address = self.accounts[self.selected_account_index].address.clone();
                    self.selected_account = Some(address);
                    self.selected_key_index = 0;
                    self.events.send(AppEvent::ShowKeys);
                } else {
                    self.events.send(AppEvent::Increment);
                }
            }
            BackendKeyCode::Left => {
                if self.current_page == Page::Keys {
                    self.events.send(AppEvent::ShowAccounts);
                } else {
                    self.events.send(AppEvent::Decrement);
                }
            }
            BackendKeyCode::Char('a') => {
                self.events.send(AppEvent::ShowAccounts);
            }
            BackendKeyCode::Char('k') => {
                self.events.send(AppEvent::ShowKeys);
            }
            BackendKeyCode::Up => {
                if self.current_page == Page::Accounts && !self.accounts.is_empty() {
                    self.selected_account_index = self.selected_account_index.saturating_sub(1);
                } else if self.current_page == Page::Keys && !self.keys.is_empty() {
                    self.selected_key_index = self.selected_key_index.saturating_sub(1);
                }
            }
            BackendKeyCode::Down => {
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
            BackendKeyCode::Enter => {
                if self.current_page == Page::Accounts && !self.accounts.is_empty() {
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
            BackendKeyCode::Char('h') => {
                self.events.send(AppEvent::ShowModal(ModalType::Hybrid));
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        self.increment_counter();
        if self.is_loading {
            if let Some(start) = self.loading_start_time {
                // Open if 1s has passed AND (we have status OR WiFi is connected)
                if start.elapsed().as_secs() >= 1 && (self.status.is_some() || self.wifi_connected) {
                    self.is_loading = false;
                }
            }
        }
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

    pub fn is_key_active(&self, key: &AlgodParticipationKey) -> bool {
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

    async fn poll_for_generated_key(
        sender: tokio::sync::mpsc::UnboundedSender<Event>,
        client: algod_client::AlgodClient,
        address: String,
        first: u64,
        last: u64,
    ) {
        let timeout = std::time::Duration::from_secs(20 * 60);
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                let _ = sender.send(Event::App(AppEvent::GenerateError("Timeout waiting for key generation".to_string())));
                break;
            }
            sleep(std::time::Duration::from_secs(2)).await;
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
    }
}
