use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::ui::modals::ModalMetadata;

pub struct PartkeyModal;

impl ModalMetadata for PartkeyModal {
    fn title(&self) -> String {
        "( Participation Keys )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Cyan
    }
    fn controls(&self) -> String {
        "( (g)enerate | (esc) to close )".to_string()
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        60
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        12
    }
}

impl PartkeyModal {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for PartkeyModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl ModalMetadata for &PartkeyModal {
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

impl Widget for &PartkeyModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = vec![
            Line::from(""),
            Line::from("Participation keys are required for your node to"),
            Line::from("participate in consensus and earn incentives."),
            Line::from(""),
            Line::from("This process is safe. Your spending keys never leave"),
            Line::from("your wallet. You only need to provide the participation"),
            Line::from("details to the node."),
            Line::from(""),
            Line::from("To generate a new key, press 'g' or use the 'goal'"),
            Line::from("command on your node."),
            Line::from(""),
        ];

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
