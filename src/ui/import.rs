use crate::app::{App, ImportField};
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
            Constraint::Length(1), // título
            Constraint::Length(1), // espaço
            Constraint::Length(3), // campo path
            Constraint::Length(1), // espaço
            Constraint::Length(3), // campo target (só TXT)
            Constraint::Length(1), // espaço
            Constraint::Length(3), // campo engagement
            Constraint::Length(1), // resultado/erro
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

    // Título
    let title = Paragraph::new("── Importar Dados ──")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[1]);

    // Campo: caminho do arquivo
    let path_active = app.import_field == ImportField::Path;
    let path_input = Paragraph::new(app.import_path.as_str())
        .block(
            Block::default()
                .title(" Caminho do arquivo (.json ou .txt) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if path_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(path_input, center(chunks[3], 60));

    // Campo: target (obrigatório apenas para .txt)
    let is_txt = app.import_path.ends_with(".txt");
    let target_active = app.import_field == ImportField::Target;
    let target_title = if is_txt {
        " Target (obrigatório para .txt — ex: empresa.com) "
    } else {
        " Target (opcional — sobrescreve o do arquivo) "
    };
    let target_input = Paragraph::new(app.import_target.as_str())
        .block(
            Block::default()
                .title(target_title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if target_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(target_input, center(chunks[5], 60));

    // Campo: engagement
    let eng_active = app.import_field == ImportField::Engagement;
    let eng_input = Paragraph::new(app.import_engagement.as_str())
        .block(
            Block::default()
                .title(" Engagement (opcional — usa o atual se vazio) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if eng_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(eng_input, center(chunks[7], 60));

    // Resultado ou erro
    if let Some(result) = &app.import_result {
        let is_err = result.starts_with("✗");
        let color = if is_err { Color::Red } else { Color::Green };
        let msg = Paragraph::new(result.as_str())
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        f.render_widget(msg, center(chunks[8], 60));
    }

    // Dica
    let hint = Paragraph::new("Tab alternar campo  •  Enter importar  •  Esc voltar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[10]);
}
