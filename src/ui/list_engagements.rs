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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    let title = Paragraph::new("── Engagements ──")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if app.editing_engagement {
        draw_edit_form(f, app, chunks[2]);
        return;
    }

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

    let hint =
        Paragraph::new("↑↓ navegar  •  Enter abrir  •  E editar  •  D deletar  •  Esc voltar")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);
}

fn draw_edit_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use crate::app::FormField;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    let center = |a, w| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(w),
                Constraint::Fill(1),
            ])
            .split(a)[1]
    };

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
    f.render_widget(name_input, center(chunks[1], 50));

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
    f.render_widget(desc_input, center(chunks[3], 50));

    if let Some(err) = &app.form_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, center(chunks[4], 50));
    }

    let hint = Paragraph::new("Tab alternar campo  •  Enter salvar  •  Esc cancelar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[6]);
}
