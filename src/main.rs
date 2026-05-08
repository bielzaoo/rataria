mod app;
mod auth;
mod db;
mod error;
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
        terminal
            .draw(|f| match app.screen {
                Screen::Password => ui::password::draw(f, app),
                Screen::Home => ui::home::draw(f, app),
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

                match app.screen {
                    Screen::Password => handle_password(key.code, app, &db_path)?,
                    Screen::Home => handle_home(key.code, app),
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
        KeyCode::Enter => match app.home_selected {
            2 => app.should_quit = true,
            _ => {}
        },
        _ => {}
    }
}
