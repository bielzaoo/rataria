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

    let sub_name = app
        .current_subdomain
        .as_ref()
        .map(|s| s.subdomain.as_str())
        .unwrap_or("?");
    let title = Paragraph::new(format!("── Technologies — {} ──", sub_name))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if app.creating_technology {
        draw_form(f, app, chunks[2]);
    } else {
        draw_list(f, app, chunks[2]);
    }

    let hint = if app.creating_technology {
        "Tab alternar campo  •  Enter confirmar  •  Esc cancelar"
    } else {
        "N novo  •  D deletar  •  Esc voltar"
    };
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, chunks[3]);
}

fn draw_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.technologies.is_empty() {
        let empty = Paragraph::new("Nenhuma tecnologia. Pressione N para adicionar.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .technologies
        .iter()
        .map(|t| {
            let label = match &t.version {
                Some(v) => format!("  {}  v{}", t.name, v),
                None => format!("  {}", t.name),
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
            Constraint::Percentage(50),
            Constraint::Fill(1),
        ])
        .split(area)[1];

    let mut state = ListState::default();
    state.select(Some(app.technology_selected));
    f.render_stateful_widget(list, center, &mut state);
}

fn draw_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
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

    let name_active = app.form_field == crate::app::FormField::Name;
    let name_input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Nome (ex: WordPress) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if name_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(name_input, center(chunks[1], 44));

    let ver_active = app.form_field == crate::app::FormField::Description;
    let ver_input = Paragraph::new(app.form_version.as_str())
        .block(
            Block::default()
                .title(" Versão (opcional) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if ver_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(ver_input, center(chunks[3], 44));

    if let Some(err) = &app.form_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, center(chunks[4], 44));
    }
}
