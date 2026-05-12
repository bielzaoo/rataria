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
        "Subdomains" => format_subdomains(app),
        "IPs" => format_ips(app),
        "ASNs" => format_asns(app),
        "URLs" => format_urls(app),
        "Technologies" => format_technologies(app),
        "Screenshots" => format_screenshots(app),
        _ => String::new(),
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

fn format_subdomains(app: &App) -> String {
    if app.subdomains.is_empty() {
        return "  Nenhum subdomain cadastrado.".to_string();
    }
    app.subdomains
        .iter()
        .map(|s| {
            let code = s
                .status_code
                .map(|c| format!(" [{}]", c))
                .unwrap_or_default();
            format!("  • {}{} — {}", s.subdomain, code, s.status.as_str())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_ips(app: &App) -> String {
    if app.ips.is_empty() {
        return "  Nenhum IP cadastrado.".to_string();
    }
    app.ips
        .iter()
        .map(|i| format!("  • {}", i.ip))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_asns(app: &App) -> String {
    if app.asns.is_empty() {
        return "  Nenhum ASN cadastrado.".to_string();
    }
    app.asns
        .iter()
        .map(|a| match &a.org {
            Some(org) => format!("  • {} — {}", a.asn, org),
            None => format!("  • {}", a.asn),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_urls(app: &App) -> String {
    if app.urls.is_empty() {
        return "  Nenhum URL cadastrado.".to_string();
    }
    app.urls
        .iter()
        .map(|u| format!("  • [{}] {}", u.url_type.as_str(), u.url))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_technologies(app: &App) -> String {
    if app.technologies.is_empty() {
        return "  Nenhuma tecnologia cadastrada.".to_string();
    }
    app.technologies
        .iter()
        .map(|t| match &t.version {
            Some(v) => format!("  • {} v{}", t.name, v),
            None => format!("  • {}", t.name),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_screenshots(app: &App) -> String {
    if app.screenshots.is_empty() {
        return "  Nenhuma screenshot cadastrada.".to_string();
    }
    app.screenshots
        .iter()
        .map(|s| {
            let exists = std::path::Path::new(&s.file_path).exists();
            let icon = if exists { "✓" } else { "✗" };
            format!("  {} {}", icon, s.file_path)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
