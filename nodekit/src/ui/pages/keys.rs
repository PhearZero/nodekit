use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{block::Title, Block, BorderType, Cell, Row, Table, Widget},
};

pub struct KeysPage {
    // In a real app, this would hold keys data
}

impl KeysPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &KeysPage {
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

        let header_cells = ["ID", "Address", "Active", "Last Vote", "Last Block Proposal"]
            .into_iter()
            .map(|h| Cell::from(h).style(Style::default().fg(Color::Indexed(240))));
        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1);

        // Placeholder row
        let rows = vec![Row::new(vec![
            Cell::from("PART-1"),
            Cell::from("7Z58..."),
            Cell::from("YES"),
            Cell::from("N/A"),
            Cell::from("N/A"),
        ])];

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

        table.render(inner_area, buf);
    }
}
