use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};
use crate::app::{App, GenerateStep, GenerateRange};

use crate::ui::modals::ModalMetadata;

pub struct GenerateModal<'a> {
    pub app: &'a App,
}

impl<'a> ModalMetadata for GenerateModal<'a> {
    fn title(&self) -> String {
        "( Generate Participation Key )".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Indexed(14)
    }
    fn controls(&self) -> String {
        match self.app.generate_step {
            GenerateStep::Address => "( (enter) to continue | (esc) to close )".to_string(),
            GenerateStep::Duration => "( (s) toggle range | (enter) to generate | (esc) to close )".to_string(),
            GenerateStep::Waiting => {
                if self.app.generate_error.is_some() {
                    "( (esc) to close )".to_string()
                } else {
                    String::new()
                }
            }
        }
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        match self.app.generate_step {
            GenerateStep::Address => 60,
            GenerateStep::Duration => 40,
            GenerateStep::Waiting => 50,
        }
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        match self.app.generate_step {
            GenerateStep::Address => 8,
            GenerateStep::Duration => 8,
            GenerateStep::Waiting => 10,
        }
    }
}

impl<'a> GenerateModal<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for GenerateModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl<'a> ModalMetadata for &GenerateModal<'a> {
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

impl<'a> Widget for &GenerateModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.app.generate_step {
            GenerateStep::Address => {
                let text = vec![
                    Line::from(""),
                    Line::from("Enter the account address to generate keys for:"),
                    Line::from(""),
                    Line::from(Span::styled(&self.app.generate_address, Style::default().fg(Color::Yellow))),
                ];
                Paragraph::new(text).render(area, buf);
            }
            GenerateStep::Duration => {
                let range_text = match self.app.generate_range {
                    GenerateRange::Day => "Days",
                    GenerateRange::Month => "Months",
                    GenerateRange::Round => "Rounds",
                };
                let text = vec![
                    Line::from(""),
                    Line::from(format!("Enter the duration in {}:", range_text)),
                    Line::from(""),
                    Line::from(Span::styled(&self.app.generate_duration, Style::default().fg(Color::Yellow))),
                ];
                Paragraph::new(text).render(area, buf);
            }
            GenerateStep::Waiting => {
                let mut text = if self.app.generate_error.is_some() {
                    vec![
                        Line::from(""),
                        Line::from(""),
                        Line::from("Key generation failed".red().bold()),
                    ]
                } else {
                    vec![
                        Line::from(""),
                        Line::from(""),
                        Line::from("Generating keys...".yellow().bold()),
                        Line::from("This may take several minutes."),
                        Line::from(""),
                        Line::from("Please do not close the application."),
                    ]
                };
                
                if let Some(err) = &self.app.generate_error {
                    text.push(Line::from(""));
                    let err_lines: Vec<Line> = err.lines()
                        .map(|l| Line::from(l.red()))
                        .collect();
                    text.extend(err_lines);
                }
                
                Paragraph::new(text).alignment(Alignment::Center).wrap(Wrap { trim: true }).render(area, buf);
            }
        }
    }
}
