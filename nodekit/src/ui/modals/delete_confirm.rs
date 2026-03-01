use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::modals::ModalMetadata;

pub struct DeleteConfirmModal {
    pub key_id: String,
    pub is_active: bool,
}

impl ModalMetadata for DeleteConfirmModal {
    fn title(&self) -> String {
        "( Delete Key )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Red
    }
    fn controls(&self) -> String {
        "( (y)es | (n)o )".to_string()
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        60
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        if self.is_active { 12 } else { 10 }
    }
}

impl DeleteConfirmModal {
    pub fn new(key_id: String, is_active: bool) -> Self {
        Self { key_id, is_active }
    }
}

impl Widget for DeleteConfirmModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl ModalMetadata for &DeleteConfirmModal {
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

impl Widget for &DeleteConfirmModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Are you sure you want to delete key "),
                Span::styled(&self.key_id, Style::default().fg(Color::Yellow)),
                Span::raw("?"),
            ]),
            Line::from(""),
        ];

        let warning_prefix = if cfg!(any(target_arch = "xtensa", target_arch = "riscv32", feature = "simulator")) {
            "!"
        } else {
            "⚠"
        };

        if self.is_active {
            text.push(Line::from(Span::styled(format!("{} WARNING: This key is currently active!", warning_prefix), Style::default().fg(Color::Red))));
            text.push(Line::from("You must take your node offline before deleting."));
            text.push(Line::from(""));
        }

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
