use crate::app::{App, FormField};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3), // campo nome
            Constraint::Length(1), // espaço
            Constraint::Length(3), // campo descrição
            Constraint::Length(1), // erro
            Constraint::Fill(1),
            Constraint::Length(1), // dica
        ])
        .split(area);

    let center = |area, width| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(width),
                Constraint::Fill(1),
            ])
            .split(area)[1]
    };

    let name_area = center(chunks[1], 50);
    let desc_area = center(chunks[3], 50);
    let err_area = center(chunks[4], 50);
    let hint_area = chunks[6];

    // Campo Nome
    let name_active = app.form_field == FormField::Name;
    let name_input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Nome do Engagement ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if name_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(name_input, name_area);

    // Campo Descrição
    let desc_active = app.form_field == FormField::Description;
    let desc_input = Paragraph::new(app.form_description.as_str())
        .block(
            Block::default()
                .title(" Descrição (opcional) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if desc_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(desc_input, desc_area);

    // Erro
    if let Some(err) = &app.form_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_area);
    }

    // Dica
    let hint = Paragraph::new("Tab alternar campo  •  Enter confirmar  •  Esc voltar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, hint_area);
}
