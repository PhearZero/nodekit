use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use std::time::Duration;
use tokio::sync::mpsc;

/// The frequency at which tick events are emitted.
const TICK_FPS: f64 = 30.0;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
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
    AlgodUpdate(Box<algod_client::models::WaitForBlock>),
    /// Version info from Algod
    VersionUpdate(Box<algod_client::models::Version>),
    /// Update availability
    UpdateAvailable(bool),
    /// Update accounts from Algod
    AccountsUpdate(Vec<algod_client::models::Account>),
    /// Update participation keys from Algod
    KeysUpdate(Vec<algod_client::models::ParticipationKey>),
    /// Select a specific participation key
    SelectKey(algod_client::models::ParticipationKey),
    /// Update average round time
    AvgRoundTimeUpdate(u64),
    /// Update node status
    NodeStatusUpdate(NodeStatus),
    /// Update metrics
    MetricsUpdate(Metrics),
    /// Generate participation keys
    GenerateKeys { address: String, last_round_delta: u64 },
    /// Key generation success
    GenerateSuccess(algod_client::models::ParticipationKey),
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
        tokio::spawn(async { actor.run().await });
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
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);
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
                self.send(Event::Crossterm(evt));
              }
            };
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
