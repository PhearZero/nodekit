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
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
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
        let status = StatusComponent::new();
        status.render(status_area, buf);

        // Render Protocol
        let protocol = ProtocolComponent::new();
        protocol.render(protocol_area, buf);

        // Render Page
        match self.app.current_page {
            Page::Accounts => {
                let page = AccountsPage::new();
                page.render(page_area, buf);
            }
            Page::Keys => {
                let page = KeysPage::new();
                page.render(page_area, buf);
            }
        }

        // Render Modal
        if let Some(modal_type) = &self.app.active_modal {
            let modal_area = centered_rect(60, 40, area);
            Clear.render(modal_area, buf);
            let block = Block::bordered()
                .title(" Modal ")
                .border_type(BorderType::Rounded);
            let inner_area = block.inner(modal_area);
            block.render(modal_area, buf);

            match modal_type {
                ModalType::Catchup => {
                    let modal = CatchupModal::new();
                    modal.render(inner_area, buf);
                }
                ModalType::Exception => {
                    let modal = ExceptionModal::new();
                    modal.render(inner_area, buf);
                }
                ModalType::Hybrid => {
                    let modal = HybridModal::new();
                    modal.render(inner_area, buf);
                }
                ModalType::Partkey => {
                    let modal = PartkeyModal::new();
                    modal.render(inner_area, buf);
                }
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
