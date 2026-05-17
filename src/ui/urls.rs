use crate::app::App;
use crate::db::models::UrlType;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
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
    let title = Paragraph::new(format!("── URLs — {} ──", sub_name))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if app.creating_url {
        draw_form(f, app, chunks[2]);
    } else if app.editing_url {
        draw_edit_form(f, app, chunks[2]);
    } else {
        draw_table(f, app, chunks[2]);
    }

    let hint = if app.creating_url {
        "Tab ciclar tipo  •  Enter confirmar  •  Esc cancelar"
    } else if app.editing_url {
        "Tab ciclar tipo  •  Enter salvar  •  Esc cancelar"
    } else {
        "N novo  •  E editar  •  D deletar  •  Esc voltar"
    };

    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, chunks[3]);

    crate::ui::draw_confirm_modal(f, app);
}

fn url_type_color(t: &UrlType) -> Color {
    match t {
        UrlType::Parameter => Color::Cyan,
        UrlType::JavaScript => Color::Yellow,
        UrlType::Endpoint => Color::Green,
        UrlType::Other => Color::DarkGray,
    }
}

fn draw_table(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.urls.is_empty() {
        let empty = Paragraph::new("Nenhuma URL. Pressione N para adicionar.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("URL").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Tipo").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .urls
        .iter()
        .map(|u| {
            Row::new(vec![
                Cell::from(u.url.as_str()),
                Cell::from(u.url_type.as_str())
                    .style(Style::default().fg(url_type_color(&u.url_type))),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Fill(1), Constraint::Length(12)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .row_highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    state.select(Some(app.url_selected));
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(area);

    // Tipo selecionado
    let type_label = Paragraph::new(format!(
        "Tipo: {}  (Tab para mudar)",
        app.form_url_type.as_str()
    ))
    .style(Style::default().fg(url_type_color(&app.form_url_type)))
    .alignment(Alignment::Center);
    f.render_widget(type_label, chunks[1]);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(70),
            Constraint::Fill(1),
        ])
        .split(chunks[2])[1];

    let input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" URL ")
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
                Constraint::Percentage(70),
                Constraint::Fill(1),
            ])
            .split(chunks[3])[1];
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_center);
    }
}

fn draw_edit_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use crate::db::models::UrlType;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(area);

    let type_label = Paragraph::new(format!(
        "Tipo: {}  (Tab para mudar)",
        app.form_url_type.as_str()
    ))
    .style(Style::default().fg(url_type_color(&app.form_url_type)))
    .alignment(Alignment::Center);
    f.render_widget(type_label, chunks[1]);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(70),
            Constraint::Fill(1),
        ])
        .split(chunks[2])[1];

    let input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Editar URL ")
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
                Constraint::Percentage(70),
                Constraint::Fill(1),
            ])
            .split(chunks[3])[1];
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_center);
    }
}
