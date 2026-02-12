use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Widget,
};

use crate::ui::modals::ModalMetadata;

pub struct ExceptionModal<'a> {
    pub message: &'a str,
}

impl<'a> ModalMetadata for ExceptionModal<'a> {
    fn title(&self) -> String {
        "( Error )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Indexed(9)
    }
    fn controls(&self) -> String {
        "( (esc) to close )".to_string()
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        (self.message.len() as u16 + 4).max(30).min(80)
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        (self.message.lines().count() as u16 + 4).max(6)
    }
}

impl<'a> ExceptionModal<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl<'a> Widget for ExceptionModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl<'a> ModalMetadata for &ExceptionModal<'a> {
    fn title(&self) -> String {
        (**self).title()
    }
    fn border_color(&self) -> Color {
        (**self).border_color()
    }
    fn controls(&self) -> String {
        (**self).controls()
    }
    fn width(&self, available_width: u16, available_height: u16) -> u16 {
        (**self).width(available_width, available_height)
    }
    fn height(&self, available_width: u16, available_height: u16) -> u16 {
        (**self).height(available_width, available_height)
    }
}

impl<'a> Widget for &ExceptionModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(self.message).style(Style::default().fg(Color::Red));
        text.render(area, buf);
    }
}
