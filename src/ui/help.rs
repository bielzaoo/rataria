use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
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
            Constraint::Fill(1),   // tabela
            Constraint::Length(1), // dica
        ])
        .split(area);

    let title = Paragraph::new("── Atalhos de Teclado ──")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Contexto atual
    let context = context_name(app);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(70),
            Constraint::Fill(1),
        ])
        .split(chunks[2])[1];

    let shortcuts = shortcuts_for_context(context);

    let header = Row::new(vec![
        Cell::from("Tecla").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Ação").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Contexto").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = shortcuts
        .iter()
        .map(|(key, action, ctx)| {
            let ctx_color = if *ctx == "Global" {
                Color::DarkGray
            } else {
                Color::Cyan
            };
            Row::new(vec![
                Cell::from(*key).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(*action),
                Cell::from(*ctx).style(Style::default().fg(ctx_color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Fill(1),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Contexto atual: {} ", context))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(table, center);

    let hint = Paragraph::new("? ou Esc para fechar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);
}

fn context_name(app: &App) -> &'static str {
    use crate::app::Screen;
    match app.help_previous_screen.as_ref().unwrap_or(&Screen::Home) {
        Screen::Home => "Home",
        Screen::ListEngagements => "Engagements",
        Screen::CreateEngagement => "Criar Engagement",
        Screen::Dashboard => "Dashboard",
        Screen::Targets => "Targets",
        Screen::TargetMenu => "Menu Target",
        Screen::Subdomains => "Subdomains",
        Screen::SubdomainMenu => "Menu Subdomain",
        Screen::IPs => "IPs",
        Screen::ASNs => "ASNs",
        Screen::URLs => "URLs",
        Screen::Technologies => "Technologies",
        Screen::Screenshots => "Screenshots",
        Screen::Import => "Import",
        Screen::Password | Screen::Help => "Home",
    }
}

fn shortcuts_for_context(context: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    let mut shortcuts = vec![
        // Globais
        ("?", "Abrir esta tela de ajuda", "Global"),
        ("Esc", "Voltar / Cancelar", "Global"),
        ("q", "Sair (no Home)", "Global"),
    ];

    match context {
        "Home" => {
            shortcuts.extend(vec![
                ("↑ ↓ / j k", "Navegar menu", "Home"),
                ("Enter", "Selecionar opção", "Home"),
            ]);
        }
        "Engagements" => {
            shortcuts.extend(vec![
                ("↑ ↓ / j k", "Navegar lista", "Engagements"),
                ("Enter", "Abrir engagement", "Engagements"),
                ("D", "Deletar engagement", "Engagements"),
            ]);
        }
        "Dashboard" => {
            shortcuts.extend(vec![
                ("↑ ↓ / j k", "Navegar seções", "Dashboard"),
                ("Enter", "Abrir seção", "Dashboard"),
                ("E", "Exportar relatório MD", "Dashboard"),
            ]);
        }
        "Targets" => {
            shortcuts.extend(vec![
                ("↑ ↓ / j k", "Navegar lista", "Targets"),
                ("Enter", "Abrir target", "Targets"),
                ("N", "Novo target", "Targets"),
                ("D", "Deletar target", "Targets"),
            ]);
        }
        "Subdomains" => {
            shortcuts.extend(vec![
                ("↑ ↓ / j k", "Navegar lista", "Subdomains"),
                ("Enter", "Abrir subdomain", "Subdomains"),
                ("N", "Novo subdomain", "Subdomains"),
                ("S", "Ciclar status", "Subdomains"),
                ("O", "Editar notas", "Subdomains"),
                ("F", "Ciclar filtro de status", "Subdomains"),
                ("D", "Deletar subdomain", "Subdomains"),
            ]);
        }
        "IPs" | "ASNs" | "URLs" | "Technologies" | "Screenshots" => {
            shortcuts.extend(vec![
                ("↑ ↓ / j k", "Navegar lista", "Lista"),
                ("N", "Novo item", "Lista"),
                ("D", "Deletar item", "Lista"),
            ]);
            if context == "URLs" {
                shortcuts.push(("Tab", "Ciclar tipo de URL", "URLs"));
            }
            if context == "Screenshots" {
                shortcuts.push(("Enter", "Preview da imagem", "Screenshots"));
                shortcuts.push(("O", "Abrir com visualizador", "Screenshots"));
            }
        }
        "Import" => {
            shortcuts.extend(vec![
                ("Tab", "Alternar campo", "Import"),
                ("Enter", "Importar arquivo", "Import"),
            ]);
        }
        _ => {}
    }

    shortcuts
}
