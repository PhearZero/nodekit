use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

pub struct LoadingComponent<'a> {
    app: &'a App,
}

impl<'a> LoadingComponent<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for &LoadingComponent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(7), // Banner
                Constraint::Length(1), // Message
                Constraint::Length(2), // Animation
                Constraint::Fill(1),
            ])
            .split(area);

        let banner_area = chunks[1];
        let message_area = chunks[2];
        let animation_area = chunks[3];

        // Stylized NODEKIT banner
        // Blue NODE, Cyan KIT
        let banner = vec![
            Line::from(vec![
                Span::styled("█   █  █████  ████   █████   ", Style::default().fg(Color::Blue)),
                Span::styled("█──█  ███  █████", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("██  █  █   █  █   █  █       ", Style::default().fg(Color::Blue)),
                Span::styled("█ ▄█   █     █  ", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("█ █ █  █   █  █   █  ████    ", Style::default().fg(Color::Blue)),
                Span::styled("██     █     █  ", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("█  ██  █   █  █   █  █       ", Style::default().fg(Color::Blue)),
                Span::styled("█ ▀█   █     █  ", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("█   █  █████  ████   █████   ", Style::default().fg(Color::Blue)),
                Span::styled("█──█  ███    █  ", Style::default().fg(Color::Cyan)),
            ]),
        ];

        Paragraph::new(banner)
            .alignment(Alignment::Center)
            .render(banner_area, buf);

        if let Some(msg) = &self.app.loading_message {
            Paragraph::new(msg.as_str())
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray))
                .render(message_area, buf);
        }

        // Loading animation (spinner)
        let spinner_chars = ["|", "/", "-", "\\"];
        let tick = self.app.counter as usize % spinner_chars.len();
        let animation = Line::from(vec![
            Span::raw("Loading "),
            Span::styled(spinner_chars[tick], Style::default().fg(Color::Yellow)),
        ]);

        Paragraph::new(animation)
            .alignment(Alignment::Center)
            .render(animation_area, buf);
    }
}
