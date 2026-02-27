use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use ratatui::crossterm::event::{Event as CrosstermEvent, KeyEvent as CrosstermKeyEvent, KeyCode as CrosstermKeyCode, KeyModifiers as CrosstermKeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

pub type AlgodStatus = algod_client::models::WaitForBlock;
pub type AlgodVersion = algod_client::models::Version;
pub type AlgodAccount = algod_client::models::Account;
pub type AlgodParticipationKey = algod_client::models::ParticipationKey;
pub type AlgodAccountParticipation = algod_client::models::AccountParticipation;

/// Spawns a task in a target-agnostic way.
/// On native, it uses `tokio::spawn`.
/// On WASM, it uses `wasm_bindgen_futures::spawn_local`.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(target_arch = "wasm32")]
use ratzilla::event::{KeyEvent as RatzillaKeyEvent, KeyCode as RatzillaKeyCode};

/// The frequency at which tick events are emitted.
const TICK_FPS: f64 = 30.0;

/// Unified key code representation for different backends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKeyCode {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Esc,
    Unidentified,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CrosstermKeyCode> for BackendKeyCode {
    fn from(code: CrosstermKeyCode) -> Self {
        match code {
            CrosstermKeyCode::Char(c) => BackendKeyCode::Char(c),
            CrosstermKeyCode::F(n) => BackendKeyCode::F(n),
            CrosstermKeyCode::Backspace => BackendKeyCode::Backspace,
            CrosstermKeyCode::Enter => BackendKeyCode::Enter,
            CrosstermKeyCode::Left => BackendKeyCode::Left,
            CrosstermKeyCode::Right => BackendKeyCode::Right,
            CrosstermKeyCode::Up => BackendKeyCode::Up,
            CrosstermKeyCode::Down => BackendKeyCode::Down,
            CrosstermKeyCode::Tab => BackendKeyCode::Tab,
            CrosstermKeyCode::Delete => BackendKeyCode::Delete,
            CrosstermKeyCode::Home => BackendKeyCode::Home,
            CrosstermKeyCode::End => BackendKeyCode::End,
            CrosstermKeyCode::PageUp => BackendKeyCode::PageUp,
            CrosstermKeyCode::PageDown => BackendKeyCode::PageDown,
            CrosstermKeyCode::Esc => BackendKeyCode::Esc,
            _ => BackendKeyCode::Unidentified,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<RatzillaKeyCode> for BackendKeyCode {
    fn from(code: RatzillaKeyCode) -> Self {
        match code {
            RatzillaKeyCode::Char(c) => BackendKeyCode::Char(c),
            RatzillaKeyCode::F(n) => BackendKeyCode::F(n),
            RatzillaKeyCode::Backspace => BackendKeyCode::Backspace,
            RatzillaKeyCode::Enter => BackendKeyCode::Enter,
            RatzillaKeyCode::Left => BackendKeyCode::Left,
            RatzillaKeyCode::Right => BackendKeyCode::Right,
            RatzillaKeyCode::Up => BackendKeyCode::Up,
            RatzillaKeyCode::Down => BackendKeyCode::Down,
            RatzillaKeyCode::Tab => BackendKeyCode::Tab,
            RatzillaKeyCode::Delete => BackendKeyCode::Delete,
            RatzillaKeyCode::Home => BackendKeyCode::Home,
            RatzillaKeyCode::End => BackendKeyCode::End,
            RatzillaKeyCode::PageUp => BackendKeyCode::PageUp,
            RatzillaKeyCode::PageDown => BackendKeyCode::PageDown,
            RatzillaKeyCode::Esc => BackendKeyCode::Esc,
            RatzillaKeyCode::Unidentified => BackendKeyCode::Unidentified,
        }
    }
}

/// Unified key event representation for different backends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendKeyEvent {
    pub code: BackendKeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CrosstermKeyEvent> for BackendKeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        Self {
            code: BackendKeyEvent::map_code(event.code),
            ctrl: event.modifiers.contains(CrosstermKeyModifiers::CONTROL),
            alt: event.modifiers.contains(CrosstermKeyModifiers::ALT),
            shift: event.modifiers.contains(CrosstermKeyModifiers::SHIFT),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BackendKeyEvent {
    fn map_code(code: CrosstermKeyCode) -> BackendKeyCode {
        code.into()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<RatzillaKeyEvent> for BackendKeyEvent {
    fn from(event: RatzillaKeyEvent) -> Self {
        Self {
            code: event.code.into(),
            ctrl: event.ctrl,
            alt: event.alt,
            shift: event.shift,
        }
    }
}

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Backend key events (unified).
    Key(BackendKeyEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

/// Application events.
///
/// You can extend this enum with your own custom events.
#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    /// Increment the counter.
    Increment,
    /// Decrement the counter.
    Decrement,
    /// Quit the application.
    Quit,
    /// Show the Accounts page.
    ShowAccounts,
    /// Show the Keys page.
    ShowKeys,
    /// Show a modal.
    ShowModal(ModalType),
    /// Error occurred.
    Error(String),
    /// Hide the current modal.
    HideModal,
    /// Update from Algod
    AlgodUpdate(Box<AlgodStatus>),
    /// Version info from Algod
    VersionUpdate(Box<AlgodVersion>),
    /// Update availability
    UpdateAvailable(bool),
    /// Update accounts from Algod
    AccountsUpdate(Vec<AlgodAccount>),
    /// Update participation keys from Algod
    KeysUpdate(Vec<AlgodParticipationKey>),
    /// Select a specific participation key
    SelectKey(AlgodParticipationKey),
    /// Update average round time
    AvgRoundTimeUpdate(u64),
    /// Update node status
    NodeStatusUpdate(NodeStatus),
    /// Update metrics
    MetricsUpdate(Metrics),
    /// Generate participation keys
    GenerateKeys { address: String, last_round_delta: u64 },
    /// Key generation success
    GenerateSuccess(AlgodParticipationKey),
    /// Key generation error
    /// Key generation error
    GenerateError(String),
    StartFastCatchup,
    DeleteKey(String),
    DeleteSuccess(String),
    ShortlinkUpdate(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    Catchup,
    Exception,
    Hybrid,
    Partkey,
    KeyInfo,
    Generate,
    Lagging,
    DeleteConfirm,
    Deleting,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum KeyInfoMode {
    #[default]
    Text,
    Online,
    QR,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Metrics {
    pub peers_ws: u64,
    pub peers_p2p: u64,
    pub tps: f64,
    pub round_time: u64, // ms
    pub rx: u64,
    pub tx: u64,
    pub rx_p2p: u64,
    pub tx_p2p: u64,
    pub last_rx: u64,
    pub last_tx: u64,
    pub last_rx_p2p: u64,
    pub last_tx_p2p: u64,
    pub last_tps: f64,
    pub last_ts: Option<std::time::Instant>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum NodeStatus {
    #[default]
    Stable,
    Syncing,
    FastCatchup,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        spawn(async move { let _ = actor.run().await; });
        Self { sender, receiver }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    /// Try to receive an event from the sender without blocking.
    pub fn try_next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .try_recv()
            .map_err(|_| color_eyre::eyre::eyre!("No events available"))
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }

    /// Returns a clone of the sender.
    pub fn get_sender(&self) -> mpsc::UnboundedSender<Event> {
        self.sender.clone()
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> color_eyre::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut tick = tokio::time::interval(tick_rate);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut reader = crossterm::event::EventStream::new();
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = self.sender.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    self.send(Event::Tick);
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    if let CrosstermEvent::Key(key) = evt {
                        if key.kind == ratatui::crossterm::event::KeyEventKind::Press {
                             self.send(Event::Key(key.into()));
                        }
                    }
                  }
                };
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            loop {
                let _ = tick.tick().await;
                if self.sender.is_closed() {
                    break;
                }
                self.send(Event::Tick);
            }
        }

        Ok(())
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}
