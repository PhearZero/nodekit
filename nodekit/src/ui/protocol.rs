use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Widget},
};

pub struct ProtocolComponent {
    // In a real app, this would hold data from the node
}

impl ProtocolComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &ProtocolComponent {
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
        let row1_left = Line::from(vec![
            Span::styled(" Node: ", Style::default().fg(Color::Indexed(12))),
            Span::raw("v0.1.0"),
        ]);
        let row1_right = Line::from(vec![
            Span::styled("[UPDATE AVAILABLE] ", Style::default().fg(Color::Green)),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &row1_left, chunks[0].width);
        let right_x = chunks[0].x + chunks[0].width - row1_right.width() as u16;
        buf.set_line(right_x, chunks[0].y, &row1_right, row1_right.width() as u16);

        // Row 3: Network
        let row3 = Line::from(vec![
            Span::styled(" Network: ", Style::default().fg(Color::Indexed(12))),
            Span::raw("mainnet-v1.0"),
        ]);
        buf.set_line(chunks[2].x, chunks[2].y, &row3, chunks[2].width);

        // Row 5: Protocol Upgrade
        let row5 = Line::from(vec![
            Span::styled(" Protocol Upgrade: ", Style::default().fg(Color::Indexed(12))),
            Span::raw("No"),
        ]);
        buf.set_line(chunks[4].x, chunks[4].y, &row5, chunks[4].width);
    }
}
