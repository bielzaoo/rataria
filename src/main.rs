use db::models::SubdomainStatus;
mod app;
mod auth;
mod db;
mod error;
mod export;
mod import;
mod ui;
use app::{App, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use db::Database;
use error::Result;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    let db_path = Database::default_path();

    loop {
        // Verifica timeout de sessão
        if app.screen != Screen::Password && app.db.is_some() && app.is_session_expired() {
            app.lock_session();
        }

        // draw
        terminal
            .draw(|f| match app.screen {
                Screen::Password => ui::password::draw(f, app),
                Screen::Home => ui::home::draw(f, app),
                Screen::CreateEngagement => ui::create_engagement::draw(f, app),
                Screen::ListEngagements => ui::list_engagements::draw(f, app),
                Screen::Dashboard => ui::dashboard::draw(f, app),
                Screen::Targets => ui::targets::draw(f, app),
                Screen::Subdomains => ui::subdomains::draw(f, app),
                Screen::TargetMenu => ui::target_menu::draw(f, app),
                Screen::IPs => ui::ips::draw(f, app),
                Screen::ASNs => ui::asns::draw(f, app),
                Screen::SubdomainMenu => ui::subdomain_menu::draw(f, app),
                Screen::URLs => ui::urls::draw(f, app),
                Screen::Technologies => ui::technologies::draw(f, app),
                Screen::Screenshots => ui::screenshots::draw(f, app),
                Screen::Import => ui::import::draw(f, app),
                Screen::Help => ui::help::draw(f, app),
            })
            .map_err(|e| {
                error::RatariaError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        if app.screen == Screen::Screenshots && !app.creating_screenshot {
            let current = app
                .screenshots
                .get(app.screenshot_selected)
                .map(|s| s.file_path.clone());

            let needs_render =
                app.screenshot_last_rendered != Some(app.screenshot_selected) && current.is_some();

            if needs_render {
                if let Ok(size) = terminal.size() {
                    let col = (size.width as f32 * 0.40) as u16 + 1;
                    let row = 3u16;
                    let w = (size.width as f32 * 0.58) as u16;
                    let h = size.height.saturating_sub(5);

                    if let Some(path) = current {
                        if ui::image_preview::is_valid_image(&path)
                            && ui::image_preview::is_kitty_supported()
                        {
                            terminal.clear().map_err(|e| {
                                error::RatariaError::IoError(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    e.to_string(),
                                ))
                            })?;
                            terminal
                                .draw(|f| ui::screenshots::draw(f, app))
                                .map_err(|e| {
                                    error::RatariaError::IoError(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        e.to_string(),
                                    ))
                                })?;
                            ui::image_preview::show_kitty_inline(&path, col, row, w, h).ok();
                            app.screenshot_last_rendered = Some(app.screenshot_selected);
                        }
                    }
                }
            }
        } else {
            if app.screenshot_last_rendered.is_some() {
                ui::image_preview::clear_kitty_inline().ok();
                app.screenshot_last_rendered = None;
            }
        }

        if event::poll(std::time::Duration::from_millis(16))
            .map_err(error::RatariaError::IoError)?
        {
            if let Event::Key(key) = event::read().map_err(error::RatariaError::IoError)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                app.update_activity(); // ← adiciona aqui

                if key.code == KeyCode::Char('?')
                    && app.screen != Screen::Password
                    && app.screen != Screen::Help
                {
                    app.open_help();
                    continue;
                }

                // eventos
                match app.screen {
                    Screen::Password => handle_password(key.code, app, &db_path)?,
                    Screen::Home => handle_home(key.code, app),
                    Screen::CreateEngagement => handle_create_engagement(key.code, app)?,
                    Screen::ListEngagements => handle_list_engagements(key.code, app)?,
                    Screen::Dashboard => handle_dashboard(key.code, app)?,
                    Screen::Targets => handle_targets(key.code, app)?,
                    Screen::Subdomains => handle_subdomains(key.code, app)?,
                    Screen::TargetMenu => handle_target_menu(key.code, app)?,
                    Screen::IPs => handle_ips(key.code, app)?,
                    Screen::ASNs => handle_asns(key.code, app)?,
                    Screen::SubdomainMenu => handle_subdomain_menu(key.code, app)?,
                    Screen::URLs => handle_urls(key.code, app)?,
                    Screen::Technologies => handle_technologies(key.code, app)?,
                    Screen::Screenshots => handle_screenshots(key.code, app, &mut *terminal)?,
                    Screen::Import => handle_import(key.code, app)?,
                    Screen::Help => {
                        if let crossterm::event::KeyCode::Char('?')
                        | crossterm::event::KeyCode::Esc = key.code
                        {
                            app.close_help();
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_password(key: KeyCode, app: &mut App, db_path: &std::path::PathBuf) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Enter => {
            if app.password_input.is_empty() {
                app.password_error = Some("Digite uma senha".to_string());
                return Ok(());
            }

            match Database::open(db_path, &app.password_input) {
                Ok(db) => {
                    app.db = Some(db);
                    app.password_input.clear();
                    app.password_error = None;
                    app.screen = Screen::Home;
                    app.update_activity(); // garante que o timer começa do login
                }
                Err(_) => {
                    app.password_error = Some("Senha incorreta".to_string());
                    app.password_input.clear();
                }
            }
        }
        KeyCode::Backspace => {
            app.password_input.pop();
            app.password_error = None;
        }
        KeyCode::Char(c) => {
            app.password_input.push(c);
            app.password_error = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_home(key: KeyCode, app: &mut App) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.home_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.home_previous();
        }
        KeyCode::Enter => match app.home_selected {
            0 => {
                if let Some(db) = &app.db {
                    app.engagements = db::queries::list_engagements(db).unwrap_or_default();
                    app.engagement_selected = 0;
                    app.screen = Screen::ListEngagements;
                }
            }
            1 => {
                app.reset_form();
                app.screen = Screen::CreateEngagement;
            }
            2 => {
                app.reset_import_form();
                app.screen = Screen::Import;
            }
            3 => app.should_quit = true,
            _ => {}
        },
        _ => {}
    }
}

fn handle_create_engagement(key: KeyCode, app: &mut App) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.reset_form();
            app.screen = Screen::Home;
        }
        KeyCode::Tab => {
            app.form_next_field();
        }
        KeyCode::Enter => {
            if app.form_name.trim().is_empty() {
                app.form_error = Some("Nome é obrigatório".to_string());
                return Ok(());
            }

            let db = app.db.as_ref().unwrap();
            let new = db::models::NewEngagement {
                name: app.form_name.trim().to_string(),
                description: if app.form_description.trim().is_empty() {
                    None
                } else {
                    Some(app.form_description.trim().to_string())
                },
            };

            match db::queries::create_engagement(db, new) {
                Ok(_) => {
                    app.reset_form();
                    app.screen = Screen::Home;
                }
                Err(_) => {
                    app.form_error = Some("Já existe um engagement com esse nome".to_string());
                }
            }
        }
        KeyCode::Backspace => {
            match app.form_field {
                app::FormField::Name => {
                    app.form_name.pop();
                }
                app::FormField::Description => {
                    app.form_description.pop();
                }
                _ => {}
            }
            app.form_error = None;
        }
        KeyCode::Char(c) => {
            match app.form_field {
                app::FormField::Name => app.form_name.push(c),
                app::FormField::Description => app.form_description.push(c),
                _ => {}
            }
            app.form_error = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_list_engagements(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(eng) = app.selected_engagement().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_engagement(db, &eng.id).ok();
                        app.engagements = db::queries::list_engagements(db).unwrap_or_default();
                        if app.engagement_selected >= app.engagements.len()
                            && !app.engagements.is_empty()
                        {
                            app.engagement_selected = app.engagements.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_engagement {
        match key {
            KeyCode::Esc => {
                app.editing_engagement = false;
                app.reset_form();
            }
            KeyCode::Tab => {
                app.form_next_field();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Nome é obrigatório".to_string());
                    return Ok(());
                }
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };
                match db::queries::update_engagement(
                    app.db.as_ref().unwrap(),
                    &id,
                    &app.form_name.trim(),
                    if app.form_description.trim().is_empty() {
                        None
                    } else {
                        Some(app.form_description.trim())
                    },
                ) {
                    Ok(_) => {
                        if let Some(db) = &app.db {
                            app.engagements = db::queries::list_engagements(db).unwrap_or_default();
                        }
                        app.editing_engagement = false;
                        app.reset_form();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                match app.form_field {
                    app::FormField::Name => {
                        app.form_name.pop();
                    }
                    app::FormField::Description => {
                        app.form_description.pop();
                    }
                    _ => {}
                }
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                match app.form_field {
                    app::FormField::Name => app.form_name.push(c),
                    app::FormField::Description => app.form_description.push(c),
                    _ => {}
                }
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    match key {
        KeyCode::Esc => {
            app.screen = Screen::Home;
        }
        KeyCode::Down | KeyCode::Char('j') => app.engagements_next(),
        KeyCode::Up | KeyCode::Char('k') => app.engagements_previous(),
        KeyCode::Char('d') => {
            if let Some(eng) = app.selected_engagement() {
                app.ask_confirm_delete(&eng.name.clone());
            }
        }
        KeyCode::Enter => {
            if let Some(eng) = app.selected_engagement().cloned() {
                let eng_id = eng.id.clone();
                app.current_engagement = Some(eng);
                app.dashboard_selected = 0;

                if let Some(db) = &app.db {
                    app.targets = db::queries::list_targets(db, &eng_id).unwrap_or_default();

                    // Carrega dados do primeiro target para preview no dashboard
                    if let Some(first_target) = app.targets.first() {
                        let tid = first_target.id.clone();
                        app.ips = db::queries::list_ips(db, &tid).unwrap_or_default();
                        app.asns = db::queries::list_asns(db, &tid).unwrap_or_default();
                        app.subdomains = db::queries::list_subdomains(db, &tid).unwrap_or_default();

                        // Carrega dados do primeiro subdomain para preview
                        if let Some(first_sub) = app.subdomains.first() {
                            let sid = first_sub.id.clone();
                            app.urls = db::queries::list_urls(db, &sid).unwrap_or_default();
                            app.technologies =
                                db::queries::list_technologies(db, &sid).unwrap_or_default();
                            app.screenshots = db::queries::list_screenshots_by_subdomain(db, &sid)
                                .unwrap_or_default();
                        }
                    }
                }

                app.target_selected = 0;
                app.screen = Screen::Dashboard;
            }
        }
        KeyCode::Char('e') => {
            if let Some(eng) = app.selected_engagement().cloned() {
                app.form_name = eng.name.clone();
                app.form_description = eng.description.clone().unwrap_or_default();
                app.form_field = app::FormField::Name;
                app.form_error = None;
                app.editing_item_id = Some(eng.id.clone());
                app.editing_engagement = true;
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_dashboard(key: KeyCode, app: &mut App) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.screen = Screen::ListEngagements;
            if let Some(db) = &app.db {
                app.engagements = db::queries::list_engagements(db).unwrap_or_default();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => app.dashboard_next(),
        KeyCode::Up | KeyCode::Char('k') => app.dashboard_previous(),
        KeyCode::Char('e') => {
            if let Some(eng) = &app.current_engagement {
                let eng_id = eng.id.clone();
                let eng_name = eng.name.clone();

                if let Some(db) = &app.db {
                    match export::export_engagement_markdown(db, &eng_id) {
                        Ok(content) => {
                            let path = export::default_report_path(&eng_name);
                            match export::save_report(&content, &path) {
                                Ok(_) => {
                                    app.form_error = Some(format!("✓ Exportado: {}", path));
                                }
                                Err(e) => {
                                    app.form_error = Some(format!("✗ Erro ao salvar: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            app.form_error = Some(format!("✗ Erro ao exportar: {}", e));
                        }
                    }
                }
            }
        }
        KeyCode::Enter => {
            // Define a intenção baseada no item selecionado
            app.dashboard_intent = Some(match app.dashboard_selected {
                0 => app::DashboardIntent::Targets,
                1 => app::DashboardIntent::Subdomains,
                2 => app::DashboardIntent::IPs,
                3 => app::DashboardIntent::ASNs,
                4 => app::DashboardIntent::URLs,
                5 => app::DashboardIntent::Technologies,
                6 => app::DashboardIntent::Screenshots,
                _ => app::DashboardIntent::Targets,
            });

            // Sempre vai para Targets primeiro para selecionar
            if let Some(eng) = &app.current_engagement {
                let eng_id = eng.id.clone();
                if let Some(db) = &app.db {
                    app.targets = db::queries::list_targets(db, &eng_id).unwrap_or_default();
                }
            }
            app.target_selected = 0;
            app.creating_target = false;
            app.screen = Screen::Targets;
        }
        _ => {}
    }
    Ok(())
}

fn handle_targets(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(target) = app.selected_target().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_target(db, &target.id).ok();
                        let eng_id = app
                            .current_engagement
                            .as_ref()
                            .map(|e| e.id.clone())
                            .unwrap_or_default();
                        app.targets = db::queries::list_targets(db, &eng_id).unwrap_or_default();
                        if app.target_selected >= app.targets.len() && !app.targets.is_empty() {
                            app.target_selected = app.targets.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_target {
        match key {
            KeyCode::Esc => {
                app.editing_target = false;
                app.reset_form();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Domínio é obrigatório".to_string());
                    return Ok(());
                }
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };
                match db::queries::update_target(
                    app.db.as_ref().unwrap(),
                    &id,
                    db::models::UpdateTarget {
                        domain: app.form_name.trim().to_string(),
                    },
                ) {
                    Ok(_) => {
                        if let Some(eng) = &app.current_engagement {
                            let eid = eng.id.clone();
                            if let Some(db) = &app.db {
                                app.targets =
                                    db::queries::list_targets(db, &eid).unwrap_or_default();
                            }
                        }
                        app.editing_target = false;
                        app.reset_form();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_target {
        match key {
            KeyCode::Esc => {
                app.creating_target = false;
                app.reset_form();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Domínio é obrigatório".to_string());
                    return Ok(());
                }
                let eng_id = match &app.current_engagement {
                    Some(e) => e.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewTarget {
                    engagement_id: eng_id.clone(),
                    domain: app.form_name.trim().to_string(),
                };
                match db::queries::create_target(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_target = false;
                        app.reset_form();
                        if let Some(db) = &app.db {
                            app.targets =
                                db::queries::list_targets(db, &eng_id).unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("Já existe um target com esse domínio".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.dashboard_intent = None;
                app.screen = Screen::Dashboard;
                if let Some(eng) = &app.current_engagement {
                    let eng_id = eng.id.clone();
                    if let Some(db) = &app.db {
                        app.targets = db::queries::list_targets(db, &eng_id).unwrap_or_default();
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => app.targets_next(),
            KeyCode::Up | KeyCode::Char('k') => app.targets_previous(),
            KeyCode::Char('n') => {
                app.reset_form();
                app.creating_target = true;
            }
            KeyCode::Enter => {
                if let Some(target) = app.selected_target().cloned() {
                    // Carrega dados do target selecionado
                    if let Some(db) = &app.db {
                        app.subdomains =
                            db::queries::list_subdomains(db, &target.id).unwrap_or_default();
                        app.ips = db::queries::list_ips(db, &target.id).unwrap_or_default();
                        app.asns = db::queries::list_asns(db, &target.id).unwrap_or_default();
                    }
                    app.current_target = Some(target);

                    // Navega baseado na intenção do Dashboard
                    match &app.dashboard_intent {
                        Some(app::DashboardIntent::Targets) | None => {
                            app.target_menu_selected = 0;
                            app.screen = Screen::TargetMenu;
                        }
                        Some(app::DashboardIntent::Subdomains) => {
                            app.subdomain_selected = 0;
                            app.subdomain_filter = None;
                            app.creating_subdomain = false;
                            app.screen = Screen::Subdomains;
                        }
                        Some(app::DashboardIntent::IPs) => {
                            app.ip_selected = 0;
                            app.creating_ip = false;
                            app.screen = Screen::IPs;
                        }
                        Some(app::DashboardIntent::ASNs) => {
                            app.asn_selected = 0;
                            app.creating_asn = false;
                            app.screen = Screen::ASNs;
                        }
                        Some(app::DashboardIntent::URLs) => {
                            // URLs ficam em SubdomainMenu — vai para Subdomains primeiro
                            app.subdomain_selected = 0;
                            app.subdomain_filter = None;
                            app.creating_subdomain = false;
                            app.screen = Screen::Subdomains;
                        }
                        Some(app::DashboardIntent::Technologies) => {
                            app.subdomain_selected = 0;
                            app.subdomain_filter = None;
                            app.creating_subdomain = false;
                            app.screen = Screen::Subdomains;
                        }
                        Some(app::DashboardIntent::Screenshots) => {
                            app.subdomain_selected = 0;
                            app.subdomain_filter = None;
                            app.creating_subdomain = false;
                            app.screen = Screen::Subdomains;
                        }
                    }
                    app.dashboard_intent = None; // limpa a intenção após usar
                }
            }
            KeyCode::Char('d') => {
                if let Some(target) = app.selected_target() {
                    app.ask_confirm_delete(&target.domain.clone());
                }
            }
            KeyCode::Char('e') => {
                if let Some(target) = app.selected_target().cloned() {
                    app.form_name = target.domain.clone();
                    app.form_error = None;
                    app.editing_item_id = Some(target.id.clone());
                    app.editing_target = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_subdomains(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(sub) = app.selected_subdomain().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_subdomain(db, &sub.id).ok();
                        app.subdomains =
                            db::queries::list_subdomains(db, &sub.target_id).unwrap_or_default();
                        if app.subdomain_selected >= app.subdomains.len()
                            && !app.subdomains.is_empty()
                        {
                            app.subdomain_selected = app.subdomains.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_subdomain {
        match key {
            KeyCode::Esc => {
                app.editing_subdomain = false;
                app.reset_form();
            }
            KeyCode::Tab => {
                app.form_field = match app.form_field {
                    app::FormField::Name => app::FormField::StatusCode,
                    app::FormField::StatusCode => app::FormField::Title,
                    app::FormField::Title => app::FormField::Name,
                    _ => app::FormField::Name,
                };
            }
            KeyCode::Enter => {
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };

                let status_code = if app.form_status_code.trim().is_empty() {
                    None
                } else {
                    match app.form_status_code.trim().parse::<i32>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            app.form_error = Some("Status code deve ser um número".to_string());
                            return Ok(());
                        }
                    }
                };

                let update = db::models::UpdateSubdomain {
                    status: None,
                    notes: None,
                    status_code,
                    title: if app.form_title.trim().is_empty() {
                        None
                    } else {
                        Some(app.form_title.trim().to_string())
                    },
                    subdomain: if app.form_name.trim().is_empty() {
                        None
                    } else {
                        Some(app.form_name.trim().to_string())
                    },
                };

                match db::queries::update_subdomain(app.db.as_ref().unwrap(), &id, update) {
                    Ok(_) => {
                        if let Some(target) = &app.current_target {
                            let tid = target.id.clone();
                            if let Some(db) = &app.db {
                                app.subdomains =
                                    db::queries::list_subdomains(db, &tid).unwrap_or_default();
                            }
                        }
                        app.editing_subdomain = false;
                        app.reset_form();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                match app.form_field {
                    app::FormField::Name => {
                        app.form_name.pop();
                    }
                    app::FormField::StatusCode => {
                        app.form_status_code.pop();
                    }
                    app::FormField::Title => {
                        app.form_title.pop();
                    }
                    _ => {}
                }
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                match app.form_field {
                    app::FormField::Name => app.form_name.push(c),
                    app::FormField::StatusCode => app.form_status_code.push(c),
                    app::FormField::Title => app.form_title.push(c),
                    _ => {}
                }
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_subdomain {
        match key {
            KeyCode::Esc => {
                app.creating_subdomain = false;
                app.reset_form();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Subdomain é obrigatório".to_string());
                    return Ok(());
                }
                let target_id = match &app.current_target {
                    Some(t) => t.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewSubdomain {
                    target_id: target_id.clone(),
                    subdomain: app.form_name.trim().to_string(),
                    status_code: None,
                    title: None,
                };
                match db::queries::create_subdomain(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_subdomain = false;
                        app.reset_form();
                        if let Some(db) = &app.db {
                            app.subdomains =
                                db::queries::list_subdomains(db, &target_id).unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("Subdomain já existe neste target".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
    } else if app.editing_notes {
        match key {
            KeyCode::Esc => {
                app.editing_notes = false;
                app.form_notes.clear();
            }
            KeyCode::Enter => {
                if let Some(sub) = app.selected_subdomain().cloned() {
                    let update = db::models::UpdateSubdomain {
                        status: None,
                        notes: Some(app.form_notes.clone()),
                        status_code: None,
                        title: None,
                        subdomain: None,
                    };
                    if let Some(db) = &app.db {
                        db::queries::update_subdomain(db, &sub.id, update).ok();
                        app.subdomains =
                            db::queries::list_subdomains(db, &sub.target_id).unwrap_or_default();
                    }
                }
                app.editing_notes = false;
                app.form_notes.clear();
            }
            KeyCode::Backspace => {
                app.form_notes.pop();
            }
            KeyCode::Char(c) => {
                app.form_notes.push(c);
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.screen = Screen::TargetMenu;
                app.subdomain_filter = None;
            }
            KeyCode::Down | KeyCode::Char('j') => app.subdomains_next(),
            KeyCode::Up | KeyCode::Char('k') => app.subdomains_previous(),
            KeyCode::Enter => {
                if let Some(sub) = app.selected_subdomain().cloned() {
                    if let Some(db) = &app.db {
                        app.urls = db::queries::list_urls(db, &sub.id).unwrap_or_default();
                        app.technologies =
                            db::queries::list_technologies(db, &sub.id).unwrap_or_default();
                        app.screenshots = db::queries::list_screenshots_by_subdomain(db, &sub.id)
                            .unwrap_or_default();
                    }
                    app.current_subdomain = Some(sub);
                    app.subdomain_menu_selected = 0;
                    app.screen = Screen::SubdomainMenu;
                }
            }
            KeyCode::Char('n') => {
                app.reset_form();
                app.creating_subdomain = true;
            }
            KeyCode::Char('o') => {
                if let Some(sub) = app.selected_subdomain() {
                    app.form_notes = sub.notes.clone().unwrap_or_default();
                    app.editing_notes = true;
                }
            }
            KeyCode::Char('s') => {
                if let Some(sub) = app.selected_subdomain().cloned() {
                    let next_status = match sub.status {
                        SubdomainStatus::NotVisited => SubdomainStatus::InProgress,
                        SubdomainStatus::InProgress => SubdomainStatus::Reviewed,
                        SubdomainStatus::Reviewed => SubdomainStatus::Vulnerable,
                        SubdomainStatus::Vulnerable => SubdomainStatus::FalsePositive,
                        SubdomainStatus::FalsePositive => SubdomainStatus::NotVisited,
                    };
                    let update = db::models::UpdateSubdomain {
                        status: Some(next_status),
                        notes: None,
                        status_code: None,
                        title: None,
                        subdomain: None,
                    };
                    if let Some(db) = &app.db {
                        db::queries::update_subdomain(db, &sub.id, update).ok();
                        app.subdomains =
                            db::queries::list_subdomains(db, &sub.target_id).unwrap_or_default();
                    }
                }
            }
            KeyCode::Char('f') => {
                app.subdomain_filter = match &app.subdomain_filter {
                    None => Some(SubdomainStatus::NotVisited),
                    Some(SubdomainStatus::NotVisited) => Some(SubdomainStatus::InProgress),
                    Some(SubdomainStatus::InProgress) => Some(SubdomainStatus::Reviewed),
                    Some(SubdomainStatus::Reviewed) => Some(SubdomainStatus::Vulnerable),
                    Some(SubdomainStatus::Vulnerable) => Some(SubdomainStatus::FalsePositive),
                    Some(SubdomainStatus::FalsePositive) => None,
                };
                app.subdomain_selected = 0;
            }
            KeyCode::Char('d') => {
                if let Some(sub) = app.selected_subdomain() {
                    app.ask_confirm_delete(&sub.subdomain.clone());
                }
            }
            KeyCode::Char('e') => {
                if let Some(sub) = app.selected_subdomain().cloned() {
                    app.form_name = sub.subdomain.clone();
                    app.form_status_code =
                        sub.status_code.map(|c| c.to_string()).unwrap_or_default();
                    app.form_title = sub.title.clone().unwrap_or_default();
                    app.form_field = app::FormField::Name;
                    app.form_error = None;
                    app.editing_item_id = Some(sub.id.clone());
                    app.editing_subdomain = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_target_menu(key: KeyCode, app: &mut App) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.screen = Screen::Targets;
        }
        KeyCode::Down | KeyCode::Char('j') => app.target_menu_next(),
        KeyCode::Up | KeyCode::Char('k') => app.target_menu_previous(),
        KeyCode::Enter => {
            match app.target_menu_selected {
                0 => {
                    // Subdomains
                    app.subdomain_selected = 0;
                    app.subdomain_filter = None;
                    app.creating_subdomain = false;
                    app.screen = Screen::Subdomains;
                }
                1 => {
                    // IPs
                    app.ip_selected = 0;
                    app.creating_ip = false;
                    app.screen = Screen::IPs;
                }
                2 => {
                    // ASNs
                    app.asn_selected = 0;
                    app.creating_asn = false;
                    app.screen = Screen::ASNs;
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_ips(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(ip) = app.selected_ip().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_ip(db, &ip.id).ok();
                        app.ips = db::queries::list_ips(db, &ip.target_id).unwrap_or_default();
                        if app.ip_selected >= app.ips.len() && !app.ips.is_empty() {
                            app.ip_selected = app.ips.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_ip {
        match key {
            KeyCode::Esc => {
                app.editing_ip = false;
                app.reset_form();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("IP é obrigatório".to_string());
                    return Ok(());
                }
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };
                match db::queries::update_ip(
                    app.db.as_ref().unwrap(),
                    &id,
                    db::models::UpdateIp {
                        ip: app.form_name.trim().to_string(),
                    },
                ) {
                    Ok(_) => {
                        if let Some(target) = &app.current_target {
                            let tid = target.id.clone();
                            if let Some(db) = &app.db {
                                app.ips = db::queries::list_ips(db, &tid).unwrap_or_default();
                            }
                        }
                        app.editing_ip = false;
                        app.reset_form();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_ip {
        match key {
            KeyCode::Esc => {
                app.creating_ip = false;
                app.reset_form();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("IP é obrigatório".to_string());
                    return Ok(());
                }
                let target_id = match &app.current_target {
                    Some(t) => t.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewIp {
                    target_id: target_id.clone(),
                    ip: app.form_name.trim().to_string(),
                };
                match db::queries::create_ip(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_ip = false;
                        app.reset_form();
                        if let Some(db) = &app.db {
                            app.ips = db::queries::list_ips(db, &target_id).unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("IP já existe neste target".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.screen = Screen::Dashboard;
            }
            KeyCode::Down | KeyCode::Char('j') => app.ips_next(),
            KeyCode::Up | KeyCode::Char('k') => app.ips_previous(),
            KeyCode::Char('n') => {
                app.reset_form();
                app.creating_ip = true;
            }
            KeyCode::Char('d') => {
                if let Some(ip) = app.selected_ip() {
                    app.ask_confirm_delete(&ip.ip.clone());
                }
            }
            KeyCode::Char('e') => {
                if let Some(ip) = app.selected_ip().cloned() {
                    app.form_name = ip.ip.clone();
                    app.form_error = None;
                    app.editing_item_id = Some(ip.id.clone());
                    app.editing_ip = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_asns(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(asn) = app.selected_asn().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_asn(db, &asn.id).ok();
                        app.asns = db::queries::list_asns(db, &asn.target_id).unwrap_or_default();
                        if app.asn_selected >= app.asns.len() && !app.asns.is_empty() {
                            app.asn_selected = app.asns.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_asn {
        match key {
            KeyCode::Esc => {
                app.editing_asn = false;
                app.reset_form();
                app.form_org.clear();
            }
            KeyCode::Tab => {
                app.form_next_field();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("ASN é obrigatório".to_string());
                    return Ok(());
                }
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };
                match db::queries::update_asn(
                    app.db.as_ref().unwrap(),
                    &id,
                    db::models::UpdateAsn {
                        asn: app.form_name.trim().to_string(),
                        org: if app.form_org.trim().is_empty() {
                            None
                        } else {
                            Some(app.form_org.trim().to_string())
                        },
                    },
                ) {
                    Ok(_) => {
                        if let Some(target) = &app.current_target {
                            let tid = target.id.clone();
                            if let Some(db) = &app.db {
                                app.asns = db::queries::list_asns(db, &tid).unwrap_or_default();
                            }
                        }
                        app.editing_asn = false;
                        app.reset_form();
                        app.form_org.clear();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                match app.form_field {
                    app::FormField::Name => {
                        app.form_name.pop();
                    }
                    app::FormField::Description => {
                        app.form_org.pop();
                    }
                    _ => {}
                }
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                match app.form_field {
                    app::FormField::Name => app.form_name.push(c),
                    app::FormField::Description => app.form_org.push(c),
                    _ => {}
                }
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_asn {
        match key {
            KeyCode::Esc => {
                app.creating_asn = false;
                app.reset_form();
                app.form_org.clear();
            }
            KeyCode::Tab => {
                app.form_next_field();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("ASN é obrigatório".to_string());
                    return Ok(());
                }
                let target_id = match &app.current_target {
                    Some(t) => t.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewAsn {
                    target_id: target_id.clone(),
                    asn: app.form_name.trim().to_string(),
                    org: if app.form_org.trim().is_empty() {
                        None
                    } else {
                        Some(app.form_org.trim().to_string())
                    },
                };
                match db::queries::create_asn(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_asn = false;
                        app.reset_form();
                        app.form_org.clear();
                        if let Some(db) = &app.db {
                            app.asns = db::queries::list_asns(db, &target_id).unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("ASN já existe neste target".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                match app.form_field {
                    app::FormField::Name => {
                        app.form_name.pop();
                    }
                    app::FormField::Description => {
                        app.form_org.pop();
                    }
                    _ => {}
                }
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                match app.form_field {
                    app::FormField::Name => app.form_name.push(c),
                    app::FormField::Description => app.form_org.push(c),
                    _ => {}
                }
                app.form_error = None;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.screen = Screen::Dashboard;
            }
            KeyCode::Down | KeyCode::Char('j') => app.asns_next(),
            KeyCode::Up | KeyCode::Char('k') => app.asns_previous(),
            KeyCode::Char('n') => {
                app.reset_form();
                app.form_org.clear();
                app.creating_asn = true;
            }
            KeyCode::Char('d') => {
                if let Some(asn) = app.selected_asn() {
                    app.ask_confirm_delete(&asn.asn.clone());
                }
            }
            KeyCode::Char('e') => {
                if let Some(asn) = app.selected_asn().cloned() {
                    app.form_name = asn.asn.clone();
                    app.form_org = asn.org.clone().unwrap_or_default();
                    app.form_field = app::FormField::Name;
                    app.form_error = None;
                    app.editing_item_id = Some(asn.id.clone());
                    app.editing_asn = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_subdomain_menu(key: KeyCode, app: &mut App) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.screen = Screen::Subdomains;
        }
        KeyCode::Down | KeyCode::Char('j') => app.subdomain_menu_next(),
        KeyCode::Up | KeyCode::Char('k') => app.subdomain_menu_previous(),
        KeyCode::Enter => match app.subdomain_menu_selected {
            0 => {
                app.url_selected = 0;
                app.creating_url = false;
                app.screen = Screen::URLs;
            }
            1 => {
                app.technology_selected = 0;
                app.creating_technology = false;
                app.screen = Screen::Technologies;
            }
            2 => {
                app.creating_screenshot = false;
                app.screen = Screen::Screenshots;
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn handle_urls(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(url) = app.selected_url().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_url(db, &url.id).ok();
                        app.urls =
                            db::queries::list_urls(db, &url.subdomain_id).unwrap_or_default();
                        if app.url_selected >= app.urls.len() && !app.urls.is_empty() {
                            app.url_selected = app.urls.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_url {
        match key {
            KeyCode::Esc => {
                app.editing_url = false;
                app.reset_form();
            }
            KeyCode::Tab => {
                app.form_url_type = match app.form_url_type {
                    db::models::UrlType::Parameter => db::models::UrlType::JavaScript,
                    db::models::UrlType::JavaScript => db::models::UrlType::Endpoint,
                    db::models::UrlType::Endpoint => db::models::UrlType::Other,
                    db::models::UrlType::Other => db::models::UrlType::Parameter,
                };
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("URL é obrigatória".to_string());
                    return Ok(());
                }
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };
                match db::queries::update_url(
                    app.db.as_ref().unwrap(),
                    &id,
                    db::models::UpdateUrl {
                        url: app.form_name.trim().to_string(),
                        url_type: app.form_url_type.clone(),
                    },
                ) {
                    Ok(_) => {
                        if let Some(sub) = &app.current_subdomain {
                            let sid = sub.id.clone();
                            if let Some(db) = &app.db {
                                app.urls = db::queries::list_urls(db, &sid).unwrap_or_default();
                            }
                        }
                        app.editing_url = false;
                        app.reset_form();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_url {
        match key {
            KeyCode::Esc => {
                app.creating_url = false;
                app.reset_form();
            }
            KeyCode::Tab => {
                app.form_url_type = match app.form_url_type {
                    db::models::UrlType::Parameter => db::models::UrlType::JavaScript,
                    db::models::UrlType::JavaScript => db::models::UrlType::Endpoint,
                    db::models::UrlType::Endpoint => db::models::UrlType::Other,
                    db::models::UrlType::Other => db::models::UrlType::Parameter,
                };
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("URL é obrigatória".to_string());
                    return Ok(());
                }
                let sub_id = match &app.current_subdomain {
                    Some(s) => s.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewUrl {
                    subdomain_id: sub_id.clone(),
                    url: app.form_name.trim().to_string(),
                    url_type: app.form_url_type.clone(),
                };
                match db::queries::create_url(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_url = false;
                        app.reset_form();
                        if let Some(db) = &app.db {
                            app.urls = db::queries::list_urls(db, &sub_id).unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("URL já existe neste subdomain".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.screen = Screen::SubdomainMenu;
            }
            KeyCode::Down | KeyCode::Char('j') => app.urls_next(),
            KeyCode::Up | KeyCode::Char('k') => app.urls_previous(),
            KeyCode::Char('n') => {
                app.reset_form();
                app.form_url_type = db::models::UrlType::Other;
                app.creating_url = true;
            }
            KeyCode::Char('d') => {
                if let Some(url) = app.selected_url() {
                    app.ask_confirm_delete(&url.url.clone());
                }
            }
            KeyCode::Char('e') => {
                if let Some(url) = app.selected_url().cloned() {
                    app.form_name = url.url.clone();
                    app.form_url_type = url.url_type.clone();
                    app.form_error = None;
                    app.editing_item_id = Some(url.id.clone());
                    app.editing_url = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_technologies(key: KeyCode, app: &mut App) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(tech) = app.selected_technology().cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_technology(db, &tech.id).ok();
                        app.technologies = db::queries::list_technologies(db, &tech.subdomain_id)
                            .unwrap_or_default();
                        if app.technology_selected >= app.technologies.len()
                            && !app.technologies.is_empty()
                        {
                            app.technology_selected = app.technologies.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.editing_technology {
        match key {
            KeyCode::Esc => {
                app.editing_technology = false;
                app.reset_form();
                app.form_version.clear();
            }
            KeyCode::Tab => {
                app.form_next_field();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Nome é obrigatório".to_string());
                    return Ok(());
                }
                let id = match &app.editing_item_id {
                    Some(id) => id.clone(),
                    None => return Ok(()),
                };
                match db::queries::update_technology(
                    app.db.as_ref().unwrap(),
                    &id,
                    db::models::UpdateTechnology {
                        name: app.form_name.trim().to_string(),
                        version: if app.form_version.trim().is_empty() {
                            None
                        } else {
                            Some(app.form_version.trim().to_string())
                        },
                    },
                ) {
                    Ok(_) => {
                        if let Some(sub) = &app.current_subdomain {
                            let sid = sub.id.clone();
                            if let Some(db) = &app.db {
                                app.technologies =
                                    db::queries::list_technologies(db, &sid).unwrap_or_default();
                            }
                        }
                        app.editing_technology = false;
                        app.reset_form();
                        app.form_version.clear();
                    }
                    Err(e) => {
                        app.form_error = Some(format!("✗ {}", e));
                    }
                }
            }
            KeyCode::Backspace => {
                match app.form_field {
                    app::FormField::Name => {
                        app.form_name.pop();
                    }
                    app::FormField::Description => {
                        app.form_version.pop();
                    }
                    _ => {}
                }
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                match app.form_field {
                    app::FormField::Name => app.form_name.push(c),
                    app::FormField::Description => app.form_version.push(c),
                    _ => {}
                }
                app.form_error = None;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_technology {
        match key {
            KeyCode::Esc => {
                app.creating_technology = false;
                app.reset_form();
                app.form_version.clear();
            }
            KeyCode::Tab => {
                app.form_next_field();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Nome é obrigatório".to_string());
                    return Ok(());
                }
                let sub_id = match &app.current_subdomain {
                    Some(s) => s.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewTechnology {
                    subdomain_id: sub_id.clone(),
                    name: app.form_name.trim().to_string(),
                    version: if app.form_version.trim().is_empty() {
                        None
                    } else {
                        Some(app.form_version.trim().to_string())
                    },
                };
                match db::queries::create_technology(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_technology = false;
                        app.reset_form();
                        app.form_version.clear();
                        if let Some(db) = &app.db {
                            app.technologies =
                                db::queries::list_technologies(db, &sub_id).unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("Erro ao salvar tecnologia".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                match app.form_field {
                    app::FormField::Name => {
                        app.form_name.pop();
                    }
                    app::FormField::Description => {
                        app.form_version.pop();
                    }
                    _ => {}
                }
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                match app.form_field {
                    app::FormField::Name => app.form_name.push(c),
                    app::FormField::Description => app.form_version.push(c),
                    _ => {}
                }
                app.form_error = None;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.screen = Screen::SubdomainMenu;
            }
            KeyCode::Down | KeyCode::Char('j') => app.technologies_next(),
            KeyCode::Up | KeyCode::Char('k') => app.technologies_previous(),
            KeyCode::Char('n') => {
                app.reset_form();
                app.form_version.clear();
                app.creating_technology = true;
            }
            KeyCode::Char('d') => {
                if let Some(tech) = app.selected_technology() {
                    app.ask_confirm_delete(&tech.name.clone());
                }
            }
            KeyCode::Char('e') => {
                if let Some(tech) = app.selected_technology().cloned() {
                    app.form_name = tech.name.clone();
                    app.form_version = tech.version.clone().unwrap_or_default();
                    app.form_field = app::FormField::Name;
                    app.form_error = None;
                    app.editing_item_id = Some(tech.id.clone());
                    app.editing_technology = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_screenshots(
    key: KeyCode,
    app: &mut App,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<()> {
    if app.confirm_delete {
        match key {
            KeyCode::Enter => {
                if let Some(shot) = app.screenshots.get(app.screenshot_selected).cloned() {
                    if let Some(db) = &app.db {
                        db::queries::delete_screenshot(db, &shot.id).ok();
                        let sub_id = shot.subdomain_id.clone();
                        app.screenshots = db::queries::list_screenshots_by_subdomain(db, &sub_id)
                            .unwrap_or_default();
                        if app.screenshot_selected >= app.screenshots.len()
                            && !app.screenshots.is_empty()
                        {
                            app.screenshot_selected = app.screenshots.len() - 1;
                        }
                    }
                }
                app.cancel_confirm_delete();
            }
            KeyCode::Esc => {
                app.cancel_confirm_delete();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.creating_screenshot {
        match key {
            KeyCode::Esc => {
                app.creating_screenshot = false;
                app.reset_form();
            }
            KeyCode::Enter => {
                if app.form_name.trim().is_empty() {
                    app.form_error = Some("Caminho é obrigatório".to_string());
                    return Ok(());
                }
                let sub_id = match &app.current_subdomain {
                    Some(s) => s.id.clone(),
                    None => return Ok(()),
                };
                let new = db::models::NewScreenshot {
                    subdomain_id: sub_id.clone(),
                    file_path: app.form_name.trim().to_string(),
                };
                match db::queries::create_screenshot(app.db.as_ref().unwrap(), new) {
                    Ok(_) => {
                        app.creating_screenshot = false;
                        app.reset_form();
                        if let Some(db) = &app.db {
                            app.screenshots =
                                db::queries::list_screenshots_by_subdomain(db, &sub_id)
                                    .unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        app.form_error = Some("Erro ao salvar screenshot".to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                app.form_name.pop();
                app.form_error = None;
            }
            KeyCode::Char(c) => {
                app.form_name.push(c);
                app.form_error = None;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => {
                app.screen = Screen::SubdomainMenu;
            }
            KeyCode::Down | KeyCode::Char('j') => app.screenshots_next(),
            KeyCode::Up | KeyCode::Char('k') => app.screenshots_previous(),
            KeyCode::Char('n') => {
                app.reset_form();
                app.creating_screenshot = true;
            }
            KeyCode::Enter => {
                if let Some(shot) = app.screenshots.get(app.screenshot_selected) {
                    let path = shot.file_path.clone();
                    if !ui::image_preview::is_valid_image(&path) {
                        app.form_error =
                            Some("Arquivo não encontrado ou formato inválido".to_string());
                    } else if ui::image_preview::is_kitty_supported() {
                        ui::image_preview::show_kitty_preview(&path).ok();
                        terminal.clear().map_err(|e| {
                            error::RatariaError::IoError(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                e.to_string(),
                            ))
                        })?;
                    } else {
                        app.form_error = Some(
                            "Terminal não suporta preview — use O para abrir externamente"
                                .to_string(),
                        );
                    }
                }
            }
            KeyCode::Char('o') => {
                if let Some(shot) = app.screenshots.get(app.screenshot_selected) {
                    let path = shot.file_path.clone();
                    if ui::image_preview::is_valid_image(&path) {
                        ui::image_preview::open_with_system(&path).ok();
                    } else {
                        app.form_error =
                            Some("Arquivo não encontrado ou formato inválido".to_string());
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(shot) = app.screenshots.get(app.screenshot_selected) {
                    app.ask_confirm_delete(&shot.file_path.clone());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_import(key: KeyCode, app: &mut App) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.reset_import_form();
            app.screen = Screen::Home;
        }
        KeyCode::Tab => {
            app.import_field = match app.import_field {
                app::ImportField::Path => app::ImportField::Target,
                app::ImportField::Target => app::ImportField::Engagement,
                app::ImportField::Engagement => app::ImportField::Path,
            };
        }
        KeyCode::Enter => {
            let path = app.import_path.trim().to_string();

            if path.is_empty() {
                app.import_result = Some("✗ Caminho do arquivo é obrigatório".to_string());
                return Ok(());
            }

            // Lê o arquivo
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    app.import_result = Some(format!("✗ Erro ao ler arquivo: {}", e));
                    return Ok(());
                }
            };

            let filename = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let engagement = if app.import_engagement.trim().is_empty() {
                app.current_engagement.as_ref().map(|e| e.name.as_str())
            } else {
                Some(app.import_engagement.trim())
            };

            let target = if app.import_target.trim().is_empty() {
                None
            } else {
                Some(app.import_target.trim())
            };

            let db = app.db.as_ref().unwrap();

            match import::auto_import(db, &content, filename, engagement, target) {
                Ok(report) => {
                    app.import_result = Some(format!(
                        "✓ Importado: {} adicionados, {} ignorados (duplicatas)",
                        report.total_added(),
                        report.total_skipped(),
                    ));
                }
                Err(e) => {
                    app.import_result = Some(format!("✗ {}", e));
                }
            }
        }
        KeyCode::Backspace => {
            match app.import_field {
                app::ImportField::Path => {
                    app.import_path.pop();
                }
                app::ImportField::Target => {
                    app.import_target.pop();
                }
                app::ImportField::Engagement => {
                    app.import_engagement.pop();
                }
            }
            app.import_result = None;
        }
        KeyCode::Char(c) => {
            match app.import_field {
                app::ImportField::Path => app.import_path.push(c),
                app::ImportField::Target => app.import_target.push(c),
                app::ImportField::Engagement => app.import_engagement.push(c),
            }
            app.import_result = None;
        }
        _ => {}
    }
    Ok(())
}
