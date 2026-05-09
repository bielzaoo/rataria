#![allow(dead_code, unused_imports)]

use chrono;
use db::models::SubdomainStatus;
use uuid;
mod app;
mod auth;
mod db;
mod error;
mod ui;
use app::{App, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use db::Database;
use db::{models, queries};
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
        // No loop de draw, troca:
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
            })
            .map_err(|e| {
                error::RatariaError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        if event::poll(std::time::Duration::from_millis(16))
            .map_err(error::RatariaError::IoError)?
        {
            if let Event::Key(key) = event::read().map_err(error::RatariaError::IoError)? {
                if key.kind != KeyEventKind::Press {
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
        KeyCode::Enter => {
            match app.home_selected {
                0 => {
                    // Abrir engagement existente
                    if let Some(db) = &app.db {
                        app.engagements = db::queries::list_engagements(db).unwrap_or_default();
                        app.engagement_selected = 0;
                        app.screen = Screen::ListEngagements;
                    }
                }
                1 => {
                    // Criar novo engagement
                    app.reset_form();
                    app.screen = Screen::CreateEngagement;
                }
                2 => app.should_quit = true,
                _ => {}
            }
        }
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
            }
            app.form_error = None;
        }
        KeyCode::Char(c) => {
            match app.form_field {
                app::FormField::Name => app.form_name.push(c),
                app::FormField::Description => app.form_description.push(c),
            }
            app.form_error = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_list_engagements(key: KeyCode, app: &mut App) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.screen = Screen::Home;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.engagements_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.engagements_previous();
        }
        KeyCode::Char('d') => {
            if let Some(eng) = app.selected_engagement().cloned() {
                let db = app.db.as_ref().unwrap();
                db::queries::delete_engagement(db, &eng.id).ok();
                // Recarrega a lista
                app.engagements = db::queries::list_engagements(db).unwrap_or_default();
                if app.engagement_selected >= app.engagements.len() && !app.engagements.is_empty() {
                    app.engagement_selected = app.engagements.len() - 1;
                }
            }
        }
        KeyCode::Enter => {
            if let Some(eng) = app.selected_engagement().cloned() {
                let eng_id = eng.id.clone();
                app.current_engagement = Some(eng);
                app.dashboard_selected = 0;

                // Carrega os targets do engagement selecionado
                if let Some(db) = &app.db {
                    app.targets = db::queries::list_targets(db, &eng_id).unwrap_or_default();
                }

                app.target_selected = 0;
                app.screen = Screen::Dashboard;
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
        KeyCode::Enter => {
            match app.dashboard_selected {
                0 => {
                    if let Some(eng) = &app.current_engagement {
                        let eng_id = eng.id.clone();
                        if let Some(db) = &app.db {
                            app.targets =
                                db::queries::list_targets(db, &eng_id).unwrap_or_default();
                        }
                    }
                    app.target_selected = 0;
                    app.creating_target = false;
                    app.screen = Screen::Targets;
                }
                1 => {
                    // Subdomains — precisa de um target selecionado
                    if let Some(target) = app.current_target.clone() {
                        if let Some(db) = &app.db {
                            app.subdomains =
                                db::queries::list_subdomains(db, &target.id).unwrap_or_default();
                        }
                        app.subdomain_selected = 0;
                        app.subdomain_filter = None;
                        app.creating_subdomain = false;
                        app.screen = Screen::Subdomains;
                    } else {
                        // Sem target selecionado, vai para targets primeiro
                        if let Some(eng) = &app.current_engagement {
                            let eng_id = eng.id.clone();
                            if let Some(db) = &app.db {
                                app.targets =
                                    db::queries::list_targets(db, &eng_id).unwrap_or_default();
                            }
                        }
                        app.target_selected = 0;
                        app.screen = Screen::Targets;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_targets(key: KeyCode, app: &mut App) -> Result<()> {
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
                    if let Some(db) = &app.db {
                        app.subdomains =
                            db::queries::list_subdomains(db, &target.id).unwrap_or_default();
                    }
                    app.current_target = Some(target);
                    app.subdomain_selected = 0;
                    app.subdomain_filter = None;
                    app.creating_subdomain = false;
                    app.screen = Screen::Subdomains;
                }
            }
            KeyCode::Char('d') => {
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
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_subdomains(key: KeyCode, app: &mut App) -> Result<()> {
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
                    };
                    if let Some(db) = &app.db {
                        db::queries::update_subdomain(db, &sub.id, update).ok();
                        let target_id = sub.target_id.clone();
                        app.subdomains =
                            db::queries::list_subdomains(db, &target_id).unwrap_or_default();
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
                app.screen = Screen::Targets;
                app.subdomain_filter = None;
            }
            KeyCode::Down | KeyCode::Char('j') => app.subdomains_next(),
            KeyCode::Up | KeyCode::Char('k') => app.subdomains_previous(),
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
                // Cicla o status do subdomain selecionado
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
                    };
                    if let Some(db) = &app.db {
                        db::queries::update_subdomain(db, &sub.id, update).ok();
                        app.subdomains =
                            db::queries::list_subdomains(db, &sub.target_id).unwrap_or_default();
                    }
                }
            }
            KeyCode::Char('f') => {
                // Cicla o filtro de status
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
            }
            _ => {}
        }
    }
    Ok(())
}
