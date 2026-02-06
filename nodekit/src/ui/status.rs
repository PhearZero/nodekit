use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Widget},
};

pub struct StatusComponent {
    // In a real app, this would hold data from the node
}

impl StatusComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &StatusComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Indexed(5))); // Color "5" is Magenta

        // Titles in the border
        let left_title = Span::styled(
            " ( Nodekit-v0.1.0 ) ",
            Style::default().fg(Color::Red),
        );
        let right_title = Span::styled(
            " Status ",
            Style::default().fg(Color::Indexed(5)),
        );

        // We can't easily put two titles in the same border with Ratatui's default Block 
        // in a way that exactly matches Lipgloss's `WithTitles` which injects into the top line.
        // But we can use the top and bottom or just custom rendering.
        // Actually, Ratatui's Block supports multiple titles since 0.26.0
        let block = block
            .title(left_title.into_left_aligned_line())
            .title(right_title.into_right_aligned_line());

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

        // Row 1: Latest Round and State
        let row1_left = Line::from(vec![
            Span::styled(" Latest Round: ", Style::default().fg(Color::Indexed(12))),
            Span::raw("0"),
        ]);
        let row1_right = Line::from(vec![
            Span::styled("STABLE ", Style::default().fg(Color::Green)),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &row1_left, chunks[0].width);
        let right_x = chunks[0].x + chunks[0].width - row1_right.width() as u16;
        buf.set_line(right_x, chunks[0].y, &row1_right, row1_right.width() as u16);

        // Row 3: P2P and Peers
        let row3_left = Line::from(vec![
            Span::raw(" P2P:        "),
            Span::styled("NO", Style::default().fg(Color::Red)),
        ]);
        let row3_right = Line::from(vec![
            Span::raw("  Peers: 0 "),
        ]);
        buf.set_line(chunks[2].x, chunks[2].y, &row3_left, chunks[2].width);
        let right_x = chunks[2].x + chunks[2].width - row3_right.width() as u16;
        buf.set_line(right_x, chunks[2].y, &row3_right, row3_right.width() as u16);

        // Row 4: TPS and Tx
        let row4_left = Line::from(vec![
            Span::styled(" TPS:        ", Style::default().fg(Color::Indexed(12))),
            Span::raw("0.00"),
        ]);
        let row4_right = Line::from(vec![
            Span::raw("Tx: 0 B/s "),
        ]);
        buf.set_line(chunks[3].x, chunks[3].y, &row4_left, chunks[3].width);
        let right_x = chunks[3].x + chunks[3].width - row4_right.width() as u16;
        buf.set_line(right_x, chunks[3].y, &row4_right, row4_right.width() as u16);

        // Row 5: Round time and Rx
        let row5_left = Line::from(vec![
            Span::styled(" Round time: ", Style::default().fg(Color::Indexed(12))),
            Span::raw("0.00s"),
        ]);
        let row5_right = Line::from(vec![
            Span::raw("Rx: 0 B/s "),
        ]);
        buf.set_line(chunks[4].x, chunks[4].y, &row5_left, chunks[4].width);
        let right_x = chunks[4].x + chunks[4].width - row5_right.width() as u16;
        buf.set_line(right_x, chunks[4].y, &row5_right, row5_right.width() as u16);
    }
}
