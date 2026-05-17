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

    let target_name = app
        .current_target
        .as_ref()
        .map(|t| t.domain.as_str())
        .unwrap_or("?");
    let title = Paragraph::new(format!("── IPs — {} ──", target_name))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if app.creating_ip {
        draw_form(f, app, chunks[2], " IP (ex: 192.168.1.1) ");
    } else if app.editing_ip {
        draw_edit_form(f, app, chunks[2]);
    } else {
        draw_list(f, app, chunks[2]);
    }

    let hint = if app.creating_ip {
        "Enter confirmar  •  Esc cancelar"
    } else if app.editing_ip {
        "Enter salvar  •  Esc cancelar"
    } else {
        "N novo  •  E editar  •  D deletar  •  Esc voltar"
    };

    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, chunks[3]);

    crate::ui::draw_confirm_modal(f, app);
}

fn draw_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.ips.is_empty() {
        let empty = Paragraph::new("Nenhum IP. Pressione N para adicionar.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .ips
        .iter()
        .map(|i| ListItem::new(format!("  {}", i.ip)))
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
    state.select(Some(app.ip_selected));
    f.render_stateful_widget(list, center, &mut state);
}

fn draw_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect, field_title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(area);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(44),
            Constraint::Fill(1),
        ])
        .split(chunks[1])[1];

    let input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(field_title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(input, center);

    if let Some(err) = &app.form_error {
        let err_center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(44),
                Constraint::Fill(1),
            ])
            .split(chunks[2])[1];
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_center);
    }
}

fn draw_edit_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(area);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(44),
            Constraint::Fill(1),
        ])
        .split(chunks[1])[1];

    let input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Editar IP ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(input, center);

    if let Some(err) = &app.form_error {
        let err_center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(44),
                Constraint::Fill(1),
            ])
            .split(chunks[2])[1];
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_center);
    }
}
