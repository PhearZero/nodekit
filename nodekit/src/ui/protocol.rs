use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Widget},
};

use crate::event::Metrics;

pub struct ProtocolComponent<'a> {
    pub status: &'a Option<algod_client::models::WaitForBlock>,
    pub version: &'a Option<algod_client::models::Version>,
    pub update_available: bool,
    pub metrics: &'a Metrics,
}

impl<'a> ProtocolComponent<'a> {
    pub fn new(
        status: &'a Option<algod_client::models::WaitForBlock>,
        version: &'a Option<algod_client::models::Version>,
        update_available: bool,
        metrics: &'a Metrics,
    ) -> Self {
        Self { status, version, update_available, metrics }
    }
}

impl<'a> Widget for &ProtocolComponent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Protocol ")
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Indexed(5))); // Magenta

        let inner_area = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner_area);

        // Row 1: Node version and Update Availability
        let version_str = self.version.as_ref().map(|v| {
            format!("v{}.{}.{}-{}", v.build.major, v.build.minor, v.build.build_number, v.build.channel)
        }).unwrap_or_else(|| self.status.as_ref().map(|s| s.last_version.clone()).unwrap_or_else(|| "v0.0.0".to_string()));
        
        let row1_left = Line::from(vec![
            Span::styled(" Node: ", Style::default().fg(Color::Indexed(12))),
            Span::raw(version_str),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &row1_left, chunks[0].width);

        if self.update_available {
            let row1_right = Line::from(vec![
                Span::styled("[UPDATE AVAILABLE] ", Style::default().fg(Color::Green)),
            ]);
            let right_x = chunks[0].x + chunks[0].width - row1_right.width() as u16;
            buf.set_line(right_x, chunks[0].y, &row1_right, row1_right.width() as u16);
        }

        // Row 3: Network
        let network = self.version.as_ref().map(|v| v.genesis_id.clone()).unwrap_or_else(|| "N/A".to_string());
        let row3 = Line::from(vec![
            Span::styled(" Network: ", Style::default().fg(Color::Indexed(12))),
            Span::raw(network),
        ]);
        buf.set_line(chunks[2].x, chunks[2].y, &row3, chunks[2].width);

        // Row 5: Protocol Upgrade
        let upgrade = if let Some(s) = self.status {
            format_protocol_vote(s, self.metrics)
        } else {
            "No".to_string()
        };
        let row5 = Line::from(vec![
            Span::styled(" Protocol Upgrade: ", Style::default().fg(Color::Indexed(12))),
            Span::raw(upgrade),
        ]);
        buf.set_line(chunks[4].x, chunks[4].y, &row5, chunks[4].width);
    }
}

fn format_protocol_vote(status: &algod_client::models::WaitForBlock, metrics: &Metrics) -> String {
    let last_round = status.last_round;
    let next_version_round = status.next_version_round;

    if next_version_round > last_round + 1 {
        return format_scheduled_upgrade(status, metrics);
    }

    let yes_votes = status.upgrade_yes_votes.unwrap_or(0);
    let no_votes = status.upgrade_no_votes.unwrap_or(0);
    let voting = yes_votes > 0 || no_votes > 0;
    
    if !voting {
        return "No".to_string();
    }

    let vote_rounds = status.upgrade_vote_rounds.unwrap_or(0);
    let votes_required = status.upgrade_votes_required.unwrap_or(0);
    let total_votes_cast = yes_votes + no_votes;

    if total_votes_cast == 0 || vote_rounds == 0 {
        return "No".to_string();
    }

    let percentage_progress = 100 * total_votes_cast / vote_rounds;
    let percentage_yes = 100 * yes_votes / total_votes_cast;

    let mut label = "Yes";
    let mut percentage_vote_display = percentage_yes;
    if percentage_yes < 50 {
        label = "No";
        percentage_vote_display = 100 * no_votes / total_votes_cast;
    }

    let mut status_string = format!("Voting {}% complete, {}% {}", percentage_progress, percentage_vote_display, label);

    let passing = yes_votes > votes_required;
    if passing {
        status_string.push_str(", will pass");
    }
    
    let fail_threshold = vote_rounds.saturating_sub(votes_required);
    if no_votes > fail_threshold {
        status_string.push_str(", will fail");
    }

    status_string
}

fn format_scheduled_upgrade(status: &algod_client::models::WaitForBlock, metrics: &Metrics) -> String {
    let round_delta = status.next_version_round.saturating_sub(status.last_round);
    let eta_ms = round_delta * metrics.round_time;
    
    let minutes = (eta_ms / 60000) % 60;
    let hours = (eta_ms / 3600000) % 24;
    let days = eta_ms / 86400000;

    let mut str = "Scheduled".to_string();
    if days > 0 {
        str.push_str(&format!(" {} {}", days, if days == 1 { "day" } else { "days" }));
    }
    if hours > 0 {
        str.push_str(&format!(" {} {}", hours, if hours == 1 { "hour" } else { "hours" }));
    }
    if days == 0 && minutes > 0 {
        str.push_str(&format!(" {} {}", minutes, if minutes == 1 { "min" } else { "mins" }));
    }
    str
}
