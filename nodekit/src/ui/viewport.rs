use crate::app::{App, Page};
use crate::event::ModalType;
use crate::ui::pages::accounts::AccountsPage;
use crate::ui::pages::keys::KeysPage;
use crate::ui::status::StatusComponent;
use crate::ui::protocol::ProtocolComponent;
use crate::ui::modals::catchup::CatchupModal;
use crate::ui::modals::exception::ExceptionModal;
use crate::ui::modals::hybrid::HybridModal;
use crate::ui::modals::partkey::PartkeyModal;
use crate::ui::modals::generate::GenerateModal;
use crate::ui::modals::lagging::LaggingModal;
use crate::ui::modals::delete_confirm::DeleteConfirmModal;
use crate::ui::modals::deleting::DeletingModal;
use crate::ui::modals::key_info::KeyInfoModal;
use crate::ui::modals::ModalMetadata;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Clear, Widget},
};

pub struct ViewportComponent<'a> {
    app: &'a App,
}

impl<'a> ViewportComponent<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for &ViewportComponent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_compact = area.width < 90;

        let (status_area, protocol_area, page_area) = if is_compact {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7), // Status
                    Constraint::Length(7), // Protocol
                    Constraint::Min(0),    // Page Content
                ])
                .split(area);
            (chunks[0], chunks[1], chunks[2])
        } else {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7),
                    Constraint::Min(0),
                ])
                .split(area);

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(vertical_chunks[0]);

            (top_chunks[0], top_chunks[1], vertical_chunks[1])
        };

        // Render Status
        let status = StatusComponent::new(
            &self.app.status,
            &self.app.node_status,
            &self.app.metrics,
            self.app.p2p_enabled,
            self.app.p2p_hybrid_enabled,
        );
        status.render(status_area, buf);

        // Render Protocol
        let protocol = ProtocolComponent::new(
            &self.app.status,
            &self.app.version,
            self.app.update_available,
            &self.app.metrics,
        );
        protocol.render(protocol_area, buf);

        // Render Page
        match self.app.current_page {
            Page::Accounts => {
                let last_round = self.app.status.as_ref().map(|s| s.last_round).unwrap_or(0);
                let page = AccountsPage::new(
                    &self.app.accounts,
                    &self.app.keys,
                    last_round,
                    self.app.avg_round_time,
                    self.app.selected_account_index,
                    &self.app.node_status,
                );
                page.render(page_area, buf);
            }
                Page::Keys => {
                    let filtered_keys: Vec<_> = if let Some(addr) = &self.app.selected_account {
                        self.app.keys.iter().filter(|k| &k.address == addr).cloned().collect()
                    } else {
                        self.app.keys.clone()
                    };

                    let participation = if let Some(addr) = &self.app.selected_account {
                        self.app.accounts.iter()
                            .find(|a| &a.address == addr)
                            .and_then(|a| a.participation.clone())
                    } else {
                        None
                    };

                    let page = KeysPage::new(&filtered_keys, self.app.selected_key_index, &participation);
                    page.render(page_area, buf);
                }
        }

        // Render Modal
        if let Some(modal_type) = &self.app.active_modal {
            let modal_metadata: Box<dyn ModalMetadata> = match modal_type {
                ModalType::Catchup => Box::new(CatchupModal::new(&self.app.status)),
                ModalType::Exception => {
                    let message = self.app.last_error.as_deref().unwrap_or("Unknown error");
                    Box::new(ExceptionModal::new(message))
                }
                ModalType::Hybrid => Box::new(HybridModal::new()),
                ModalType::Partkey => Box::new(PartkeyModal::new()),
                ModalType::Generate => Box::new(GenerateModal::new(self.app)),
                ModalType::Lagging => Box::new(LaggingModal::new()),
                ModalType::DeleteConfirm => {
                    if let Some(key) = &self.app.selected_key {
                        let is_active = self.app.is_key_active(key);
                        Box::new(DeleteConfirmModal::new(key.id.clone(), is_active))
                    } else {
                        Box::new(ExceptionModal::new("No key selected"))
                    }
                }
                ModalType::Deleting => Box::new(DeletingModal::new()),
                ModalType::KeyInfo => {
                    if let Some(key) = &self.app.selected_key {
                        let is_active = self.app.is_key_active(key);
                        let account_incentive_eligible = self.app.accounts.iter()
                            .find(|a| a.address == key.address)
                            .and_then(|a| a.incentive_eligible)
                            .unwrap_or(false);
                        Box::new(KeyInfoModal::new(
                            key,
                            &self.app.key_info_mode,
                            &self.app.current_shortlink,
                            self.app.selected_key_just_generated,
                            is_active,
                            self.app.incentives_disabled,
                            account_incentive_eligible,
                            &self.app.key_info_success_message,
                        ))
                    } else {
                        // This case should theoretically not be hit if KeyInfo is active
                        Box::new(ExceptionModal::new("No key selected"))
                    }
                }
            };

            let width = modal_metadata.width(area.width, area.height);
            let height = modal_metadata.height(area.width, area.height);
            
            let modal_area = centered_rect(width, height, area);
            Clear.render(modal_area, buf);

            match modal_type {
                ModalType::Catchup => {
                    let modal = CatchupModal::new(&self.app.status);
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::Exception => {
                    let message = self.app.last_error.as_deref().unwrap_or("Unknown error");
                    let modal = ExceptionModal::new(message);
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::Hybrid => {
                    let modal = HybridModal::new();
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::Partkey => {
                    let modal = PartkeyModal::new();
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::Generate => {
                    let modal = GenerateModal::new(self.app);
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::Lagging => {
                    let modal = LaggingModal::new();
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::DeleteConfirm => {
                    if let Some(key) = &self.app.selected_key {
                        let is_active = self.app.is_key_active(key);
                        let modal = DeleteConfirmModal::new(key.id.clone(), is_active);
                        render_modal(modal_area, &modal, buf);
                    }
                }
                ModalType::Deleting => {
                    let modal = DeletingModal::new();
                    render_modal(modal_area, &modal, buf);
                }
                ModalType::KeyInfo => {
                    if let Some(key) = &self.app.selected_key {
                        let is_active = self.app.is_key_active(key);
                        let account_incentive_eligible = self.app.accounts.iter()
                            .find(|a| a.address == key.address)
                            .and_then(|a| a.incentive_eligible)
                            .unwrap_or(false);
                        let modal = KeyInfoModal::new(
                            key,
                            &self.app.key_info_mode,
                            &self.app.current_shortlink,
                            self.app.selected_key_just_generated,
                            is_active,
                            self.app.incentives_disabled,
                            account_incentive_eligible,
                            &self.app.key_info_success_message,
                        );
                        render_modal(modal_area, &modal, buf);
                    }
                }
            }
        }
    }
}

fn render_modal<'a, T>(area: Rect, modal: &'a T, buf: &mut Buffer) 
where 
    &'a T: ModalMetadata + Widget
{
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(ratatui::style::Style::default().fg(modal.border_color()));

    block = block.title(format!(" {} ", modal.title()));

    let controls = modal.controls();
    if !controls.is_empty() {
        block = block.title_bottom(
            Line::from(format!(" {} ", controls))
                .alignment(ratatui::layout::Alignment::Right),
        );
    }

    let inner_area = block.inner(area);
    block.render(area, buf);
    
    // Add horizontal padding of 1
    let content_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y,
        width: inner_area.width.saturating_sub(2),
        height: inner_area.height,
    };
    
    modal.render(content_area, buf);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let width = width.min(r.width);
    let height = height.min(r.height);

    let x = r.x + (r.width - width) / 2;
    let y = r.y + (r.height - height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
}
