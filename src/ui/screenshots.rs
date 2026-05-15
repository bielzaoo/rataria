use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
            Constraint::Fill(1),   // conteúdo
            Constraint::Length(1), // dica
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
        draw_split(f, app, chunks[2]);
    }

    // Erro/aviso
    if let Some(err) = &app.form_error {
        let color = if err.starts_with('✓') {
            Color::Green
        } else {
            Color::Red
        };
        let err_widget = Paragraph::new(err.as_str())
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        f.render_widget(err_widget, chunks[1]);
    }

    let hint = if app.creating_screenshot {
        "Enter confirmar  •  Esc cancelar"
    } else {
        "N novo  •  Enter preview fullscreen  •  O abrir externo  •  D deletar  •  Esc voltar"
    };
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, chunks[3]);

    crate::ui::draw_confirm_modal(f, app);
}

fn draw_split(f: &mut Frame, app: &App, area: Rect) {
    // Divide em lista (40%) e preview (60%)
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_list(f, app, split[0]);
    draw_preview_placeholder(f, app, split[1]);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    if app.screenshots.is_empty() {
        let empty = Paragraph::new("Nenhuma screenshot.\nPressione N para adicionar.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .screenshots
        .iter()
        .map(|s| {
            let exists = std::path::Path::new(&s.file_path).exists();
            let icon = if exists { "✓" } else { "✗" };
            let color = if exists { Color::Green } else { Color::Red };
            // Mostra apenas o nome do arquivo, não o path completo
            let filename = std::path::Path::new(&s.file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&s.file_path);
            ListItem::new(format!(" {} {}", icon, filename)).style(Style::default().fg(color))
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

    let mut state = ListState::default();
    state.select(Some(app.screenshot_selected));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_preview_placeholder(f: &mut Frame, app: &App, area: Rect) {
    // Placeholder visual — a imagem real é renderizada pelo kitten
    // fora do loop de draw via show_kitty_inline
    let selected = app.screenshots.get(app.screenshot_selected);

    let content = match selected {
        None => "Nenhuma screenshot selecionada.".to_string(),
        Some(shot) => {
            let exists = std::path::Path::new(&shot.file_path).exists();
            if exists {
                if crate::ui::image_preview::is_kitty_supported() {
                    format!("  {}\n\n  Carregando preview...", shot.file_path)
                } else {
                    format!(
                        "  {}\n\n  Terminal não suporta preview.\n  Use O para abrir externamente.",
                        shot.file_path
                    )
                }
            } else {
                format!("  {}\n\n  ✗ Arquivo não encontrado.", shot.file_path)
            }
        }
    };

    let preview = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Preview ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(preview, area);
}

fn draw_form(f: &mut Frame, app: &App, area: Rect) {
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
                .title(" Caminho da screenshot ")
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
