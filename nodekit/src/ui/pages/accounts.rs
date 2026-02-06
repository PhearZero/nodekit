use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{block::Title, Block, BorderType, Cell, Row, Table, Widget},
};

pub struct AccountsPage {
    // In a real app, this would hold account data
}

impl AccountsPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &AccountsPage {
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
            .title(Title::from(title).alignment(ratatui::layout::Alignment::Left))
            .title(
                Title::from(controls)
                    .alignment(ratatui::layout::Alignment::Left)
                    .position(ratatui::widgets::block::Position::Bottom),
            )
            .title(
                Title::from(navigation)
                    .alignment(ratatui::layout::Alignment::Right)
                    .position(ratatui::widgets::block::Position::Bottom),
            );

        let inner_area = block.inner(area);
        block.render(area, buf);

        let header_cells = ["Account", "Status", "Rewards", "Expires", "Balance"]
            .into_iter()
            .map(|h| Cell::from(h).style(Style::default().fg(Color::Indexed(240))));
        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1);

        // Placeholder row
        let rows = vec![Row::new(vec![
            Cell::from("7Z58..."),
            Cell::from("IDLE"),
            Cell::from(""),
            Cell::from("SYNCING"),
            Cell::from("0"),
        ])];

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

        table.render(inner_area, buf);
    }
}
