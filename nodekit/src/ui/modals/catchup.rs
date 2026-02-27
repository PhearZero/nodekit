use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::event::{AlgodStatus};
use crate::ui::modals::ModalMetadata;

pub struct CatchupModal<'a> {
    pub status: &'a Option<AlgodStatus>,
}

impl<'a> ModalMetadata for CatchupModal<'a> {
    fn title(&self) -> String {
        "Fast Catchup".to_string()
    }
    fn border_color(&self) -> Color {
        Color::Indexed(7)
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        let mut max_width = 52; // Base width for the message lines (without leading space)
        if let Some(s) = self.status {
            let lines = vec![
                format!("Accounts Processed:   {} / {}", s.catchpoint_processed_accounts.unwrap_or(0), s.catchpoint_total_accounts.unwrap_or(0)),
                format!("Accounts Verified:    {} / {}", s.catchpoint_verified_accounts.unwrap_or(0), s.catchpoint_total_accounts.unwrap_or(0)),
                format!("Key Values Processed: {} / {}", s.catchpoint_processed_kvs.unwrap_or(0), s.catchpoint_total_kvs.unwrap_or(0)),
                format!("Key Values Verified:  {} / {}", s.catchpoint_verified_kvs.unwrap_or(0), s.catchpoint_total_kvs.unwrap_or(0)),
                format!("Downloaded blocks:    {} / {}", s.catchpoint_acquired_blocks.unwrap_or(0), s.catchpoint_total_blocks.unwrap_or(0)),
                format!("Sync Time: {}s", s.catchup_time / 1_000_000_000),
            ];
            for line in lines {
                max_width = max_width.max(line.len() as u16);
            }
        }
        max_width + 4 // Content width + 2 border chars + 2 padding chars
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        if self.status.is_some() {
            12
        } else {
            6
        }
    }
}

impl<'a> CatchupModal<'a> {
    pub fn new(status: &'a Option<AlgodStatus>) -> Self {
        Self { status }
    }
}

impl<'a> Widget for CatchupModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl<'a> ModalMetadata for &CatchupModal<'a> {
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

impl<'a> Widget for &CatchupModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(Color::Indexed(14));
        let mut lines = vec![
            Line::from("Please wait while your node syncs with the network.").style(style),
            Line::from("This process can take up to an hour.").style(style),
            Line::from(""),
        ];

        if let Some(s) = self.status {
            lines.push(Line::from(format!("Accounts Processed:   {} / {}", s.catchpoint_processed_accounts.unwrap_or(0), s.catchpoint_total_accounts.unwrap_or(0))).style(style));
            lines.push(Line::from(format!("Accounts Verified:    {} / {}", s.catchpoint_verified_accounts.unwrap_or(0), s.catchpoint_total_accounts.unwrap_or(0))).style(style));
            lines.push(Line::from(format!("Key Values Processed: {} / {}", s.catchpoint_processed_kvs.unwrap_or(0), s.catchpoint_total_kvs.unwrap_or(0))).style(style));
            lines.push(Line::from(format!("Key Values Verified:  {} / {}", s.catchpoint_verified_kvs.unwrap_or(0), s.catchpoint_total_kvs.unwrap_or(0))).style(style));
            lines.push(Line::from(format!("Downloaded blocks:    {} / {}", s.catchpoint_acquired_blocks.unwrap_or(0), s.catchpoint_total_blocks.unwrap_or(0))).style(style));
            lines.push(Line::from(""));
            lines.push(Line::from(format!("Sync Time: {}s", s.catchup_time / 1_000_000_000)).style(style));
        }

        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .render(area, buf);
    }
}
