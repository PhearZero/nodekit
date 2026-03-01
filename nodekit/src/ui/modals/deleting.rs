use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::ui::modals::ModalMetadata;

pub struct DeletingModal;

impl ModalMetadata for DeletingModal {
    fn title(&self) -> String {
        "( Deleting Key )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Red
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        40
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        6
    }
}

impl DeletingModal {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for DeletingModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl ModalMetadata for &DeletingModal {
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

impl Widget for &DeletingModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = vec![
            Line::from(""),
            Line::from("Deleting key...".yellow().bold()),
            Line::from("Please wait."),
        ];

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
