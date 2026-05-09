use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
    let title = Paragraph::new(format!("── Screenshots — {} ──", sub_name))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if app.creating_screenshot {
        draw_form(f, app, chunks[2]);
    } else {
        draw_list(f, app, chunks[2]);
    }

    let hint = if app.creating_screenshot {
        "Enter confirmar  •  Esc cancelar"
    } else {
        "N novo  •  D deletar  •  Esc voltar"
    };
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, chunks[3]);
}

fn draw_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.screenshots.is_empty() {
        let empty = Paragraph::new("Nenhuma screenshot. Pressione N para adicionar o caminho.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .screenshots
        .iter()
        .map(|s| ListItem::new(format!("  {}", s.file_path)))
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
        );

    f.render_widget(list, area);
}

fn draw_form(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
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
            Constraint::Percentage(70),
            Constraint::Fill(1),
        ])
        .split(chunks[1])[1];

    let input = Paragraph::new(app.form_name.as_str())
        .block(
            Block::default()
                .title(" Caminho da screenshot (ex: /home/user/screenshots/api.png) ")
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
            .split(chunks[2])[1];
        let error_msg = Paragraph::new(format!("✗ {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_msg, err_center);
    }
}
