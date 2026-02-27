use crate::event::{NodeStatus, AlgodAccount, AlgodParticipationKey};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Row, Table, Widget},
};
use chrono::{Utc, Duration};

pub struct AccountsPage<'a> {
    pub accounts: &'a [AlgodAccount],
    pub keys: &'a [AlgodParticipationKey],
    pub last_round: u64,
    pub avg_round_time: u64,
    pub selected_index: usize,
    pub node_status: &'a NodeStatus,
}

impl<'a> AccountsPage<'a> {
    pub fn new(
        accounts: &'a [AlgodAccount],
        keys: &'a [AlgodParticipationKey],
        last_round: u64,
        avg_round_time: u64,
        selected_index: usize,
        node_status: &'a NodeStatus,
    ) -> Self {
        Self {
            accounts,
            keys,
            last_round,
            avg_round_time,
            selected_index,
            node_status,
        }
    }
}

impl<'a> Widget for &AccountsPage<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Indexed(6))); // Color "6" is Cyan/Teal-ish

        // Titles and Navigation
        let title = Span::styled(" Accounts ", Style::default());
        let controls = Span::styled(
            " ( (g)enerate | (enter) to select ) ",
            Style::default(),
        );
        let navigation = Line::from(vec![
            Span::raw(" | <- | "),
            Span::styled("accounts", Style::default().fg(Color::Green)),
            Span::raw(" | keys | "),
        ]);

        let block = block
            .title(title)
            .title_bottom(controls)
            .title_bottom(navigation.alignment(ratatui::layout::Alignment::Right));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let header_cells = ["Account", "Status", "Incentive", "Expires", "Balance"]
            .into_iter()
            .map(|h| Cell::from(h).style(Style::default().fg(Color::Indexed(240))));
        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1);

        let rows: Vec<Row> = self.accounts.iter().map(|account| {
            // Calculate Status
            let is_online = account.status == "Online";
            let mut expires_round = 0;
            let mut has_resident_key = false;

            if let Some(part) = &account.participation {
                for key in self.keys {
                    if key.address == account.address &&
                       key.key.vote_first_valid == part.vote_first_valid &&
                       key.key.vote_last_valid == part.vote_last_valid &&
                       key.key.vote_participation_key == part.vote_participation_key {
                        has_resident_key = true;
                        break;
                    }
                }
            }

            for key in self.keys {
                if key.address == account.address {
                    expires_round = expires_round.max(key.key.vote_last_valid);
                }
            }
            
            let is_expired = expires_round > 0 && expires_round < self.last_round;
            let status = if is_online && !is_expired {
                "PARTICIPATING"
            } else {
                "IDLE"
            };

            // Calculate Incentive
            let incentive = if account.incentive_eligible.unwrap_or(false) && status == "PARTICIPATING" {
                let balance = account.amount / 1_000_000;
                if balance >= 30_000 && balance <= 70_000_000 {
                     "ELIGIBLE"
                } else {
                     "PAUSED"
                }
            } else if status == "PARTICIPATING" {
                "INELIGIBLE"
            } else {
                ""
            };

            // Calculate Expires
            let mut non_resident = false;
            if is_online && !has_resident_key {
                non_resident = true;
            }

            let expires = if *self.node_status == NodeStatus::FastCatchup {
                "SYNCING".to_string()
            } else if non_resident && !is_expired && expires_round != 0 {
                "⚠ NON-RESIDENT-KEY".to_string()
            } else if expires_round == 0 {
                "N/A".to_string()
            } else if is_expired {
                "EXPIRED".to_string()
            } else {
                let round_diff = expires_round.saturating_sub(self.last_round);
                let duration_ms = round_diff * self.avg_round_time;
                let expires_at = Utc::now() + Duration::milliseconds(duration_ms as i64);
                
                let mut expires_str = expires_at.format("%d %b %y %H:%M").to_string();

                // Warning if expires within a week
                if expires_at < Utc::now() + Duration::days(7) {
                    expires_str = format!("⚠ {}", expires_str);
                }
                expires_str
            };

            Row::new(vec![
                Cell::from(account.address.clone()),
                Cell::from(status),
                Cell::from(incentive),
                Cell::from(expires),
                Cell::from((account.amount / 1_000_000).to_string()),
            ])
        }).collect();

        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(Some(self.selected_index));

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .row_highlight_style(
            Style::default()
                .fg(Color::Indexed(229))
                .bg(Color::Indexed(6)),
        )
        .highlight_symbol(">> ");

        ratatui::widgets::StatefulWidget::render(table, inner_area, buf, &mut table_state);
    }
}
