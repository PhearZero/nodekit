use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Row, Table, Widget},
};

pub struct KeysPage<'a> {
    pub keys: &'a [algod_client::models::ParticipationKey],
    pub selected_index: usize,
    pub participation: &'a Option<algod_client::models::AccountParticipation>,
}

impl<'a> KeysPage<'a> {
    pub fn new(
        keys: &'a [algod_client::models::ParticipationKey],
        selected_index: usize,
        participation: &'a Option<algod_client::models::AccountParticipation>,
    ) -> Self {
        Self {
            keys,
            selected_index,
            participation,
        }
    }
}

impl<'a> Widget for &KeysPage<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Indexed(4))); // Color "4" is Blue

        // Titles and Navigation
        let title = Span::styled(" Keys ", Style::default());
        let controls = Span::styled(
            " ( (g)enerate | (enter) to select | (esc) to go back ) ",
            Style::default(),
        );
        let navigation = Line::from(vec![
            Span::raw(" | <- | accounts | "),
            Span::styled("keys", Style::default().fg(Color::Green)),
            Span::raw(" | "),
        ]);

        let block = block
            .title(title)
            .title_bottom(controls)
            .title_bottom(navigation.alignment(ratatui::layout::Alignment::Right));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let header_cells = ["ID", "Address", "Active", "Last Vote", "Last Block Proposal"]
            .into_iter()
            .map(|h| Cell::from(h).style(Style::default().fg(Color::Indexed(240))));
        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1);

        let rows: Vec<Row> = self.keys.iter().map(|key| {
            let is_active = if let Some(part) = self.participation {
                if key.key.vote_first_valid == part.vote_first_valid &&
                   key.key.vote_last_valid == part.vote_last_valid &&
                   key.key.vote_participation_key == part.vote_participation_key {
                    "YES"
                } else {
                    "NO"
                }
            } else {
                "N/A"
            };

            Row::new(vec![
                Cell::from(key.id.clone()),
                Cell::from(key.address.clone()),
                Cell::from(is_active),
                Cell::from(if key.last_vote.unwrap_or(0) == 0 { "N/A".to_string() } else { key.last_vote.unwrap_or(0).to_string() }),
                Cell::from(if key.last_block_proposal.unwrap_or(0) == 0 { "N/A".to_string() } else { key.last_block_proposal.unwrap_or(0).to_string() }),
            ])
        }).collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .row_highlight_style(
            Style::default()
                .fg(Color::Indexed(229))
                .bg(Color::Indexed(4)),
        )
        .highlight_symbol(">> ");

        let mut state = ratatui::widgets::TableState::default();
        state.select(Some(self.selected_index));

        ratatui::widgets::StatefulWidget::render(table, inner_area, buf, &mut state);
    }
}
