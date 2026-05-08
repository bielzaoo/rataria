use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

const BANNER: &str = r#"
██████╗  █████╗ ████████╗ █████╗ ██████╗ ██╗ █████╗
██╔══██╗██╔══██╗╚══██╔══╝██╔══██╗██╔══██╗██║██╔══██╗
██████╔╝███████║   ██║   ███████║██████╔╝██║███████║
██╔══██╗██╔══██║   ██║   ██╔══██║██╔══██╗██║██╔══██║
██║  ██║██║  ██║   ██║   ██║  ██║██║  ██║██║██║  ██║
╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═╝
"#;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // banner
            Constraint::Length(1), // subtítulo
            Constraint::Fill(1),   // menu
            Constraint::Length(1), // dica
        ])
        .split(area);

    // Banner ASCII
    let banner = Paragraph::new(BANNER)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(banner, chunks[0]);

    // Subtítulo
    let subtitle = Paragraph::new("Pentest Recon Manager")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(subtitle, chunks[1]);

    // Menu centralizado
    let menu_items: Vec<ListItem> = App::home_menu_items()
        .iter()
        .map(|item| ListItem::new(*item))
        .collect();

    let menu = List::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let menu_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(40),
            Constraint::Fill(1),
        ])
        .split(chunks[2]);

    let mut list_state = ListState::default();
    list_state.select(Some(app.home_selected));

    f.render_stateful_widget(menu, menu_horizontal[1], &mut list_state);

    // Dica de navegação
    let hint = Paragraph::new("↑↓ navegar  •  Enter selecionar  •  q sair")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);
}
