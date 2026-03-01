use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::ui::modals::ModalMetadata;
use ratatui::style::Color;

pub struct HybridModal {
    // Add fields as needed
}

impl ModalMetadata for HybridModal {
    fn title(&self) -> String {
        "( Hybrid Mode )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Cyan
    }
    fn controls(&self) -> String {
        "( (esc) to close )".to_string()
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        60
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        10
    }
}

impl HybridModal {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for HybridModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl ModalMetadata for &HybridModal {
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

impl Widget for &HybridModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = "Hybrid Modal Content Placeholder\n\nThis is shown when P2P Hybrid mode is available.\n\nPress 'Esc' to close.";
        let paragraph = Paragraph::new(text)
            .centered();

        paragraph.render(area, buf);
    }
}
