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
    let title = Paragraph::new(format!("── ASNs — {} ──", target_name))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if app.creating_asn {
        draw_asn_form(f, app, chunks[2]);
    } else if app.editing_asn {
        draw_edit_form(f, app, chunks[2]);
    } else {
        draw_list(f, app, chunks[2]);
    }

    let hint = if app.creating_asn {
        "Tab alternar campo  •  Enter confirmar  •  Esc cancelar"
    } else if app.editing_asn {
        "Tab alternar campo  •  Enter salvar  •  Esc cancelar"
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
    if app.asns.is_empty() {
        let empty = Paragraph::new("Nenhum ASN. Pressione N para adicionar.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .asns
        .iter()
        .map(|a| {
            let label = match &a.org {
                Some(org) => format!("  {}  —  {}", a.asn, org),
                None => format!("  {}", a.asn),
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
            Constraint::Percentage(60),
            Constraint::Fill(1),
        ])
        .split(area)[1];

    let mut state = ListState::default();
    state.select(Some(app.asn_selected));
    f.render_stateful_widget(list, center, &mut state);
}

fn draw_asn_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
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

    // Campo ASN (ativo quando form_field == Name)
    let asn_active = app.form_field == crate::app::FormField::Name;
    let asn_input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" ASN (ex: AS12345) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if asn_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(asn_input, center(chunks[1], 44));

    // Campo Org (ativo quando form_field == Description)
    let org_active = app.form_field == crate::app::FormField::Description;
    let org_input = Paragraph::new(app.form_org.as_str())
        .block(
            Block::default()
                .title(" Organização (opcional) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if org_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(org_input, center(chunks[3], 44));

    if let Some(err) = &app.form_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, center(chunks[4], 44));
    }
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

    let asn_active = app.form_field == FormField::Name;
    let asn_input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" ASN (ex: AS12345) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if asn_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(asn_input, center(chunks[1], 44));

    let org_active = app.form_field == FormField::Description;
    let org_input = Paragraph::new(app.form_org.as_str())
        .block(
            Block::default()
                .title(" Organização (opcional) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if org_active {
                    Color::Yellow
                } else {
                    Color::DarkGray
                })),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(org_input, center(chunks[3], 44));

    if let Some(err) = &app.form_error {
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, center(chunks[4], 44));
    }
}
