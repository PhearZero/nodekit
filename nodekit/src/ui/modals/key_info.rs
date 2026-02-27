use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};
use qrcode::QrCode;
use base64::{engine::general_purpose, Engine as _};
use crate::event::{KeyInfoMode, AlgodParticipationKey};

use crate::ui::modals::ModalMetadata;

pub struct KeyInfoModal<'a> {
    pub key: &'a AlgodParticipationKey,
    pub mode: &'a KeyInfoMode,
    pub shortlink: &'a Option<String>,
    pub just_generated: bool,
    pub is_active: bool,
    pub incentives_disabled: bool,
    pub account_incentive_eligible: bool,
    pub success_message: &'a Option<String>,
}

impl<'a> ModalMetadata for KeyInfoModal<'a> {
    fn title(&self) -> String {
        match self.mode {
            KeyInfoMode::Text => "Key Information".to_string(),
            KeyInfoMode::Online | KeyInfoMode::QR => {
                if self.is_active {
                    "Register Offline".to_string()
                } else {
                    "Register Online".to_string()
                }
            }
        }
    }
    fn border_color(&self) -> Color {
        match self.mode {
            KeyInfoMode::Text => Color::Indexed(3),
            KeyInfoMode::Online | KeyInfoMode::QR => {
                if self.is_active {
                    Color::Indexed(9) // Red (Matching Go's Register Offline)
                } else {
                    Color::Indexed(2) // Green (Matching Go's Register Online)
                }
            }
        }
    }
    fn controls(&self) -> String {
        match self.mode {
            KeyInfoMode::Text => {
                if self.is_active {
                    "( (d)elete | take (t)offline )".to_string()
                } else {
                    "( (d)elete | (r)egister online )".to_string()
                }
            }
            KeyInfoMode::Online => "( (s)how QR | (esc) go back )".to_string(),
            KeyInfoMode::QR => "( (s)how link | (esc) go back )".to_string(),
        }
    }
    fn width(&self, _available_width: u16, _available_height: u16) -> u16 {
        match self.mode {
            KeyInfoMode::Text => 80,
            KeyInfoMode::Online => {
                let link = self.shortlink.as_deref().unwrap_or("https://nodekit.run");
                let status_adj = if self.is_active { "offline" } else { "online" };
                let line1 = format!("Sign this transaction to register your account as {}", status_adj);
                let line3 = "Open this URL in your browser:";
                let max_content_width = line1.len().max(line3.len()).max(link.len());
                (max_content_width as u16 + 8).min(_available_width)
            }
            KeyInfoMode::QR => {
                let qr_data = self.generate_qr_data();
                if let Ok(code) = QrCode::new(qr_data) {
                    let qr_width = code.width() as u16;
                    let status_adj = if self.is_active { "offline" } else { "online" };
                    let intro = format!("Sign this transaction to register your account as {}", status_adj);
                    let sub_intro = "Scan the QR code with Pera or press S to show a link instead";
                    let max_text_width = intro.len().max(sub_intro.len()) as u16;
                    
                    let mut content_width = qr_width.max(max_text_width);
                    if self.is_active {
                        let note1 = "Note: this will take effect after 320 rounds (~15 min.)";
                        let note2 = "Please keep your node running during this cooldown period.";
                        content_width = content_width.max(note1.len() as u16).max(note2.len() as u16);
                    }
                    (content_width + 8).min(_available_width)
                } else {
                    _available_width.saturating_sub(2)
                }
            }
        }
    }
    fn height(&self, _available_width: u16, _available_height: u16) -> u16 {
        match self.mode {
            KeyInfoMode::Text => {
                let mut h = if self.just_generated { 21 } else { 16 };
                if self.success_message.is_some() {
                    h += 2;
                }
                h
            }
            KeyInfoMode::Online => {
                if self.is_active { 12 } else { 8 }
            }
            KeyInfoMode::QR => {
                let qr_data = self.generate_qr_data();
                if let Ok(code) = QrCode::new(qr_data) {
                    let qr_height = (code.width() + 1) / 2;
                    let mut total_height = qr_height as u16 + 2; // intro + sub_intro
                    if self.is_active {
                        total_height += 4; // empty + note1 + note2 + empty
                    }
                    (total_height + 4).min(_available_height) // +4 for borders and internal padding
                } else {
                    _available_height.saturating_sub(2)
                }
            }
        }
    }
}

