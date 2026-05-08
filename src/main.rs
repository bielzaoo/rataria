#![allow(dead_code, unused_imports)]

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
                Screen::Dashboard => ui::list_engagements::draw(f, app), // placeholder
                Screen::Targets => ui::list_engagements::draw(f, app),   // placeholder
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
                    Screen::Dashboard => {} // placeholder
                    Screen::Targets => {}   // placeholder
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
                app.current_engagement = Some(eng);
                // Dashboard virá na próxima fase
            }
        }
        _ => {}
    }
    Ok(())
}
