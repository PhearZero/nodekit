use crate::ui::theme;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::event::{NodeStatus, Metrics};

pub struct StatusComponent<'a> {
    pub status: &'a Option<algod_client::models::WaitForBlock>,
    pub node_status: &'a NodeStatus,
    pub metrics: &'a Metrics,
    pub p2p_enabled: bool,
    pub p2p_hybrid_enabled: bool,
}

impl<'a> StatusComponent<'a> {
    pub fn new(
        status: &'a Option<algod_client::models::WaitForBlock>,
        node_status: &'a NodeStatus,
        metrics: &'a Metrics,
        p2p_enabled: bool,
        p2p_hybrid_enabled: bool,
    ) -> Self {
        Self {
            status,
            node_status,
            metrics,
            p2p_enabled,
            p2p_hybrid_enabled,
        }
    }
}

impl<'a> Widget for &StatusComponent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_set(theme::get_border_set())
            .border_type(theme::get_border_type())
            .border_style(Style::default().fg(Color::Magenta)); // Standard Magenta

        // Titles in the border
        let left_title = Span::styled(
            " ( Nodekit-v0.1.0 ) ",
            Style::default().fg(Color::Red),
        );
        let right_title = Span::styled(
            " Status ",
            Style::default().fg(Color::Magenta),
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

        let last_round = self.status.as_ref().map(|s| s.last_round).unwrap_or(0);
        let row1_left = Line::from(vec![
            Span::styled(" Latest Round: ", Style::default().fg(Color::LightBlue)),
            Span::raw(last_round.to_string()),
        ]);

        let (status_text, status_color) = match self.node_status {
            NodeStatus::Stable => ("STABLE ", Color::Green),
            NodeStatus::Syncing => ("SYNCING ", Color::Yellow),
            NodeStatus::FastCatchup => ("FAST-CATCHUP ", Color::Yellow),
            NodeStatus::Disconnected => ("OFFLINE ", Color::Red),
        };

        let row1_right = Line::from(vec![
            Span::styled(status_text, Style::default().fg(status_color)),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &row1_left, chunks[0].width);
        let right_x = chunks[0].x + chunks[0].width - row1_right.width() as u16;
        buf.set_line(right_x, chunks[0].y, &row1_right, row1_right.width() as u16);

// Row 3: P2P and Peers
        let p2p_status = if self.p2p_hybrid_enabled {
            Span::styled("HYBRID", Style::default().fg(Color::Green))
        } else if self.p2p_enabled {
            Span::styled("ONLY", Style::default().fg(Color::Green))
        } else {
            Span::styled("NO", Style::default().fg(Color::Red))
        };

        let row3_left = Line::from(vec![
            Span::raw(" P2P:        "),
            p2p_status,
            Span::raw(" "),
        ]);

        let mut row3_right_spans = vec![Span::raw("  Peers: ")];
        if self.p2p_hybrid_enabled {
            row3_right_spans.push(Span::raw(format!(
                " {: >4} WS | {: >4} P2P ",
                self.metrics.peers_ws, self.metrics.peers_p2p
            )));
        } else if self.p2p_enabled {
            row3_right_spans.push(Span::raw(format!("{} ", self.metrics.peers_p2p)));
        } else {
            row3_right_spans.push(Span::raw(format!("{} ", self.metrics.peers_ws)));
        }
        let row3_right = Line::from(row3_right_spans);

        buf.set_line(chunks[2].x, chunks[2].y, &row3_left, chunks[2].width);
        let right_x = chunks[2].x + chunks[2].width - row3_right.width() as u16;
        buf.set_line(right_x, chunks[2].y, &row3_right, row3_right.width() as u16);

        // Row 4: TPS and Tx
        let tps_text = if *self.node_status != NodeStatus::Stable && *self.node_status != NodeStatus::Disconnected {
            "--".to_string()
        } else if *self.node_status == NodeStatus::Disconnected {
            "OFFLINE".to_string()
        } else {
            format!("{:.2}", self.metrics.tps)
        };
        let row4_left = Line::from(vec![
            Span::styled(" TPS:        ", Style::default().fg(Color::LightBlue)),
            Span::raw(tps_text),
        ]);

        let mut tx_right_spans = vec![Span::raw("Tx: ")];
        if self.p2p_hybrid_enabled {
            tx_right_spans.push(Span::raw(format!(
                "{: >9} | {: >9} ",
                get_bit_rate(self.metrics.tx),
                get_bit_rate(self.metrics.tx_p2p)
            )));
        } else if self.p2p_enabled {
            tx_right_spans.push(Span::raw(format!("{: >9} ", get_bit_rate(self.metrics.tx_p2p))));
        } else {
            tx_right_spans.push(Span::raw(format!("{: >9} ", get_bit_rate(self.metrics.tx))));
        }
        let row4_right = Line::from(tx_right_spans);

        buf.set_line(chunks[3].x, chunks[3].y, &row4_left, chunks[3].width);
        let right_x = chunks[3].x + chunks[3].width - row4_right.width() as u16;
        buf.set_line(right_x, chunks[3].y, &row4_right, row4_right.width() as u16);

        // Row 5: Round time and Rx
        let round_time_text = if (*self.node_status != NodeStatus::Stable && *self.node_status != NodeStatus::Disconnected) || self.metrics.round_time == 0 {
            "--".to_string()
        } else if *self.node_status == NodeStatus::Disconnected {
            "OFFLINE".to_string()
        } else {
            format!("{:.2}s", self.metrics.round_time as f64 / 1000.0)
        };
        let row5_left = Line::from(vec![
            Span::styled(" Round time: ", Style::default().fg(Color::LightBlue)),
            Span::raw(round_time_text),
        ]);

        let mut rx_right_spans = vec![Span::raw("Rx: ")];
        if self.p2p_hybrid_enabled {
            rx_right_spans.push(Span::raw(format!(
                "{: >9} | {: >9} ",
                get_bit_rate(self.metrics.rx),
                get_bit_rate(self.metrics.rx_p2p)
            )));
        } else if self.p2p_enabled {
            rx_right_spans.push(Span::raw(format!("{: >9} ", get_bit_rate(self.metrics.rx_p2p))));
        } else {
            rx_right_spans.push(Span::raw(format!("{: >9} ", get_bit_rate(self.metrics.rx))));
        }
        let row5_right = Line::from(rx_right_spans);

        buf.set_line(chunks[4].x, chunks[4].y, &row5_left, chunks[4].width);
        let right_x = chunks[4].x + chunks[4].width - row5_right.width() as u16;
        buf.set_line(right_x, chunks[4].y, &row5_right, row5_right.width() as u16);
    }
}

fn get_bit_rate(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{} GB/s", bytes / (1 << 30))
    } else if bytes >= 1 << 20 {
        format!("{} MB/s", bytes / (1 << 20))
    } else if bytes >= 1 << 10 {
        format!("{} KB/s", bytes / (1 << 10))
    } else {
        format!("{} B/s", bytes)
    }
}
