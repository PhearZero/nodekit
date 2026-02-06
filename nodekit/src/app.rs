use crate::event::{AppEvent, Event, EventHandler, ModalType};
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
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Current page.
    pub current_page: Page,
    /// Active modal.
    pub active_modal: Option<ModalType>,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_page: Page::default(),
            active_modal: None,
            counter: 0,
            events: EventHandler::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
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
                    AppEvent::ShowModal(modal) => self.active_modal = Some(modal),
                    AppEvent::HideModal => self.active_modal = None,
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.active_modal.is_some() {
                    self.events.send(AppEvent::HideModal);
                } else {
                    self.events.send(AppEvent::Quit);
                }
            }
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => {
                if self.current_page == Page::Accounts {
                    self.events.send(AppEvent::ShowKeys);
                } else {
                    self.events.send(AppEvent::Increment);
                }
            }
            KeyCode::Left => {
                if self.current_page == Page::Keys {
                    self.events.send(AppEvent::ShowAccounts);
                } else {
                    self.events.send(AppEvent::Decrement);
                }
            }
            KeyCode::Char('a') => self.events.send(AppEvent::ShowAccounts),
            KeyCode::Char('k') => self.events.send(AppEvent::ShowKeys),
            KeyCode::Char('h') => self.events.send(AppEvent::ShowModal(ModalType::Hybrid)),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

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
}
