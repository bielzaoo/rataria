use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Clear, area);

    // Divide verticalmente: espaço | caixa(3) | erro(1) | espaço | dica(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Centraliza horizontalmente — largura 44
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

    let box_area = center(chunks[1], 44);
    let err_area = center(chunks[2], 44);
    let hint_area = chunks[4];

    // Input com asteriscos
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

    // Mensagem de erro (se houver)
    if let Some(err) = &app.password_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_area);
    }

    // Dica no rodapé
    let hint = Paragraph::new("Enter para confirmar  •  Esc para sair")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, hint_area);
}
