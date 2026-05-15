use crate::app::App;
use crate::db::models::SubdomainStatus;
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
            Constraint::Length(1), // título
            Constraint::Length(1), // filtro ativo
            Constraint::Fill(1),   // conteúdo
            Constraint::Length(1), // dica
        ])
        .split(area);

    // Título
    let target_name = app
        .current_target
        .as_ref()
        .map(|t| t.domain.as_str())
        .unwrap_or("?");

    let title = Paragraph::new(format!("── Subdomains — {} ──", target_name))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Filtro ativo
    let filter_text = match &app.subdomain_filter {
        None => "  Filtro: todos".to_string(),
        Some(s) => format!("  Filtro: {}", s.as_str()),
    };
    let filter_widget = Paragraph::new(filter_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(filter_widget, chunks[1]);

    if app.creating_subdomain {
        draw_create_form(f, app, chunks[2]);
    } else if app.editing_notes {
        draw_edit_notes(f, app, chunks[2]);
    } else if app.editing_subdomain {
        draw_edit_form(f, app, chunks[2]);
    } else {
        draw_table(f, app, chunks[2]);
    }

    // Dica
    let hint = if app.creating_subdomain {
        "Enter confirmar  •  Esc cancelar"
    } else if app.editing_notes {
        "Enter salvar  •  Esc cancelar"
    } else if app.editing_subdomain {
        "Tab alternar campo  •  Enter salvar  •  Esc cancelar"
    } else {
        "N novo  •  E editar  •  S status  •  O notas  •  D deletar  •  F filtro  •  Esc voltar"
    };

    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, chunks[3]);

    crate::ui::draw_confirm_modal(f, app);
}

fn status_color(status: &SubdomainStatus) -> Color {
    match status {
        SubdomainStatus::NotVisited => Color::DarkGray,
        SubdomainStatus::InProgress => Color::Cyan,
        SubdomainStatus::Reviewed => Color::Blue,
        SubdomainStatus::Vulnerable => Color::Red,
        SubdomainStatus::FalsePositive => Color::Yellow,
    }
}

fn draw_table(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let filtered = app.subdomains_filtered();

    if filtered.is_empty() {
        let msg = if app.subdomain_filter.is_some() {
            "Nenhum subdomain com esse filtro."
        } else {
            "Nenhum subdomain. Pressione N para adicionar."
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Subdomain").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Status").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Code").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Title").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Notas").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = filtered
        .iter()
        .map(|s| {
            let status_cell =
                Cell::from(s.status.as_str()).style(Style::default().fg(status_color(&s.status)));
            let code = s
                .status_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            let title = s.title.as_deref().unwrap_or("-");
            let notes = s.notes.as_deref().unwrap_or("-");

            Row::new(vec![
                Cell::from(s.subdomain.as_str()),
                status_cell,
                Cell::from(code),
                Cell::from(title),
                Cell::from(notes),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(13),
            Constraint::Percentage(7),
            Constraint::Percentage(20),
            Constraint::Fill(1),
        ],
    )
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

    // Mapeia o índice selecionado para o índice filtrado
    let selected_in_filtered = filtered
        .iter()
        .position(|s| Some(s.id.as_str()) == app.selected_subdomain().map(|x| x.id.as_str()));

    let mut state = TableState::default();
    state.select(selected_in_filtered);
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_create_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
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
            Constraint::Length(50),
            Constraint::Fill(1),
        ])
        .split(chunks[1])[1];

    let input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Subdomain (ex: api.empresa.com) ")
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
                Constraint::Length(50),
                Constraint::Fill(1),
            ])
            .split(chunks[2])[1];

        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_center);
    }
}

pub fn draw_edit_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use crate::app::FormField;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3), // nome
            Constraint::Length(1), // espaço
            Constraint::Length(3), // status code
            Constraint::Length(1), // espaço
            Constraint::Length(3), // title
            Constraint::Length(1), // erro
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

    // Campo nome
    let name_active = app.form_field == FormField::Name;
    let name_input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Subdomain ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if name_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(name_input, center(chunks[1], 54));

    // Campo status code
    let code_active = app.form_field == FormField::StatusCode;
    let code_input = Paragraph::new(app.form_status_code.as_str())
        .block(
            Block::default()
                .title(" Status Code (ex: 200) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if code_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(code_input, center(chunks[3], 54));

    // Campo title
    let title_active = app.form_field == FormField::Title;
    let title_input = Paragraph::new(app.form_title.as_str())
        .block(
            Block::default()
                .title(" Title ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if title_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(title_input, center(chunks[5], 54));

    // Erro
    if let Some(err) = &app.form_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, center(chunks[6], 54));
    }
}

fn draw_edit_notes(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let sub_name = app
        .selected_subdomain()
        .map(|s| s.subdomain.as_str())
        .unwrap_or("?");

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
            Constraint::Length(60),
            Constraint::Fill(1),
        ])
        .split(chunks[1])[1];

    let input = Paragraph::new(app.form_notes.as_str())
        .block(
            Block::default()
                .title(format!(" Notas — {} ", sub_name))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(input, center);
}
