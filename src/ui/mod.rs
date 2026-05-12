pub mod asns;
pub mod create_engagement;
pub mod dashboard;
pub mod help;
pub mod home;
pub mod image_preview;
pub mod import;
pub mod ips;
pub mod list_engagements;
pub mod password;
pub mod screenshots;
pub mod subdomain_menu;
pub mod subdomains;
pub mod target_menu;
pub mod targets;
pub mod technologies;
pub mod urls;

use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Modal de confirmação reutilizável
/// Renderiza por cima de qualquer tela quando app.confirm_delete == true
pub fn draw_confirm_modal(f: &mut Frame, app: &App) {
    if !app.confirm_delete {
        return;
    }

    let area = f.area();

    // Caixa centralizada
    let modal = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .split(area)[1];

    let modal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(50),
            Constraint::Fill(1),
        ])
        .split(modal)[1];

    f.render_widget(Clear, modal);

    let text = format!(
        "\n  Deletar: {}\n\n  Enter confirmar  •  Esc cancelar",
        app.confirm_delete_label
    );

    let modal_widget = Paragraph::new(text)
        .block(
            Block::default()
                .title(" ⚠ Confirmar exclusão ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(modal_widget, modal);
}
