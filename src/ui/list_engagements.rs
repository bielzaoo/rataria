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
            Constraint::Length(1), // título
            Constraint::Length(1), // espaço
            Constraint::Fill(1),   // lista
            Constraint::Length(1), // dica
        ])
        .split(area);

    // Título
    let title = Paragraph::new("── Engagements ──")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Lista vazia
    if app.engagements.is_empty() {
        let empty = Paragraph::new("Nenhum engagement encontrado. Crie um novo na tela inicial.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, chunks[2]);
    } else {
        let items: Vec<ListItem> = app
            .engagements
            .iter()
            .map(|e| {
                let label = match &e.description {
                    Some(d) => format!("  {}  —  {}", e.name, d),
                    None => format!("  {}", e.name),
                };
                ListItem::new(label)
            })
            .collect();

        let list = List::new(items)
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

        let center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Percentage(70),
                Constraint::Fill(1),
            ])
            .split(chunks[2])[1];

        let mut state = ListState::default();
        state.select(Some(app.engagement_selected));
        f.render_stateful_widget(list, center, &mut state);
    }

    // Dica
    let hint = Paragraph::new("↑↓ navegar  •  Enter abrir  •  D deletar  •  Esc voltar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);

    crate::ui::draw_confirm_modal(f, app);
}
