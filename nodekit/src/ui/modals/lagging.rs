use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::ui::modals::ModalMetadata;

pub struct LaggingModal;

impl ModalMetadata for LaggingModal {
    fn title(&self) -> String {
        "( Out of Sync )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Red
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        60
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        30
    }
}

impl LaggingModal {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for LaggingModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl ModalMetadata for &LaggingModal {
    fn title(&self) -> String {
        (**self).title()
    }
    fn border_color(&self) -> Color {
        (**self).border_color()
    }
    fn width(&self, available_width: u16, available_height: u16) -> u16 {
        (**self).width(available_width, available_height)
    }
    fn height(&self, available_width: u16, available_height: u16) -> u16 {
        (**self).height(available_width, available_height)
    }
}

impl Widget for &LaggingModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = vec![
            Line::from(""),
            Line::from("Your node is significantly behind the network."),
            Line::from("Would you like to perform a fast-catchup?"),
            Line::from(""),
        ];

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
