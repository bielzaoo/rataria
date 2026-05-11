use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Fill(1),   // conteúdo
            Constraint::Length(1), // mensagem resultado
            Constraint::Length(1), // dica
        ])
        .split(area);

    // Header com nome do engagement
    let title = match &app.current_engagement {
        Some(e) => format!(" 🎯 {} ", e.name),
        None => " 🎯 Dashboard ".to_string(),
    };

    let header = Paragraph::new(title.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Layout lateral: menu | painel
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // menu lateral
            Constraint::Fill(1),    // painel direito
        ])
        .split(chunks[1]);

    // Menu lateral
    let items: Vec<ListItem> = App::dashboard_menu_items()
        .iter()
        .map(|i| ListItem::new(*i))
        .collect();

    let menu = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.dashboard_selected));
    f.render_stateful_widget(menu, body[0], &mut state);

    // Painel direito — resumo da seção selecionada
    let section = App::dashboard_menu_items()[app.dashboard_selected];
    let content = match section {
        "Targets" => format_targets(app),
        _ => format!("  {} — em breve", section),
    };

    let panel = Paragraph::new(content)
        .block(
            Block::default()
                .title(format!(" {} ", section))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(panel, body[1]);

    // Mensagem de resultado
    if let Some(msg) = &app.form_error {
        let color = if msg.starts_with('✓') {
            Color::Green
        } else {
            Color::Red
        };
        let msg_widget = Paragraph::new(msg.as_str())
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        f.render_widget(msg_widget, chunks[2]);
    }

    // Dica
    let hint = Paragraph::new("↑↓ navegar  •  Enter abrir seção  •  E exportar MD  •  Esc voltar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);
}

fn format_targets(app: &App) -> String {
    if app.targets.is_empty() {
        return "  Nenhum target cadastrado.".to_string();
    }
    app.targets
        .iter()
        .map(|t| format!("  • {}", t.domain))
        .collect::<Vec<_>>()
        .join("\n")
}