impl<'a> KeyInfoModal<'a> {
    pub fn new(
        key: &'a AlgodParticipationKey,
        mode: &'a KeyInfoMode,
        shortlink: &'a Option<String>,
        just_generated: bool,
        is_active: bool,
        incentives_disabled: bool,
        account_incentive_eligible: bool,
        success_message: &'a Option<String>,
    ) -> Self {
        Self {
            key,
            mode,
            shortlink,
            just_generated,
            is_active,
            incentives_disabled,
            account_incentive_eligible,
            success_message,
        }
    }

    fn generate_qr_data(&self) -> String {
        let mut parts = vec![
            "type=keyreg".to_string(),
        ];

        if !self.is_active {
            // Online registration
            let vote_key = general_purpose::URL_SAFE_NO_PAD.encode(&self.key.key.vote_participation_key);
            let selection_key = general_purpose::URL_SAFE_NO_PAD.encode(&self.key.key.selection_participation_key);
            let state_proof_key = self.key.key.state_proof_key.as_ref()
                .map(|k| general_purpose::URL_SAFE_NO_PAD.encode(k))
                .unwrap_or_default();

            parts.push(format!("votekey={}", vote_key));
            parts.push(format!("selkey={}", selection_key));
            if !state_proof_key.is_empty() {
                parts.push(format!("sprfkey={}", state_proof_key));
            }
            parts.push(format!("votefst={}", self.key.key.vote_first_valid));
            parts.push(format!("votelst={}", self.key.key.vote_last_valid));
            parts.push(format!("votekd={}", self.key.key.vote_key_dilution));

            // Incentives fee check
            if !self.incentives_disabled && !self.account_incentive_eligible {
                parts.push("fee=2000000".to_string());
            }
        } else {
            // Offline registration
            // Based on Go implementation, offline registration transaction only includes type=keyreg
            // The votefst=0 and votelst=0 parameters are often used to signify taking an account offline
            // in some tools, but Go's AUrlTxn implementation for offline sets all keyreg fields to nil.
            parts.push("votefst=0".to_string());
            parts.push("votelst=0".to_string());
        }

        format!("algorand://{}?{}", self.key.address, parts.join("&"))
    }
}

impl<'a> Widget for KeyInfoModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl<'a> ModalMetadata for &KeyInfoModal<'a> {
    fn title(&self) -> String {
        (**self).title()
    }
    fn border_color(&self) -> Color {
        (**self).border_color()
    }
    fn controls(&self) -> String {
        (**self).controls()
    }
    fn width(&self, available_width: u16, available_height: u16) -> u16 {
        (**self).width(available_width, available_height)
    }
    fn height(&self, available_width: u16, available_height: u16) -> u16 {
        (**self).height(available_width, available_height)
    }
}

