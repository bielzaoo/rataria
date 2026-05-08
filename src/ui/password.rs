use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    // Fundo escuro
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(50),
            Constraint::Fill(1),
        ])
        .split(chunks[1]);

    let box_area = horizontal[1];

    // Mascara a senha com asteriscos
    let masked: String = "*".repeat(app.password_input.len());

    let input = Paragraph::new(masked)
        .block(
            Block::default()
                .title(" 🔐 Senha Master ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(input, box_area);

    // Exibe erro se houver
    if let Some(err) = &app.password_error {
        let error_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(7),
                Constraint::Length(2),
                Constraint::Fill(1),
            ])
            .split(area);

        let err_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(50),
                Constraint::Fill(1),
            ])
            .split(error_chunks[2]);

        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);

        f.render_widget(error_msg, err_horizontal[1]);
    }

    // Dica na parte de baixo
    let hint = Paragraph::new("Enter para confirmar  •  Esc para sair")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    let hint_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(area);

    f.render_widget(hint, hint_chunks[1]);
}