impl<'a> Widget for &KeyInfoModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_area = area;

        match self.mode {
            KeyInfoMode::Text => {
                let mut constraints = vec![];
                if let Some(_msg) = self.success_message {
                    constraints.push(Constraint::Length(1)); // msg
                    constraints.push(Constraint::Length(1)); // empty
                }
                if self.just_generated {
                    constraints.push(Constraint::Length(1)); // empty
                    constraints.push(Constraint::Length(1)); // Participation keys generated.
                    constraints.push(Constraint::Length(1)); // empty
                    constraints.push(Constraint::Length(1)); // Next step: ...
                    constraints.push(Constraint::Length(1)); // Press the R key ...
                    constraints.push(Constraint::Length(1)); // empty
                }
                constraints.extend([
                    Constraint::Length(1), // Account
                    Constraint::Length(1), // ID
                    Constraint::Length(1), // empty
                    Constraint::Length(2), // Vote Key
                    Constraint::Length(2), // Selection Key
                    Constraint::Length(2), // State Proof Key
                    Constraint::Length(1), // empty
                    Constraint::Length(1), // First Valid
                    Constraint::Length(1), // Last Valid
                    Constraint::Length(1), // Key Dilution
                ]);

                let details_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(main_area);

                let label_style = Style::default().fg(Color::Cyan);
                let value_style = Style::default().fg(Color::White);
                let key_label_style = Style::default().fg(Color::Yellow);
                let round_label_style = Style::default().fg(Color::Indexed(5)); // Purple/Magenta

                let mut current_idx = 0;
                if let Some(msg) = self.success_message {
                    Line::from(msg.clone().green().bold()).render(details_chunks[0], buf);
                    current_idx = 2;
                }
                if self.just_generated {
                    Line::from("Participation keys generated.").render(details_chunks[current_idx + 1], buf);
                    Line::from("Next step: register the participation keys with the network by signing a keyreg online transaction.").render(details_chunks[current_idx + 3], buf);
                    Line::from("Press the R key to start this process.").render(details_chunks[current_idx + 4], buf);
                    current_idx += 6;
                }

                // Account
                Line::from(vec![
                    Span::styled("Account: ", label_style),
                    Span::styled(&self.key.address, value_style),
                ]).render(details_chunks[current_idx], buf);

                // ID
                Line::from(vec![
                    Span::styled("Participation ID: ", label_style),
                    Span::styled(&self.key.id, value_style),
                ]).render(details_chunks[current_idx + 1], buf);

                // Vote Key
                Paragraph::new(vec![
                    Line::from(Span::styled("Vote Key: ", key_label_style)),
                    Line::from(Span::styled(general_purpose::STANDARD.encode(&self.key.key.vote_participation_key), value_style)),
                ]).wrap(Wrap { trim: true }).render(details_chunks[current_idx + 3], buf);

                // Selection Key
                Paragraph::new(vec![
                    Line::from(Span::styled("Selection Key: ", key_label_style)),
                    Line::from(Span::styled(general_purpose::STANDARD.encode(&self.key.key.selection_participation_key), value_style)),
                ]).wrap(Wrap { trim: true }).render(details_chunks[current_idx + 4], buf);

                // State Proof Key
                let sp_key = self.key.key.state_proof_key.as_ref()
                    .map(|k| general_purpose::STANDARD.encode(k))
                    .unwrap_or_else(|| "N/A".to_string());
                Paragraph::new(vec![
                    Line::from(Span::styled("State Proof Key: ", key_label_style)),
                    Line::from(Span::styled(sp_key, value_style)),
                ]).wrap(Wrap { trim: true }).render(details_chunks[current_idx + 5], buf);

                // First Valid
                Line::from(vec![
                    Span::styled("Vote First Valid: ", round_label_style),
                    Span::styled(self.key.key.vote_first_valid.to_string(), value_style),
                ]).render(details_chunks[current_idx + 7], buf);

                // Last Valid
                Line::from(vec![
                    Span::styled("Vote Last Valid: ", round_label_style),
                    Span::styled(self.key.key.vote_last_valid.to_string(), value_style),
                ]).render(details_chunks[current_idx + 8], buf);

                // Key Dilution
                Line::from(vec![
                    Span::styled("Vote Key Dilution: ", round_label_style),
                    Span::styled(self.key.key.vote_key_dilution.to_string(), value_style),
                ]).render(details_chunks[current_idx + 9], buf);
            }
            KeyInfoMode::Online => {
                let link = self.shortlink.as_deref().unwrap_or("https://nodekit.run");
                let status_adj = if self.is_active { "offline" } else { "online" };
                let mut text = vec![
                    Line::from(format!("Sign this transaction to register your account as {}", status_adj)),
                    Line::from(""),
                    Line::from("Open this URL in your browser:"),
                    Line::from(""),
                    Line::from(link.green()),
                ];

                if self.is_active {
                    text.push(Line::from(""));
                    text.push(Line::from(Span::styled("Note: this will take effect after 320 rounds (~15 min.)", Style::default().bold())));
                    text.push(Line::from("Please keep your node running during this cooldown period."));
                }

                Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .render(main_area, buf);
            }
            KeyInfoMode::QR => {
                let status_adj = if self.is_active { "offline" } else { "online" };
                let intro = Line::from(format!("Sign this transaction to register your account as {}", status_adj));
                let sub_intro = Line::from(vec![
                    Span::raw("Scan the QR code with Pera"),
                    Span::raw(" or "),
                    Span::styled("press S to show a link instead", Style::default().fg(Color::Yellow)),
                ]);

                let qr_data = self.generate_qr_data();
                if let Ok(code) = QrCode::new(qr_data) {
                    let width = code.width();
                    let mut qr_lines = Vec::new();
                    for y in (0..width).step_by(2) {
                        let mut line = String::new();
                        for x in 0..width {
                            let top = code[(x, y)];
                            let bottom = if y + 1 < width { code[(x, y + 1)] } else { qrcode::types::Color::Light };

                            let ch = match (top, bottom) {
                                (qrcode::types::Color::Dark, qrcode::types::Color::Dark) => '█',
                                (qrcode::types::Color::Dark, qrcode::types::Color::Light) => '▀',
                                (qrcode::types::Color::Light, qrcode::types::Color::Dark) => '▄',
                                (qrcode::types::Color::Light, qrcode::types::Color::Light) => ' ',
                            };
                            line.push(ch);
                        }
                        qr_lines.push(Line::from(line));
                    }

                    let qr_width = qr_lines.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
                    let qr_height = qr_lines.len() as u16;

                    let mut required_height = qr_height + 2; // intro + sub_intro
                    if self.is_active {
                        required_height += 3; // note1 + note2 + empty (one empty is enough)
                    }

                    if qr_width > main_area.width || required_height > main_area.height {
                        let error_text = vec![
                            intro,
                            Line::from(""),
                            Line::from("QR code is available but it does not fit on screen.".red()),
                            Line::from("Adjust terminal dimensions/font size to display.".red()),
                            Line::from(""),
                            Line::from("Or press S to switch to Link view."),
                        ];
                        Paragraph::new(error_text)
                            .alignment(Alignment::Center)
                            .wrap(Wrap { trim: true })
                            .render(main_area, buf);
                        return;
                    }

                    let mut constraints = vec![
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ];

                    if self.is_active {
                        constraints.push(Constraint::Length(3)); // Note1 + Note2 + empty
                    }

                    constraints.push(Constraint::Min(0));

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(constraints)
                        .split(main_area);

                    Paragraph::new(intro).alignment(Alignment::Center).render(chunks[0], buf);
                    Paragraph::new(sub_intro).alignment(Alignment::Center).render(chunks[1], buf);

                    let qr_area = if self.is_active {
                        Paragraph::new(vec![
                            Line::from(""),
                            Line::from(Span::styled("Note: this will take effect after 320 rounds (~15 min.)", Style::default().bold())),
                            Line::from("Please keep your node running during this cooldown period."),
                        ]).alignment(Alignment::Center).wrap(Wrap { trim: true }).render(chunks[2], buf);
                        chunks[3]
                    } else {
                        chunks[2]
                    };

                    Paragraph::new(qr_lines)
                        .alignment(Alignment::Center)
                        .render(qr_area, buf);
                } else {
                    Paragraph::new("Failed to generate QR code")
                        .alignment(Alignment::Center)
                        .render(main_area, buf);
                }
            }
        }
    }
}
